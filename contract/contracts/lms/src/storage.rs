use soroban_sdk::{contracttype, Address};

/// Storage keys used by the LMS contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    /// Reserved for future LMS configuration.
    Configuration,

    /// Access-control record for a registered LMS user.
    User(Address),
}
