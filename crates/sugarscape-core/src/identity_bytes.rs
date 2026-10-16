//! Raw identity preimages stay embedded on native; WASM uses generated receipts.
macro_rules! identity_bytes {
    ($p:expr) => {{
        #[cfg(not(target_arch = "wasm32"))]
        {
            include_bytes!($p).as_slice()
        }
        #[cfg(target_arch = "wasm32")]
        {
            b"".as_slice()
        }
    }};
}
pub(crate) use identity_bytes;
