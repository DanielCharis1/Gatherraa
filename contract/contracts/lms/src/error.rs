use soroban_sdk::contracterror;

/// Errors returned by the LMS contract.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// Reserved for future initialization/configuration failures.
    InitializationFailed = 1,
}
