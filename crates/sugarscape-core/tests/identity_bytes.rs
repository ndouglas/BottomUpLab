#[path = "../src/identity_bytes.rs"]
mod identity_bytes;
use identity_bytes::identity_bytes;

#[test]
fn native_identity_bytes_preserve_raw_bytes_and_call_site_paths() {
    let expected = include_bytes!("../Cargo.toml");
    assert_eq!(identity_bytes!("../Cargo.toml"), expected.as_slice());
    assert_eq!(
        identity_bytes!(concat!("../", "Cargo.toml")),
        expected.as_slice()
    );
}
