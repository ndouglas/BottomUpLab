use std::io::Write;
#[test]
fn independent_sha256_known_answers_and_chunk_boundaries() {
    for (bytes, expected) in [
        (
            b"".as_slice(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".as_slice(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_slice(),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ] {
        assert_eq!(super::super::sha256::sha256(bytes), expected);
        for n in [1, 3, 7, 55, 56, 63, 64, 65] {
            let mut d = super::super::sha256::Digest::default();
            for chunk in bytes.chunks(n) {
                d.write_all(chunk).unwrap();
            }
            assert_eq!(d.finish(), expected);
        }
    }
}
#[test]
fn canonical_unicode_control_bytes_are_exact() {
    assert_eq!(
        super::super::canonical::canonical_bytes(
            &serde_json::json!({"z":"é😀/\u{0001}\n\t\r\u{0008}\u{000c}\\\"","a":-12})
        )
        .unwrap(),
        b"{\"a\":-12,\"z\":\"\xc3\xa9\xf0\x9f\x98\x80/\\u0001\\n\\t\\r\\b\\f\\\\\\\"\"}"
    );
}
#[test]
fn sha256_million_a_streamed_has_independent_answer() {
    let data = vec![b'a'; 1_000_000];
    let expected = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";
    assert_eq!(super::super::sha256::sha256(&data), expected);
    for n in [55, 56, 63, 64, 65, 127] {
        let mut d = super::super::sha256::Digest::default();
        for chunk in data.chunks(n) {
            d.write_all(chunk).unwrap();
        }
        assert_eq!(d.finish(), expected);
    }
}
#[test]
fn unexecuted_words_remain_in_trace_identity() {
    let c = super::config();
    let a = super::super::run_episode(&c, super::super::Profile::new(486, 60).unwrap()).unwrap();
    let b = super::super::run_episode(&c, super::super::Profile::new(488, 63).unwrap()).unwrap();
    assert_eq!(a.events, b.events);
    assert_eq!(a.outcome, b.outcome);
    assert_ne!(
        super::super::trace_hash(&a).unwrap(),
        super::super::trace_hash(&b).unwrap()
    );
}
