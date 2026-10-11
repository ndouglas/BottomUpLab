use super::{source, wire};
#[test]
fn selected_manifest_is_source_bound() {
    let binding = source::verify().unwrap();
    assert_eq!(binding.manifest_sha256, wire::sha256(wire::FIXTURE));
    assert_eq!(binding.compiled_inputs_sha256.len(), 64);
    assert_eq!(binding.executable_sha256.len(), 64);
}
#[test]
fn source_fingerprint_changes_on_map_mutation() {
    let root = std::path::Path::new(source::ROOT);
    let original = source::inputs::read(root, source::MANIFEST_PATH).unwrap();
    let mut changed = original.clone();
    changed
        .iter_mut()
        .find(|(p, _)| p.ends_with("sugar-map.txt"))
        .unwrap()
        .1
        .push(b' ');
    assert_ne!(
        source::inputs::fingerprint(original, source::MANIFEST_PATH).unwrap(),
        source::inputs::fingerprint(changed, source::MANIFEST_PATH).unwrap()
    );
}
