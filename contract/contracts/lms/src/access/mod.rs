pub mod errors;
pub mod storage;
pub mod types;

pub use errors::AccessError;
pub use types::{Role, UserRecord};

use soroban_sdk::{Address, Env};

/// Access-control operations for the LMS contract.
pub struct AccessControl;

impl AccessControl {
    /// Register the first LMS administrator.
    ///
    /// This runs exactly once, at contract initialization. It is the only
    /// route to an administrator role that does not require an existing
    /// administrator's approval, which is precisely why it must be a
    /// one-time event.
    ///
    /// The guard is on the contract, not on the address. Checking only that
    /// *this* address is unregistered would leave the door open: any fresh
    /// address could call this after launch and appoint itself
    /// administrator, then hand out instructor and administrator roles at
    /// will. `AlreadyRegistered` protects one address; `AlreadyInitialized`
    /// protects the contract.
    ///
    /// # Errors
    /// * `AlreadyInitialized` — initialization has already happened
    /// * `AlreadyRegistered` — the address somehow already holds a role
    pub fn initialize_admin(env: &Env, admin: &Address) -> Result<(), AccessError> {
        admin.require_auth();

        if storage::is_initialized(env) {
            return Err(AccessError::AlreadyInitialized);
        }

        storage::set_role(env, admin, Role::Admin)?;
        storage::mark_initialized(env);

        Ok(())
    }

    /// Whether the contract has been initialized.
    pub fn is_initialized(env: &Env) -> bool {
        storage::is_initialized(env)
    }

    /// Register an additional administrator.
    ///
    /// Only an existing administrator may authorize another administrator.
    pub fn register_admin(env: &Env, caller: &Address, admin: &Address) -> Result<(), AccessError> {
        Self::require_admin(env, caller)?;

        storage::set_role(env, admin, Role::Admin)
    }

    /// Authorize an instructor.
    ///
    /// Only an administrator may authorize instructors.
    pub fn authorize_instructor(
        env: &Env,
        caller: &Address,
        instructor: &Address,
    ) -> Result<(), AccessError> {
        Self::require_admin(env, caller)?;

        storage::set_role(env, instructor, Role::Instructor)
    }

    /// Register a student.
    ///
    /// Students must authorize their own registration. This prevents a
    /// third party from registering an address as a student without consent.
    pub fn register_student(env: &Env, student: &Address) -> Result<(), AccessError> {
        student.require_auth();

        storage::set_role(env, student, Role::Student)
    }

    /// Look up the role assigned to an address.
    pub fn get_role(env: &Env, user: &Address) -> Option<Role> {
        storage::get_role(env, user)
    }

    /// Look up the complete access-control record for an address.
    pub fn get_user(env: &Env, user: &Address) -> Option<UserRecord> {
        storage::get_user(env, user)
    }

    /// Check whether an address has a specific role.
    pub fn has_role(env: &Env, user: &Address, expected_role: Role) -> bool {
        Self::get_role(env, user) == Some(expected_role)
    }

    /// Require administrator authorization.
    pub fn require_admin(env: &Env, caller: &Address) -> Result<(), AccessError> {
        caller.require_auth();

        match Self::get_role(env, caller) {
            Some(Role::Admin) => Ok(()),
            Some(_) => Err(AccessError::AdminRequired),
            None => Err(AccessError::UserNotRegistered),
        }
    }

    /// Require instructor authorization.
    pub fn require_instructor(env: &Env, caller: &Address) -> Result<(), AccessError> {
        caller.require_auth();

        match Self::get_role(env, caller) {
            Some(Role::Instructor) => Ok(()),
            Some(_) => Err(AccessError::InstructorRequired),
            None => Err(AccessError::UserNotRegistered),
        }
    }

    /// Require either administrator or instructor privileges.
    pub fn require_staff(env: &Env, caller: &Address) -> Result<(), AccessError> {
        caller.require_auth();

        match Self::get_role(env, caller) {
            Some(Role::Admin | Role::Instructor) => Ok(()),
            Some(Role::Student) => Err(AccessError::Unauthorized),
            None => Err(AccessError::UserNotRegistered),
        }
    }

    /// Require the caller to be a registered LMS user.
    pub fn require_registered(env: &Env, caller: &Address) -> Result<(), AccessError> {
        caller.require_auth();

        if storage::has_user(env, caller) {
            Ok(())
        } else {
            Err(AccessError::UserNotRegistered)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LmsContract;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    /// Run one contract call.
    ///
    /// These functions touch contract storage, which the host only permits
    /// inside a contract invocation — calling them straight from a test
    /// fails with `Error(Context, InternalError)`, "no contract running".
    ///
    /// Each call also needs its own frame. Two `require_auth()` calls on the
    /// same address within a single frame fail with
    /// `Error(Auth, ExistingValue)`, "frame is already authorized", so
    /// several operations cannot share one `as_contract` block. One frame
    /// per call matches how these functions are actually reached anyway:
    /// one invocation per transaction.
    fn call<T>(env: &Env, contract_id: &Address, f: impl FnOnce() -> T) -> T {
        env.as_contract(contract_id, f)
    }

    fn setup() -> (Env, Address, Address, Address, Address, Address) {
        let env = Env::default();
        let contract_id = env.register(LmsContract, ());

        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let student = Address::generate(&env);
        let outsider = Address::generate(&env);

        (env, contract_id, admin, instructor, student, outsider)
    }

    #[test]
    fn initializes_admin() {
        let (env, id, admin, _, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::get_role(&env, &admin)),
            Some(Role::Admin)
        );
    }

    #[test]
    fn re_initializing_with_the_same_admin_is_rejected() {
        let (env, id, admin, _, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::initialize_admin(&env, &admin)),
            Err(AccessError::AlreadyInitialized)
        );
    }

