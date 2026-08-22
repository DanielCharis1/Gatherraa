use soroban_sdk::{Address, Env};

use crate::StorageKey;

use super::types::{AssessmentConfig, AssessmentResult};

pub fn get_config(env: &Env, assessment_id: u64) -> Option<AssessmentConfig> {
    env.storage()
        .persistent()
        .get(&StorageKey::AssessmentConfig(assessment_id))
}

pub fn set_config(env: &Env, assessment_id: u64, config: &AssessmentConfig) {
    env.storage()
        .persistent()
        .set(&StorageKey::AssessmentConfig(assessment_id), config);
}

pub fn get_attempt_count(env: &Env, student: &Address, assessment_id: u64) -> u32 {
    env.storage()
        .persistent()
        .get(&StorageKey::AssessmentAttemptCount(
            student.clone(),
            assessment_id,
        ))
        .unwrap_or(0)
}

pub fn set_attempt_count(env: &Env, student: &Address, assessment_id: u64, count: u32) {
    env.storage().persistent().set(
        &StorageKey::AssessmentAttemptCount(student.clone(), assessment_id),
        &count,
    );
}

pub fn set_result(
    env: &Env,
    student: &Address,
    assessment_id: u64,
    attempt: u32,
    result: &AssessmentResult,
) {
    env.storage().persistent().set(
        &StorageKey::AssessmentResult(student.clone(), assessment_id, attempt),
        result,
    );
    env.storage().persistent().set(
        &StorageKey::AssessmentLatestAttempt(student.clone(), assessment_id),
        &attempt,
    );
}

pub fn get_result(
    env: &Env,
    student: &Address,
    assessment_id: u64,
    attempt: u32,
) -> Option<AssessmentResult> {
    env.storage().persistent().get(&StorageKey::AssessmentResult(
        student.clone(),
        assessment_id,
        attempt,
    ))
}

pub fn get_latest_attempt_number(env: &Env, student: &Address, assessment_id: u64) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&StorageKey::AssessmentLatestAttempt(
            student.clone(),
            assessment_id,
        ))
}