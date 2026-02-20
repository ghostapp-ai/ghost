// Ghost Pro — Stub Implementation
//
// This is a stub crate for the open-source build.
// The real implementation lives in ghostapp-ai/ghost-pro (private).
//
// Pro developers: clone ghost-pro and replace this directory contents,
// or use [patch] in .cargo/config.toml:
//   [patch."https://github.com/ghostapp-ai/ghost-pro"]
//   ghost-pro = { path = "/path/to/your/ghost-pro" }

pub mod encryption;
pub mod sync;

/// Returns whether the user has an active Pro license.
/// Stub always returns false.
pub fn is_licensed() -> bool {
    false
}

/// Returns the Pro module version string.
/// Stub returns "0.0.0-stub".
pub fn version() -> &'static str {
    "0.0.0-stub"
}

/// Initialize the Pro subsystem (encryption, sync, etc.).
/// Stub is a no-op.
pub async fn initialize() -> anyhow::Result<()> {
    Ok(())
}
