use super::super::{
    archive::{limits, raw_path, ArchiveReader, ArchiveWriter, Index, RawRef},
    manifest::{
        authorize, candidate, canonical_bytes, expected_keys, manifest_bytes, resolve_saved,
        wire_manifest,
    },
    report::analyze,
    sha256,
    wire::{encode_envelope, WireEnvelope, WireManifest},
    CollectionMode,
};
use super::support::{
    cached_candidate_episode, owned_tempdir, test_provenance, TestArchiveFixture,
};
use std::{fs, path::PathBuf, sync::LazyLock};

static LEGACY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/superpowers/specs/2026-10-07-foraging-5-draft-manifest.json"
    ))
    .unwrap()
});
static CLOSED: LazyLock<Vec<u8>> = LazyLock::new(|| {
    fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/superpowers/specs/2026-10-08-foraging-5-registration-proposal.json"
    ))
    .unwrap()
});

// Saved historical evidence is assembled directly, never through the active-only writer.
fn historical_archive(identity: &[u8]) -> TestArchiveFixture {
    let files = {
        let m = candidate().unwrap();
        let keys = expected_keys(&m, CollectionMode::Construction);
        let mut p = test_provenance();
        p.manifest_sha256 = sha256(identity);
        let mut files = vec![];
        let mut runs = vec![];
        let mut total = 0;
        for key in &keys {
            let e = cached_candidate_episode(&key.condition, key.seed).unwrap();
            let bytes = encode_envelope(
                key,
                CollectionMode::Construction,
                &p,
                &e,
                m.raw_record_limit,
            )
            .unwrap();
            let path = raw_path(key);
            total += bytes.len() as u64;
            runs.push(RawRef {
                key: key.clone(),
                path: path.clone(),
                sha256: sha256(&bytes),
                bytes: bytes.len() as u64,
            });
            files.push((PathBuf::from(path), bytes));
        }
        let index = Index {
            schema: "foraging-shortcut-archive-v1".into(),
            manifest: serde_json::from_slice(identity).unwrap(),
            mode: CollectionMode::Construction,
            provenance: p,
            approval_context: "immutable historical construction fixture".into(),
            expected_keys: keys,
            completed: true,
            runs,
            raw_bytes: total,
        };
        files.push((
            PathBuf::from("index.json"),
            serde_json::to_vec(&index).unwrap(),
        ));
        files
    };
    let temp = owned_tempdir();
    for (relative, bytes) in &files {
        let path = temp.path().join("saved").join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    TestArchiveFixture {
        index: temp.path().join("saved/index.json"),
        temp,
    }
}

static ACTIVE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/superpowers/specs/2026-10-09-foraging-5-authorized-manifest.json"
    ))
    .unwrap()
});

#[test]
fn authorized_factory_changes_only_approval_and_pins_all_three_identities() {
    let active = candidate().unwrap();
    assert!(active.execution_authorized);
    assert_eq!(manifest_bytes().unwrap(), *ACTIVE);
    let mut closed = active.clone();
    closed.execution_authorized = false;
    assert_eq!(canonical_bytes(&closed).unwrap(), *CLOSED);
    for (bytes, expected, authorized) in [
        (
            &*ACTIVE,
            "3a78882035f89758659131da929b32d78d6af0b29f890aeb9bce4f03f0825216",
            true,
        ),
        (
            &*CLOSED,
            "a96d3a678676b581a385880cf8f4fd80d0b737bcd0557b76ec9e77267a07d759",
            false,
        ),
        (
            &*LEGACY,
            "8e532eaf779b970d4bd9a2550fc1634f89abe63a00b7d05d3134f04a4a2deb62",
            false,
        ),
    ] {
        assert_eq!(sha256(bytes), expected);
        let wire = serde_json::from_slice(bytes).unwrap();
        let resolved = resolve_saved(&wire, expected).unwrap();
        assert_eq!(wire_manifest(&resolved).unwrap(), wire);
        assert_eq!(resolved.execution_authorized, authorized);
        assert_eq!(
            authorize(&resolved, CollectionMode::Scientific).is_ok(),
            authorized
        );
        let keys = expected_keys(&resolved, CollectionMode::Scientific);
        let expected_keys: Vec<_> = resolved
            .conditions
            .iter()
            .flat_map(|c| {
                (10001..=10040).map(move |seed| super::super::RunKey {
                    condition: c.id.clone(),
                    seed,
                })
            })
            .collect();
        assert_eq!(keys, expected_keys);
        assert_eq!(keys.len(), 600);
    }
}

