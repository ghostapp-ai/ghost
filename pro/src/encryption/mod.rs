/// Stub encryption engine.
/// The real implementation uses ChaCha20-Poly1305 via the `age` crate.
pub struct EncryptionEngine;

impl EncryptionEngine {
    pub fn new() -> Self {
        Self
    }
}
