pub mod errors;
pub mod scoring;
pub mod storage;
pub mod types;

pub use errors::AssessmentError;
pub use types::{AssessmentConfig, AssessmentResult};

use soroban_sdk::{Address, Env};

use crate::access::{AccessControl, Role};
use crate::events;

/// Assessment submission and result operations for the LMS contract.
pub struct AssessmentService;

impl AssessmentService {
    /// Register or update the configuration for an assessment.
    ///
    /// Only staff (admin or instructor) may configure assessments.
    pub fn configure_assessment(
        env: &Env,
        caller: &Address,
        assessment_id: u64,
        config: AssessmentConfig,
    ) -> Result<(), AssessmentError> {
        AccessControl::require_staff(env, caller).map_err(|_| AssessmentError::Unauthorized)?;

        storage::set_config(env, assessment_id, &config);

        Ok(())
    }

    /// Submit a student's score for an assessment.
    ///
    /// - Requires the student's own authorization.
    /// - Requires the caller be registered with the Student role.
    ///   NOTE: this checks *role* registration, not enrollment in a specific
    ///   course/cohort — if the LMS has (or will have) a separate
    ///   course-enrollment concept, swap this check for that instead.
    /// - Enforces the assessment's configured attempt limit.
    /// - Persists and returns the result.
    pub fn submit_assessment(
        env: &Env,
        student: Address,
        assessment_id: u64,
        score: u32,
    ) -> Result<AssessmentResult, AssessmentError> {
        student.require_auth();

        if !AccessControl::has_role(env, &student, Role::Student) {
            return Err(AssessmentError::NotEnrolled);
        }

        let config =
        storage::get_config(env, assessment_id).ok_or(AssessmentError::AssessmentNotFound)?;

        let passed = scoring::validate_score(score, &config)?;

        let current_attempts = storage::get_attempt_count(env, &student, assessment_id);
        let attempt = scoring::check_attempt_limit(current_attempts, config.max_attempts)?;
        let result = AssessmentResult {
            student: student.clone(),
            assessment_id,
            score,
            passed,
            attempt,
            submitted_at: env.ledger().timestamp(),
        };

        storage::set_attempt_count(env, &student, assessment_id, attempt);
        storage::set_result(env, &student, assessment_id, attempt, &result);

        events::assessment_submitted(env, assessment_id, &student, attempt, score, passed);

        Ok(result)
    }

    /// Fetch a specific attempt's result, or the latest attempt if `attempt` is `None`.
    pub fn get_assessment_result(
        env: &Env,
        student: Address,
        assessment_id: u64,
        attempt: Option<u32>,
    ) -> Result<AssessmentResult, AssessmentError> {
        let attempt_number = match attempt {
            Some(n) => n,
            None => storage::get_latest_attempt_number(env, &student, assessment_id)
                .ok_or(AssessmentError::ResultNotFound)?,
        };

        storage::get_result(env, &student, assessment_id, attempt_number)
            .ok_or(AssessmentError::ResultNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();

        let admin = Address::generate(&env);
        let student = Address::generate(&env);
        let outsider = Address::generate(&env);

        (env, admin, student, outsider)
    }

    fn default_config() -> AssessmentConfig {
        AssessmentConfig {
            max_score: 100,
            passing_score: 70,
            max_attempts: 2,
        }
    }

    #[test]
    fn admin_can_configure_assessment() {
        let (env, admin, _, _) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();

        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();
    }

    #[test]
    fn non_staff_cannot_configure_assessment() {
        let (env, _, student, _) = setup();
        env.mock_all_auths();

        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(
            AssessmentService::configure_assessment(&env, &student, 1, default_config()),
            Err(AssessmentError::Unauthorized)
        );
    }

    #[test]
    fn enrolled_student_can_submit_and_pass() {
        let (env, admin, student, _) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();
        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();

        let result = AssessmentService::submit_assessment(&env, student.clone(), 1, 80).unwrap();

        assert_eq!(result.student, student);
        assert_eq!(result.score, 80);
        assert_eq!(result.attempt, 1);
        assert!(result.passed);
    }

    #[test]
    fn submission_below_passing_score_is_recorded_as_failed() {
        let (env, admin, student, _) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();
        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();

        let result = AssessmentService::submit_assessment(&env, student, 1, 40).unwrap();

        assert!(!result.passed);
    }

    #[test]
    fn unregistered_caller_cannot_submit() {
        let (env, admin, _, outsider) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();

        assert_eq!(
            AssessmentService::submit_assessment(&env, outsider, 1, 80),
            Err(AssessmentError::NotEnrolled)
        );
    }

    #[test]
    fn submission_to_unconfigured_assessment_fails() {
        let (env, _, student, _) = setup();
        env.mock_all_auths();

        AccessControl::register_student(&env, &student).unwrap();

        assert_eq!(
            AssessmentService::submit_assessment(&env, student, 1, 80),
            Err(AssessmentError::AssessmentNotFound)
        );
    }

    #[test]
    fn score_above_max_is_rejected() {
        let (env, admin, student, _) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();
        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();

        assert_eq!(
            AssessmentService::submit_assessment(&env, student, 1, 150),
            Err(AssessmentError::InvalidScore)
        );
    }

    #[test]
    fn attempt_limit_is_enforced() {
        let (env, admin, student, _) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();
        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();

        AssessmentService::submit_assessment(&env, student.clone(), 1, 40).unwrap();
        AssessmentService::submit_assessment(&env, student.clone(), 1, 50).unwrap();

        assert_eq!(
            AssessmentService::submit_assessment(&env, student, 1, 90),
            Err(AssessmentError::AttemptLimitExceeded)
        );
    }

    #[test]
    fn get_result_defaults_to_latest_attempt() {
        let (env, admin, student, _) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();
        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();

        AssessmentService::submit_assessment(&env, student.clone(), 1, 40).unwrap();
        AssessmentService::submit_assessment(&env, student.clone(), 1, 90).unwrap();

        let latest =
            AssessmentService::get_assessment_result(&env, student, 1, None).unwrap();

        assert_eq!(latest.attempt, 2);
        assert_eq!(latest.score, 90);
    }

    #[test]
    fn get_result_can_target_a_specific_attempt() {
        let (env, admin, student, _) = setup();
        env.mock_all_auths();

        AccessControl::initialize_admin(&env, &admin).unwrap();
        AccessControl::register_student(&env, &student).unwrap();
        AssessmentService::configure_assessment(&env, &admin, 1, default_config()).unwrap();

        AssessmentService::submit_assessment(&env, student.clone(), 1, 40).unwrap();
        AssessmentService::submit_assessment(&env, student.clone(), 1, 90).unwrap();

        let first =
            AssessmentService::get_assessment_result(&env, student, 1, Some(1)).unwrap();

        assert_eq!(first.attempt, 1);
        assert_eq!(first.score, 40);
    }

    #[test]
    fn get_result_fails_when_nothing_submitted() {
        let (env, _, student, _) = setup();

        assert_eq!(
            AssessmentService::get_assessment_result(&env, student, 1, None),
            Err(AssessmentError::ResultNotFound)
        );
    }
}