//! Privacy and safety use cases.
//!
//! Gates reporting, guidance, and UI-facing output so suppressed internal data
//! cannot be returned as public output.

use super::domain::{
    InternalWorkforceDataset, PublicProfileGroup, PublicReport, SuppressionSummary,
    MINIMUM_SAFE_AGGREGATE_GROUP_SIZE, PERSONAL_HEALTH_DETAIL_SUPPRESSION_CODE,
    SMALL_GROUP_SUPPRESSION_CODE,
};
use std::collections::BTreeMap;

#[derive(Default)]
struct GroupAccumulator {
    employee_count: usize,
    attendance_days_observed: u32,
}

pub fn filter_public_report(dataset: InternalWorkforceDataset) -> PublicReport {
    let mut groups = BTreeMap::<String, GroupAccumulator>::new();
    let mut affected_record_count = 0;
    let mut suppressed_field_count = 0;

    for employee in dataset.employees {
        let personal_health_detail_count = employee.personal_health_detail_count();
        if personal_health_detail_count > 0 {
            affected_record_count += 1;
            suppressed_field_count += personal_health_detail_count;
        }

        let group = groups.entry(employee.group_key).or_default();
        group.employee_count += 1;
        group.attendance_days_observed += employee.attendance_days_observed;
    }

    let profiles = groups
        .iter()
        .filter(|(_, group)| group.employee_count >= MINIMUM_SAFE_AGGREGATE_GROUP_SIZE)
        .map(|(group_key, group)| {
            let group_key = group_key.clone();
            PublicProfileGroup {
                profile_id: format!("group:{group_key}"),
                group_key,
                employee_count: group.employee_count,
                attendance_days_observed: group.attendance_days_observed,
                health_detail_status: "suppressed".to_string(),
            }
        })
        .collect();

    let mut suppression_summary = Vec::new();
    if suppressed_field_count > 0 {
        suppression_summary.push(SuppressionSummary {
            suppression_code: PERSONAL_HEALTH_DETAIL_SUPPRESSION_CODE.to_string(),
            category: "personal_health_detail".to_string(),
            reason: "個人の健康関連詳細は公開レポートの対象外である。".to_string(),
            affected_record_count,
            suppressed_field_count,
        });
    }
    for group in groups.values() {
        if group.employee_count < MINIMUM_SAFE_AGGREGATE_GROUP_SIZE {
            suppression_summary.push(SuppressionSummary {
                suppression_code: SMALL_GROUP_SUPPRESSION_CODE.to_string(),
                category: "small_group".to_string(),
                reason: format!(
                    "有効データ数が {} 未満の集団は公開レポートの対象外である。",
                    MINIMUM_SAFE_AGGREGATE_GROUP_SIZE
                ),
                affected_record_count: group.employee_count,
                suppressed_field_count: 0,
            });
        }
    }

    PublicReport {
        run_id: dataset.run_id,
        input_traces: dataset.input_traces,
        policy_trace: dataset.policy.into(),
        profiles,
        suppression_summary,
    }
}

#[cfg(test)]
mod tests {
    use crate::contexts::privacy_safety::application::filter_public_report;
    use crate::contexts::privacy_safety::domain::{
        InputTrace, InternalEmployeeProfile, InternalWorkforceDataset, PrivacyPolicy,
        SMALL_GROUP_SUPPRESSION_CODE,
    };
    use crate::shared::RunId;

    fn dataset_with_employees(employees: Vec<InternalEmployeeProfile>) -> InternalWorkforceDataset {
        InternalWorkforceDataset {
            run_id: RunId::new("run-small-group-001"),
            input_traces: vec![InputTrace {
                dataset_id: "attendance_by_employee".to_string(),
                source_ref: "fixtures/privacy/attendance.csv".to_string(),
                fingerprint: "sha256:privacy".to_string(),
                record_count: employees.len(),
            }],
            policy: PrivacyPolicy {
                policy_id: "privacy-safety-v1".to_string(),
                version: "2026-06-03".to_string(),
            },
            employees,
        }
    }

    fn employee(employee_ref: &str, group_key: &str) -> InternalEmployeeProfile {
        InternalEmployeeProfile {
            employee_ref: employee_ref.to_string(),
            group_key: group_key.to_string(),
            attendance_days_observed: 20,
            fatigue_value: None,
            sleep_duration_hours: None,
            fatigue_comment: None,
        }
    }

    #[test]
    fn suppresses_unsafe_small_group_before_public_report() {
        let public_report =
            filter_public_report(dataset_with_employees(vec![employee("E001", "operations")]));

        assert!(public_report.profiles.is_empty());
        assert!(public_report.suppression_summary.iter().any(|summary| {
            summary.suppression_code == SMALL_GROUP_SUPPRESSION_CODE
                && summary.category == "small_group"
                && summary.affected_record_count == 1
        }));
    }

    #[test]
    fn keeps_safe_aggregate_group_public() {
        let employees = (0..10)
            .map(|index| employee(&format!("E{index:03}"), "operations"))
            .collect();
        let public_report = filter_public_report(dataset_with_employees(employees));

        assert_eq!(public_report.profiles.len(), 1);
        assert_eq!(public_report.profiles[0].employee_count, 10);
        assert!(!public_report
            .suppression_summary
            .iter()
            .any(|summary| summary.suppression_code == SMALL_GROUP_SUPPRESSION_CODE));
    }

    #[test]
    fn property_group_visibility_matches_minimum_safe_group_size() {
        for group_size in 0..=20 {
            let employees = (0..group_size)
                .map(|index| employee(&format!("E{index:03}"), "operations"))
                .collect();
            let public_report = filter_public_report(dataset_with_employees(employees));

            let should_be_public = group_size
                >= crate::contexts::privacy_safety::domain::MINIMUM_SAFE_AGGREGATE_GROUP_SIZE;
            assert_eq!(!public_report.profiles.is_empty(), should_be_public);
            assert_eq!(
                public_report
                    .suppression_summary
                    .iter()
                    .any(|summary| summary.suppression_code == SMALL_GROUP_SUPPRESSION_CODE),
                group_size > 0 && !should_be_public
            );
        }
    }
}
