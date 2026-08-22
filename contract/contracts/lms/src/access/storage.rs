use soroban_sdk::{Address, Env};

use crate::StorageKey;

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
