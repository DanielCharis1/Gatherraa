use soroban_sdk::{contracttype, Address};

/// Roles supported by the LMS access-control system.
///
/// The enum is represented using Soroban's contract type system so it can
/// safely be persisted in contract storage and returned from contract calls.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Instructor,
    Student,
}

/// Persistent access-control record for an LMS user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRecord {
    pub address: Address,
    pub role: Role,
}