#[test]
fn historical_construction_reanalysis_preserves_original_bytes() {
    for identity in [LEGACY.as_slice(), CLOSED.as_slice()] {
        let f = historical_archive(identity);
        let index_before = fs::read(&f.index).unwrap();
        let index: Index = serde_json::from_slice(&index_before).unwrap();
        let raw_before: Vec<_> = index
            .runs
            .iter()
            .map(|r| fs::read(f.index.parent().unwrap().join(&r.path)).unwrap())
            .collect();
        let mut reader = ArchiveReader::open(&f.index).unwrap();
        let mut count = 0;
        while reader.next().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 30);
        for out in ["first", "second"] {
            analyze(&f.index, &f.temp.path().join(out)).unwrap();
        }
        for file in ["analysis.json", "results.md"] {
            let bytes = fs::read(f.temp.path().join("first").join(file)).unwrap();
            assert_eq!(
                bytes,
                fs::read(f.temp.path().join("second").join(file)).unwrap()
            );
            let expected = if identity == LEGACY.as_slice() && file == "analysis.json" {
                "2b0a808d007bfbc399fcd0c70f56c91f39aaf4b4e23b915f2033ff46f4543694"
            } else if identity == LEGACY.as_slice() {
                "fbeac26a842d456ec5b29b09283f2f4aaffc3db247b4ebd703477f18c17d9df0"
            } else if file == "analysis.json" {
                "3eb028131dab1e3c2337cf2c62e03047ff07d5290787885f954562832495dae6"
            } else {
                "d9c3b7281a244f6b37934d1a0e5b29896820cb1da90c0c60e5ca1fd37073777a"
            };
            assert_eq!(
                sha256(&bytes),
                expected,
                "historical report changed from pre-activation baseline"
            );
        }
        assert_eq!(fs::read(&f.index).unwrap(), index_before);
        for (r, before) in index.runs.iter().zip(raw_before) {
            assert_eq!(
                fs::read(f.index.parent().unwrap().join(&r.path)).unwrap(),
                before
            );
        }
    }
}

#[test]
fn all_saved_identities_reject_mutations_and_scientific_mode_before_output() {
    let f = historical_archive(&LEGACY);
    let original: Index = serde_json::from_slice(&fs::read(&f.index).unwrap()).unwrap();
    for identity in [LEGACY.as_slice(), CLOSED.as_slice(), ACTIVE.as_slice()] {
        for mutation in 0..14 {
            let mut bad = original.clone();
            bad.manifest = serde_json::from_slice(identity).unwrap();
            bad.provenance.manifest_sha256 = sha256(identity);
            match mutation {
                0 => bad.manifest.status = "unknown".into(),
                1 => bad.manifest.execution_authorized = !bad.manifest.execution_authorized,
                2 => bad.manifest.protocol.push_str(".changed"),
                3 => bad.manifest.conditions[0].setup.width += 1,
                4 => bad.manifest.conditions[0].options.ticks -= 1,
                5 => bad.provenance.manifest_sha256 = "d".repeat(64),
                6 => {
                    bad.provenance.manifest_sha256 = sha256(if identity == LEGACY.as_slice() {
                        &ACTIVE
                    } else {
                        &LEGACY
                    })
                }
                7 => bad.expected_keys[0].seed = 10001,
                8 => bad.mode = CollectionMode::Scientific,
                9 => bad.manifest.metadata_limit = u64::MAX,
                10 => bad.manifest.conditions.swap(0, 1),
                11 => bad.manifest.construction_seeds.push(9),
                12 => bad.manifest.scientific_seeds[0] += 1,
                _ => bad.manifest.conditions[0].setup.parameters.p_search.0 = "0.050".into(),
            }
            if matches!(mutation, 0..=4 | 9..=13) {
                let mut bytes = serde_json::to_vec_pretty(&bad.manifest).unwrap();
                bytes.push(b'\n');
                bad.provenance.manifest_sha256 = sha256(&bytes);
            }
            fs::write(&f.index, serde_json::to_vec(&bad).unwrap()).unwrap();
            let out = f.temp.path().join(format!("refused-{mutation}"));
            assert!(
                analyze(&f.index, &out).is_err(),
                "accepted identity mutation {mutation}"
            );
            assert!(!out.exists());
        }
    }
}

