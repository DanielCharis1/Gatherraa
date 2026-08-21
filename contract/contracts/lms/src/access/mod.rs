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
    /// This operation is intended to be used during contract initialization.
    /// The administrator must authorize the registration.
    pub fn initialize_admin(env: &Env, admin: &Address) -> Result<(), AccessError> {
        admin.require_auth();

        storage::set_role(env, admin, Role::Admin)
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
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();

        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let student = Address::generate(&env);
        let outsider = Address::generate(&env);

        (env, admin, instructor, student, outsider)
    }

    #[test]
    fn initializes_admin() {
        let (env, admin, _, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        assert_eq!(AccessControl::get_role(&env, &admin), Some(Role::Admin));
    }

    #[test]
    fn duplicate_registration_is_rejected() {
        let (env, admin, _, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        assert_eq!(
            AccessControl::initialize_admin(&env, &admin),
            Err(AccessError::AlreadyRegistered)
        );
    }

    #[test]
    fn admin_can_register_another_admin() {
        let (env, admin, new_admin, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        AccessControl::register_admin(&env, &admin, &new_admin).unwrap();

        assert_eq!(AccessControl::get_role(&env, &new_admin), Some(Role::Admin));
    }

    #[test]
    fn non_admin_cannot_register_admin() {
        let (env, admin, _, student, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(
            AccessControl::register_admin(&env, &student, &student,),
            Err(AccessError::AdminRequired)
        );
    }

    #[test]
    fn admin_can_authorize_instructor() {
        let (env, admin, instructor, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        AccessControl::authorize_instructor(&env, &admin, &instructor).unwrap();

        assert_eq!(
            AccessControl::get_role(&env, &instructor),
            Some(Role::Instructor)
        );
    }

    #[test]
    fn non_admin_cannot_authorize_instructor() {
        let (env, admin, _, student, instructor) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(
            AccessControl::authorize_instructor(&env, &student, &instructor,),
            Err(AccessError::AdminRequired)
        );
    }

    #[test]
    fn student_can_register() {
        let (env, _, _, student, _) = setup();

        env.mock_all_auths();

        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(AccessControl::get_role(&env, &student), Some(Role::Student));
    }

    #[test]
    fn unknown_user_has_no_role() {
        let (env, _, _, _, outsider) = setup();

        assert_eq!(AccessControl::get_role(&env, &outsider), None);
    }

    #[test]
    fn admin_authorization_succeeds_for_admin() {
        let (env, admin, _, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        assert_eq!(AccessControl::require_admin(&env, &admin), Ok(()));
    }

    #[test]
    fn admin_authorization_rejects_student() {
        let (env, _, _, student, _) = setup();

        env.mock_all_auths();

        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(
            AccessControl::require_admin(&env, &student),
            Err(AccessError::AdminRequired)
        );
    }

    #[test]
    fn instructor_authorization_succeeds_for_instructor() {
        let (env, admin, instructor, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        AccessControl::authorize_instructor(&env, &admin, &instructor).unwrap();

        assert_eq!(AccessControl::require_instructor(&env, &instructor), Ok(()));
    }

    #[test]
    fn instructor_authorization_rejects_student() {
        let (env, _, _, student, _) = setup();

        env.mock_all_auths();

        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(
            AccessControl::require_instructor(&env, &student),
            Err(AccessError::InstructorRequired)
        );
    }

    #[test]
    fn staff_authorization_accepts_admin() {
        let (env, admin, _, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        assert_eq!(AccessControl::require_staff(&env, &admin), Ok(()));
    }

    #[test]
    fn staff_authorization_accepts_instructor() {
        let (env, admin, instructor, _, _) = setup();

        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        AccessControl::authorize_instructor(&env, &admin, &instructor).unwrap();

        assert_eq!(AccessControl::require_staff(&env, &instructor), Ok(()));
    }

    #[test]
    fn staff_authorization_rejects_student() {
        let (env, _, _, student, _) = setup();

        env.mock_all_auths();

        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(
            AccessControl::require_staff(&env, &student),
            Err(AccessError::Unauthorized)
        );
    }

    #[test]
    fn registered_authorization_rejects_unknown_user() {
        let (env, _, _, _, outsider) = setup();

        env.mock_all_auths();

        assert_eq!(
            AccessControl::require_registered(&env, &outsider),
            Err(AccessError::UserNotRegistered)
        );
    }
}
