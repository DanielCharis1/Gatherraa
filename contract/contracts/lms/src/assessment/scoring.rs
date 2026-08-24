use super::errors::AssessmentError;
use super::types::AssessmentConfig;

/// Validates a submitted score against the assessment's configured max,
/// returning whether it meets the passing threshold.
///
/// Pure function: no storage access, easy to unit test in isolation.
pub fn validate_score(score: u32, config: &AssessmentConfig) -> Result<bool, AssessmentError> {
    if score > config.max_score {
        return Err(AssessmentError::InvalidScore);
    }
    Ok(score >= config.passing_score)
}

/// Checks the student has attempts remaining and returns the attempt
/// number to record if so.
pub fn check_attempt_limit(
    current_attempts: u32,
    max_attempts: u32,
) -> Result<u32, AssessmentError> {
    if current_attempts >= max_attempts {
        return Err(AssessmentError::AttemptLimitExceeded);
    }
    Ok(current_attempts + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AssessmentConfig {
        AssessmentConfig {
            max_score: 100,
            passing_score: 70,
            max_attempts: 2,
        }
    }

    #[test]
    fn score_at_or_above_passing_score_passes() {
        assert_eq!(validate_score(70, &config()), Ok(true));
        assert_eq!(validate_score(100, &config()), Ok(true));
    }

    #[test]
    fn score_below_passing_score_fails() {
        assert_eq!(validate_score(69, &config()), Ok(false));
    }

    #[test]
    fn score_above_max_is_rejected() {
        assert_eq!(
            validate_score(101, &config()),
            Err(AssessmentError::InvalidScore)
        );
    }

    #[test]
    fn attempt_within_limit_is_allowed() {
        assert_eq!(check_attempt_limit(0, 2), Ok(1));
        assert_eq!(check_attempt_limit(1, 2), Ok(2));
    }

    #[test]
    fn attempt_at_limit_is_rejected() {
        assert_eq!(
            check_attempt_limit(2, 2),
            Err(AssessmentError::AttemptLimitExceeded)
        );
    }
}