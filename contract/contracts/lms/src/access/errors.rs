use soroban_sdk::contracterror;

/// Errors produced by the LMS access-control module.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AccessError {
    /// The caller does not have the required role.
    Unauthorized = 1,

    /// The target address is already registered.
    AlreadyRegistered = 2,

    /// The target address has not been registered.
    UserNotRegistered = 3,

    /// The operation requires administrator privileges.
    AdminRequired = 4,

    /// The operation requires instructor privileges.
    InstructorRequired = 5,

    /// No administrator has been initialized yet.
    AdminNotInitialized = 6,
}
