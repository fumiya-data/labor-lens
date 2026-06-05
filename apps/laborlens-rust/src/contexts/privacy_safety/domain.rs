//! Privacy and safety domain terms.
//!
//! Owns suppression reasons, public-output safety status, small-group rules,
//! and the implementation contract corresponding to the Lean privacy spec.

use crate::shared::RunId;
use serde::{Deserialize, Serialize};

pub const PERSONAL_HEALTH_DETAIL_SUPPRESSION_CODE: &str = "PERSONAL_HEALTH_DETAIL_SUPPRESSED";
pub const SMALL_GROUP_SUPPRESSION_CODE: &str = "SMALL_GROUP_SUPPRESSED";
pub const MINIMUM_SAFE_AGGREGATE_GROUP_SIZE: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputTrace {
    pub dataset_id: String,
    pub source_ref: String,
    pub fingerprint: String,
    pub record_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyPolicy {
    pub policy_id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalEmployeeProfile {
    pub employee_ref: String,
    pub group_key: String,
    pub attendance_days_observed: u32,
    pub fatigue_value: Option<u8>,
    pub sleep_duration_hours: Option<f32>,
    pub fatigue_comment: Option<String>,
}

impl InternalEmployeeProfile {
    pub fn personal_health_detail_count(&self) -> usize {
        self.fatigue_value.is_some() as usize
            + self.sleep_duration_hours.is_some() as usize
            + self
                .fatigue_comment
                .as_ref()
                .is_some_and(|value| !value.is_empty()) as usize
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalWorkforceDataset {
    pub run_id: RunId,
    pub input_traces: Vec<InputTrace>,
    pub policy: PrivacyPolicy,
    pub employees: Vec<InternalEmployeeProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyTrace {
    pub policy_id: String,
    pub version: String,
    pub safety_boundary: String,
}

impl From<PrivacyPolicy> for PolicyTrace {
    fn from(policy: PrivacyPolicy) -> Self {
        Self {
            policy_id: policy.policy_id,
            version: policy.version,
            safety_boundary: "公開 artifact 生成前に個人の健康関連詳細と少人数集団を抑制する"
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicProfileGroup {
    pub profile_id: String,
    pub group_key: String,
    pub employee_count: usize,
    pub attendance_days_observed: u32,
    pub health_detail_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuppressionSummary {
    pub suppression_code: String,
    pub category: String,
    pub reason: String,
    pub affected_record_count: usize,
    pub suppressed_field_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicReport {
    pub run_id: RunId,
    pub input_traces: Vec<InputTrace>,
    pub policy_trace: PolicyTrace,
    pub profiles: Vec<PublicProfileGroup>,
    pub suppression_summary: Vec<SuppressionSummary>,
}
