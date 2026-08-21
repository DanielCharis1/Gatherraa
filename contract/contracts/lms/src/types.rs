use soroban_sdk::contracttype;

/// Version of the LMS contract interface.
///
/// Keeping this as a contract type makes the version part of the
/// Soroban contract's type system rather than exposing a magic integer.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LmsVersion {
    V1,
}