#[test]
fn active_index_with_historical_envelopes_fails_closed() {
    let f = historical_archive(&LEGACY);
    let mut index: Index = serde_json::from_slice(&fs::read(&f.index).unwrap()).unwrap();
    index.manifest = serde_json::from_slice(&ACTIVE).unwrap();
    index.provenance.manifest_sha256 = sha256(&ACTIVE);
    fs::write(&f.index, serde_json::to_vec(&index).unwrap()).unwrap();
    assert!(analyze(&f.index, &f.temp.path().join("mixed"))
        .unwrap_err()
        .contains("provenance mismatch"));
    assert!(!f.temp.path().join("mixed").exists());
}

#[test]
fn historical_index_rejects_a_resigned_active_envelope_at_the_last_key() {
    let f = historical_archive(&LEGACY);
    let mut index: Index = serde_json::from_slice(&fs::read(&f.index).unwrap()).unwrap();
    let reference = index.runs.last_mut().unwrap();
    let path = f.index.parent().unwrap().join(&reference.path);
    let mut envelope: WireEnvelope = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    envelope.provenance.manifest_sha256 = sha256(&ACTIVE);
    let bytes = serde_json::to_vec(&envelope).unwrap();
    index.raw_bytes = index.raw_bytes - reference.bytes + bytes.len() as u64;
    reference.bytes = bytes.len() as u64;
    reference.sha256 = sha256(&bytes);
    fs::write(path, bytes).unwrap();
    fs::write(&f.index, serde_json::to_vec(&index).unwrap()).unwrap();
    let out = f.temp.path().join("mixed-last");
    assert!(analyze(&f.index, &out)
        .unwrap_err()
        .contains("provenance mismatch"));
    assert!(!out.exists());
}

#[test]
fn both_historical_identities_cannot_create_new_archives() {
    let temp = owned_tempdir();
    for bytes in [LEGACY.as_slice(), CLOSED.as_slice()] {
        let wire: WireManifest = serde_json::from_slice(bytes).unwrap();
        let m = resolve_saved(&wire, &sha256(bytes)).unwrap();
        let mut p = test_provenance();
        p.manifest_sha256 = sha256(bytes);
        for mode in [CollectionMode::Construction, CollectionMode::Scientific] {
            let out = temp.path().join("new");
            assert!(ArchiveWriter::create(&out, &m, mode, &p, "test", limits(&m)).is_err());
            assert!(!out.exists());
        }
    }
}

#[test]
fn authorized_scientific_writer_records_exact_complete_keyset_without_episodes() {
    let temp = owned_tempdir();
    let out = temp.path().join("authorized-preflight");
    let m = candidate().unwrap();
    let _writer = ArchiveWriter::create(
        &out,
        &m,
        CollectionMode::Scientific,
        &test_provenance(),
        "explicit full run approval",
        limits(&m),
    )
    .unwrap();
    let index: Index =
        serde_json::from_slice(&fs::read(out.join("index.incomplete.json")).unwrap()).unwrap();
    assert!(!index.completed);
    assert!(index.runs.is_empty());
    assert_eq!(
        index.expected_keys,
        expected_keys(&m, CollectionMode::Scientific)
    );
    assert_eq!(index.expected_keys.len(), 600);
    assert!(!out.join("index.json").exists());
}
