// Ghost — Platform Extensions
//
// Extension trait for optional platform capabilities.
// The default implementation provides free-tier behavior (community edition).
//
// This pattern is inspired by Grafana's open-core architecture:
// the public repo defines interfaces, the private repo provides implementations.

/// Extension trait for optional platform capabilities.
///
/// The default implementation is a no-op that always returns community-edition
/// behavior. Override methods are injected at build time from an external crate.
#[allow(dead_code)]
pub trait PlatformExtensions: Send + Sync {
    /// Whether the current installation has an active license.
    fn is_licensed(&self) -> bool {
        false
    }

    /// Returns the edition version string.
    fn version(&self) -> &'static str {
        "community"
    }

    /// Initialize the extensions subsystem.
    fn initialize(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Community (free) edition — default no-op implementation.
pub struct CommunityEdition;

impl PlatformExtensions for CommunityEdition {}

/// Returns the active platform extensions provider.
///
/// In the open-source build this always returns [`CommunityEdition`].
pub fn extensions() -> &'static dyn PlatformExtensions {
    &CommunityEdition
}
