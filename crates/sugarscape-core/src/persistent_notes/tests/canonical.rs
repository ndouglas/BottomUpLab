use super::super::*;
#[test]
fn independent_sha_vectors() {
    assert_eq!(
        sha256::digest(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256::digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let mut hash = sha256::Sha256::new();
    for _ in 0..1_000_000 {
        hash.update(b"a");
    }
    assert_eq!(
        hash.finish(),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}
#[test]
fn canonical_sorted_utf8_and_controls() {
    let value = serde_json::json!({"z": [3, -2], "a": {"é": "\n\t\u{0001}\\\"", "b": null}});
    let mut out = Vec::new();
    canonical::write_record(&mut out, &value, 65536).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "{\"a\":{\"b\":null,\"é\":\"\\n\\t\\u0001\\\\\\\"\"},\"z\":[3,-2]}"
    );
}
#[test]
fn write_and_flush_failures_are_preserved() {
    struct Broken(bool);
    impl std::io::Write for Broken {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if self.0 {
                Err(std::io::Error::other("write failure"))
            } else {
                Ok(b.len())
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("flush failure"))
        }
    }
    assert!(matches!(
        canonical::write_record(&mut Broken(true), &1, 65536),
        Err(Error::Io(_))
    ));
    assert!(matches!(
        canonical::write_record_flush(&mut Broken(false), &1, 65536),
        Err(Error::Io(_))
    ));
}
#[test]
fn codec_writer_chunks_never_exceed_the_registered_buffer() {
    struct Bounded;
    impl std::io::Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            assert!(bytes.len() <= 65536);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    canonical::write_record(&mut Bounded, &"a".repeat(80000), 100000).unwrap();
}
