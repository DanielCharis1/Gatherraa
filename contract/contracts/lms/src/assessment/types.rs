use soroban_sdk::{contracttype, Address};

/// Result of a single assessment attempt, persisted in contract storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssessmentResult {
    pub student: Address,
    pub assessment_id: u64,
    pub score: u32,
    pub passed: bool,
    pub attempt: u32,
    pub submitted_at: u64,
}

/// Configuration for an assessment, set by staff before students can submit.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssessmentConfig {
    pub max_score: u32,
    pub passing_score: u32,
    pub max_attempts: u32,
}