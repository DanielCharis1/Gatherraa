use soroban_sdk::contracterror;

/// Errors produced by the LMS assessment module.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AssessmentError {
    /// Caller is not a registered student (and so cannot submit).
    NotEnrolled = 1,

    /// No configuration exists yet for this assessment_id.
    AssessmentNotFound = 2,

    /// Student has already used all allowed attempts.
    AttemptLimitExceeded = 3,

    /// Submitted score exceeds the assessment's max_score.
    InvalidScore = 4,

    /// No result exists yet for this student/assessment(/attempt).
    ResultNotFound = 5,

    /// Caller lacks the staff privileges required to configure an assessment.
    Unauthorized = 6,
}