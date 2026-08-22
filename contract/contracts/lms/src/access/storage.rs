use soroban_sdk::{Address, Env};

use crate::{LmsVersion, StorageKey};

use super::{
    errors::AccessError,
    types::{Role, UserRecord},
};

/// Returns whether the given address has a registered LMS role.
pub fn has_user(env: &Env, user: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&StorageKey::User(user.clone()))
}

/// Returns the role assigned to a user.
pub fn get_role(env: &Env, user: &Address) -> Option<Role> {
    env.storage()
        .persistent()
        .get(&StorageKey::User(user.clone()))
}

/// Returns the complete access-control record for a user.
pub fn get_user(env: &Env, user: &Address) -> Option<UserRecord> {
    get_role(env, user).map(|role| UserRecord {
        address: user.clone(),
        role,
    })
}

/// Persist a role for a new user.
///
/// Registration policy belongs to the access-control service. This
/// function only protects the storage invariant that one address can
/// only have one initial registration.
pub fn set_role(env: &Env, user: &Address, role: Role) -> Result<(), AccessError> {
    if has_user(env, user) {
        return Err(AccessError::AlreadyRegistered);
    }

    env.storage()
        .persistent()
        .set(&StorageKey::User(user.clone()), &role);

    Ok(())
}

/// Returns whether the contract has been initialized.
///
/// The marker lives in instance storage rather than persistent storage
/// because it is contract-level configuration: it shares the contract's own
/// lifetime and archival, and there is exactly one of it. Per-user records
/// stay in persistent storage, where they are keyed and extended
/// individually.
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::Configuration)
}

/// Mark the contract as initialized.
///
/// Stores the interface version rather than a bare flag, so the same entry
/// answers "has this been initialized" and "which interface version was it
/// initialized at" — the latter being what a future migration would need.
pub fn mark_initialized(env: &Env) {
    env.storage()
        .instance()
        .set(&StorageKey::Configuration, &LmsVersion::V1);
}
