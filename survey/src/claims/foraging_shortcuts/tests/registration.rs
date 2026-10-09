use super::super::{
    archive::{limits, raw_path, ArchiveReader, ArchiveWriter, Index, RawRef},
    manifest::{candidate, expected_keys, manifest_bytes},
    report::analyze,
    sha256,
    wire::{encode_envelope, WireEnvelope, WireManifest},
    CollectionMode,
};
use super::support::{
    cached_candidate_episode, owned_tempdir, test_provenance, TestArchiveFixture,
};
use std::{
    fs,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

static LEGACY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/superpowers/specs/2026-10-07-foraging-5-draft-manifest.json"
    ))
    .unwrap()
});
static ACTIVE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../docs/superpowers/specs/2026-10-08-foraging-5-registration-proposal.json"
    ))
    .unwrap()
});

// Saved historical evidence is assembled directly, never through the active-only writer.
fn historical_archive() -> TestArchiveFixture {
    static FILES: OnceLock<Vec<(PathBuf, Vec<u8>)>> = OnceLock::new();
    let files = FILES.get_or_init(|| {
        let m = candidate().unwrap();
        let keys = expected_keys(&m, CollectionMode::Construction);
        let mut p = test_provenance();
        p.manifest_sha256 = sha256(&LEGACY);
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
            manifest: serde_json::from_slice(&LEGACY).unwrap(),
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
    });
    let temp = owned_tempdir();
    for (relative, bytes) in files {
        let path = temp.path().join("saved").join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    TestArchiveFixture {
        index: temp.path().join("saved/index.json"),
        temp,
    }
}

#[test]
fn registered_disabled_factory_is_exact_reviewed_artifact() {
    assert!(
        manifest_bytes().unwrap() == *ACTIVE,
        "factory must equal the complete reviewed artifact bytes"
    );
    assert_eq!(
        sha256(&ACTIVE),
        "a96d3a678676b581a385880cf8f4fd80d0b737bcd0557b76ec9e77267a07d759"
    );
    assert_eq!(
        sha256(&LEGACY),
        "8e532eaf779b970d4bd9a2550fc1634f89abe63a00b7d05d3134f04a4a2deb62"
    );
}

#[test]
fn historical_construction_reanalysis_preserves_original_bytes() {
    let f = historical_archive();
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
        let expected = if file == "analysis.json" {
            "2b0a808d007bfbc399fcd0c70f56c91f39aaf4b4e23b915f2033ff46f4543694"
        } else {
            "fbeac26a842d456ec5b29b09283f2f4aaffc3db247b4ebd703477f18c17d9df0"
        };
        assert_eq!(
            sha256(&bytes),
            expected,
            "historical report changed from pre-registration baseline"
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

#[test]
fn both_saved_identities_reject_mutations_and_scientific_mode_before_output() {
    let f = historical_archive();
    let original: Index = serde_json::from_slice(&fs::read(&f.index).unwrap()).unwrap();
    for identity in [LEGACY.as_slice(), ACTIVE.as_slice()] {
        for mutation in 0..14 {
            let mut bad = original.clone();
            bad.manifest = serde_json::from_slice(identity).unwrap();
            bad.provenance.manifest_sha256 = sha256(identity);
            match mutation {
                0 => bad.manifest.status = "unknown".into(),
                1 => bad.manifest.execution_authorized = true,
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
    let f = historical_archive();
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
    let f = historical_archive();
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
fn historical_and_authorization_edited_manifests_cannot_create_new_archives() {
    let temp = owned_tempdir();
    for historical in [true, false] {
        let mut m = candidate().unwrap();
        if historical {
            let wire: WireManifest = serde_json::from_slice(&LEGACY).unwrap();
            m.status = wire.status;
            m.protocol = wire.protocol;
        } else {
            m.execution_authorized = true;
        }
        let mut p = test_provenance();
        if historical {
            p.manifest_sha256 = sha256(&LEGACY);
        }
        let out = temp.path().join("new");
        assert!(ArchiveWriter::create(
            &out,
            &m,
            CollectionMode::Construction,
            &p,
            "test",
            limits(&m)
        )
        .is_err());
        assert!(!out.exists());
    }
}
