use soroban_sdk::contracttype;

/// Storage keys reserved for the LMS contract.
///
/// No keys are persisted yet. Concrete storage entries should be added
/// alongside the feature that owns them.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    /// Reserved for future LMS configuration.
    Configuration,
}
