//! Privacy and safety use cases.
//!
//! Gates reporting, guidance, and UI-facing output so suppressed internal data
//! cannot be returned as public output.

use super::domain::{
    InternalWorkforceDataset, PublicProfileGroup, PublicReport, SuppressionSummary,
    PERSONAL_HEALTH_DETAIL_SUPPRESSION_CODE,
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
        .into_iter()
        .map(|(group_key, group)| PublicProfileGroup {
            profile_id: format!("group:{group_key}"),
            group_key,
            employee_count: group.employee_count,
            attendance_days_observed: group.attendance_days_observed,
            health_detail_status: "suppressed".to_string(),
        })
        .collect();

    let suppression_summary = if suppressed_field_count == 0 {
        Vec::new()
    } else {
        vec![SuppressionSummary {
            suppression_code: PERSONAL_HEALTH_DETAIL_SUPPRESSION_CODE.to_string(),
            category: "personal_health_detail".to_string(),
            reason: "個人の健康関連詳細は公開レポートの対象外である。".to_string(),
            affected_record_count,
            suppressed_field_count,
        }]
    };

    PublicReport {
        run_id: dataset.run_id,
        input_traces: dataset.input_traces,
        policy_trace: dataset.policy.into(),
        profiles,
        suppression_summary,
    }
}
