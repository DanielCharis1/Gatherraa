use soroban_sdk::contract;

/// Root contract for the Learning Management System.
///
/// This contract intentionally exposes no business functionality yet.
/// It establishes the Soroban contract entry point for future LMS modules.
#[contract]
pub struct LmsContract;

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn contract_can_be_registered() {
        let env = Env::default();

        let _contract_id = env.register(LmsContract, ());
    }
}