    /// The one that matters. Guarding only the address would let any fresh
    /// address self-appoint as administrator after launch, which is a full
    /// privilege escalation: an attacker-admin can then authorize
    /// instructors and mint further administrators at will.
    #[test]
    fn a_second_address_cannot_initialize_after_launch() {
        let (env, id, founder, _, _, attacker) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &founder).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::initialize_admin(
                &env, &attacker
            )),
            Err(AccessError::AlreadyInitialized)
        );

        // The attacker gained nothing at all.
        assert_eq!(
            call(&env, &id, || AccessControl::get_role(&env, &attacker)),
            None
        );

        // And the legitimate admin is untouched.
        assert_eq!(
            call(&env, &id, || AccessControl::get_role(&env, &founder)),
            Some(Role::Admin)
        );
    }

    #[test]
    fn initialization_state_is_reported() {
        let (env, id, admin, _, _, _) = setup();

        env.mock_all_auths();

        assert!(!call(&env, &id, || AccessControl::is_initialized(&env)));

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });

        assert!(call(&env, &id, || AccessControl::is_initialized(&env)));
    }

    /// A rejected initialization must not leave the marker set, or a failed
    /// deployment attempt would brick the contract permanently.
    #[test]
    fn a_failed_initialization_leaves_the_contract_uninitialized() {
        let (env, id, _, _, student, _) = setup();

        env.mock_all_auths();

        // Registering as a student first makes `set_role` fail inside
        // `initialize_admin`, after the guard has been passed.
        call(&env, &id, || {
            AccessControl::register_student(&env, &student).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::initialize_admin(
                &env, &student
            )),
            Err(AccessError::AlreadyRegistered)
        );

        assert!(
            !call(&env, &id, || AccessControl::is_initialized(&env)),
            "a failed initialization must not mark the contract initialized"
        );

        // Recovery is still possible.
        let admin = Address::generate(&env);
        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });
        assert!(call(&env, &id, || AccessControl::is_initialized(&env)));
    }

    #[test]
    fn admin_can_register_another_admin() {
        let (env, id, admin, new_admin, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });
        call(&env, &id, || {
            AccessControl::register_admin(&env, &admin, &new_admin).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::get_role(&env, &new_admin)),
            Some(Role::Admin)
        );
    }

    #[test]
    fn non_admin_cannot_register_admin() {
        let (env, id, admin, _, student, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });
        call(&env, &id, || {
            AccessControl::register_student(&env, &student).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::register_admin(
                &env, &student, &student
            )),
            Err(AccessError::AdminRequired)
        );
    }

    #[test]
    fn admin_can_authorize_instructor() {
        let (env, id, admin, instructor, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });
        call(&env, &id, || {
            AccessControl::authorize_instructor(&env, &admin, &instructor).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::get_role(&env, &instructor)),
            Some(Role::Instructor)
        );
    }

    #[test]
    fn non_admin_cannot_authorize_instructor() {
        let (env, id, admin, instructor, student, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });
        call(&env, &id, || {
            AccessControl::register_student(&env, &student).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::authorize_instructor(
                &env,
                &student,
                &instructor
            )),
            Err(AccessError::AdminRequired)
        );
    }

    #[test]
    fn student_can_register() {
        let (env, id, _, _, student, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::register_student(&env, &student).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::get_role(&env, &student)),
            Some(Role::Student)
        );
    }

    #[test]
    fn unknown_user_has_no_role() {
        let (env, id, _, _, _, outsider) = setup();

        assert_eq!(
            call(&env, &id, || AccessControl::get_role(&env, &outsider)),
            None
        );
    }

    #[test]
    fn admin_authorization_succeeds_for_admin() {
        let (env, id, admin, _, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::require_admin(&env, &admin)),
            Ok(())
        );
    }

    #[test]
    fn admin_authorization_rejects_student() {
        let (env, id, _, _, student, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::register_student(&env, &student).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::require_admin(&env, &student)),
            Err(AccessError::AdminRequired)
        );
    }

    #[test]
    fn instructor_authorization_succeeds_for_instructor() {
        let (env, id, admin, instructor, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });
        call(&env, &id, || {
            AccessControl::authorize_instructor(&env, &admin, &instructor).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::require_instructor(
                &env,
                &instructor
            )),
            Ok(())
        );
    }

    #[test]
    fn instructor_authorization_rejects_student() {
        let (env, id, _, _, student, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::register_student(&env, &student).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::require_instructor(
                &env, &student
            )),
            Err(AccessError::InstructorRequired)
        );
    }

    #[test]
    fn staff_authorization_accepts_admin() {
        let (env, id, admin, _, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::require_staff(&env, &admin)),
            Ok(())
        );
    }

    #[test]
    fn staff_authorization_accepts_instructor() {
        let (env, id, admin, instructor, _, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::initialize_admin(&env, &admin).unwrap()
        });
        call(&env, &id, || {
            AccessControl::authorize_instructor(&env, &admin, &instructor).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::require_staff(
                &env,
                &instructor
            )),
            Ok(())
        );
    }

    #[test]
    fn staff_authorization_rejects_student() {
        let (env, id, _, _, student, _) = setup();

        env.mock_all_auths();

        call(&env, &id, || {
            AccessControl::register_student(&env, &student).unwrap()
        });

        assert_eq!(
            call(&env, &id, || AccessControl::require_staff(&env, &student)),
            Err(AccessError::Unauthorized)
        );
    }

    #[test]
    fn registered_authorization_rejects_unknown_user() {
        let (env, id, _, _, _, outsider) = setup();

        env.mock_all_auths();

        assert_eq!(
            call(&env, &id, || AccessControl::require_registered(
                &env, &outsider
            )),
            Err(AccessError::UserNotRegistered)
        );
    }
}
