//! Workforce analysis use cases.
//!
//! Coordinates readiness assessment, aggregate calculations, and handoff to
//! privacy/safety before public reporting.

use super::domain::{
    BusinessCheck, BusinessCheckKind, BusinessCheckStatus, LaborCostGrain, ReadinessStatus,
    WorkforceAnalysisInput, WorkforceAnalysisResult, WorkforceIssue, WorkforceIssueCategory,
};
use crate::contexts::ingest::domain::EmployeeRecord;
use std::collections::{BTreeMap, BTreeSet};

pub fn analyze_workforce(input: WorkforceAnalysisInput) -> WorkforceAnalysisResult {
    let employees_by_id: BTreeMap<String, EmployeeRecord> = input
        .employees
        .iter()
        .map(|employee| (employee.employee_id.clone(), employee.clone()))
        .collect();
    let attendance_employee_ids: BTreeSet<String> = input
        .attendance
        .iter()
        .map(|attendance| attendance.employee_id.clone())
        .collect();
    let mut issues = Vec::new();

    if input.employees.is_empty() || input.attendance.is_empty() {
        issues.push(issue(
            "processing:missing_core_dataset",
            "missing_core_dataset",
            WorkforceIssueCategory::Processing,
            "high",
            None,
            None,
            "employees と attendance は readiness 判定に必須である。",
        ));
    }

    for employee_id in &attendance_employee_ids {
        match employees_by_id.get(employee_id) {
            Some(employee) if is_retired(employee) => issues.push(issue(
                &format!("master:retired_employee_has_attendance:{employee_id}"),
                "retired_employee_has_attendance",
                WorkforceIssueCategory::Master,
                "medium",
                Some(employee_id.clone()),
                Some("attendance_by_employee".to_string()),
                "退職扱いの従業員に attendance record がある。",
            )),
            Some(_) => {}
            None => issues.push(issue(
                &format!("master:missing_employee:{employee_id}"),
                "missing_employee",
                WorkforceIssueCategory::Master,
                "high",
                Some(employee_id.clone()),
                Some("attendance_by_employee".to_string()),
                "attendance record の employee_id が employees master に存在しない。",
            )),
        }
    }

    let mut labor_cost_joinable = true;
    for (index, labor_cost) in input.labor_costs.iter().enumerate() {
        if labor_cost.grain != LaborCostGrain::EmployeeMonthly {
            labor_cost_joinable = false;
            issues.push(issue(
                &format!("grain:labor_cost_attendance:{index}"),
                "grain_mismatch",
                WorkforceIssueCategory::Grain,
                "medium",
                labor_cost.employee_id.clone(),
                Some("labor_cost".to_string()),
                "employee monthly 以外の labor-cost data は個人別 attendance と直接 join できない。",
            ));
        }

        let Some(employee_id) = labor_cost.employee_id.as_ref() else {
            labor_cost_joinable = false;
            issues.push(issue(
                &format!("joinability:labor_cost_missing_employee_id:{index}"),
                "labor_cost_missing_employee_id",
                WorkforceIssueCategory::Joinability,
                "high",
                None,
                Some("labor_cost".to_string()),
                "employee_id を持たない labor-cost data は個人別 attendance と join できない。",
            ));
            continue;
        };

        match employees_by_id.get(employee_id) {
            Some(employee) => {
                if let Some(labor_department) = labor_cost.department.as_deref() {
                    if labor_department != employee.department {
                        labor_cost_joinable = false;
                        issues.push(issue(
                            &format!("master:department_mismatch:{employee_id}:{index}"),
                            "department_mismatch",
                            WorkforceIssueCategory::Master,
                            "medium",
                            Some(employee_id.clone()),
                            Some("labor_cost".to_string()),
                            "labor-cost department が employees master の department と一致しない。",
                        ));
                    }
                }
            }
            None => {
                labor_cost_joinable = false;
                issues.push(issue(
                    &format!("master:labor_cost_missing_employee:{employee_id}:{index}"),
                    "missing_employee",
                    WorkforceIssueCategory::Master,
                    "high",
                    Some(employee_id.clone()),
                    Some("labor_cost".to_string()),
                    "labor-cost record の employee_id が employees master に存在しない。",
                ));
            }
        }
    }

    let employee_attendance_joinable = !input.employees.is_empty()
        && !input.attendance.is_empty()
        && !issues
            .iter()
            .any(|issue| issue.issue_code == "missing_employee");
    let readiness_status = readiness_status(&input, &issues);

    WorkforceAnalysisResult {
        readiness_status,
        joinable_employee_attendance: employee_attendance_joinable,
        joinable_labor_cost_attendance: labor_cost_joinable,
        business_checks: business_checks(
            readiness_status,
            employee_attendance_joinable,
            labor_cost_joinable,
        ),
        issues,
    }
}

fn readiness_status(
    input: &WorkforceAnalysisInput,
    issues: &[WorkforceIssue],
) -> ReadinessStatus {
    if input.employees.is_empty() || input.attendance.is_empty() {
        ReadinessStatus::Blocked
    } else if issues.is_empty() {
        ReadinessStatus::Ready
    } else {
        ReadinessStatus::Partial
    }
}

fn business_checks(
    readiness_status: ReadinessStatus,
    joinable_employee_attendance: bool,
    joinable_labor_cost_attendance: bool,
) -> Vec<BusinessCheck> {
    vec![
        BusinessCheck {
            check_id: "business_check:readiness".to_string(),
            kind: BusinessCheckKind::Readiness,
            status: match readiness_status {
                ReadinessStatus::Ready => BusinessCheckStatus::Passed,
                ReadinessStatus::Partial => BusinessCheckStatus::Warning,
                ReadinessStatus::Blocked => BusinessCheckStatus::Failed,
            },
            message: "readiness は issue と別に業務確認結果として扱う。".to_string(),
        },
        BusinessCheck {
            check_id: "business_check:joinability".to_string(),
            kind: BusinessCheckKind::Joinability,
            status: if joinable_employee_attendance && joinable_labor_cost_attendance {
                BusinessCheckStatus::Passed
            } else {
                BusinessCheckStatus::Warning
            },
            message: "joinability は grain と employee_id の有無から判定する。".to_string(),
        },
    ]
}

fn is_retired(employee: &EmployeeRecord) -> bool {
    employee
        .employment_status
        .as_deref()
        .map(|status| matches!(status, "退職" | "retired" | "inactive"))
        .unwrap_or(false)
}

fn issue(
    issue_id: &str,
    issue_code: &str,
    category: WorkforceIssueCategory,
    severity: &str,
    employee_id: Option<String>,
    dataset_id: Option<String>,
    message: &str,
) -> WorkforceIssue {
    WorkforceIssue {
        issue_id: issue_id.to_string(),
        issue_code: issue_code.to_string(),
        category,
        severity: severity.to_string(),
        employee_id,
        dataset_id,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::contexts::ingest::domain::{AttendanceRecord, EmployeeRecord};
    use crate::contexts::workforce_analysis::application::analyze_workforce;
    use crate::contexts::workforce_analysis::domain::{
        BusinessCheckKind, LaborCostGrain, LaborCostRecord, ReadinessStatus,
        WorkforceAnalysisInput, WorkforceIssueCategory,
    };

    fn employee(employee_id: &str, department: &str, status: &str) -> EmployeeRecord {
        EmployeeRecord {
            employee_id: employee_id.to_string(),
            employee_name: format!("{employee_id} name"),
            department: department.to_string(),
            hire_date: Some("2024-04-01".to_string()),
            employment_status: Some(status.to_string()),
        }
    }

    fn attendance(employee_id: &str) -> AttendanceRecord {
        AttendanceRecord {
            employee_id: employee_id.to_string(),
            work_date: "2026-01-05".to_string(),
            clock_in: "09:00".to_string(),
            clock_out: "18:00".to_string(),
            break_minutes: Some(60),
        }
    }

    #[test]
    fn ready_when_employee_attendance_and_labor_cost_are_joinable() {
        let result = analyze_workforce(WorkforceAnalysisInput {
            employees: vec![employee("E001", "operations", "在籍")],
            attendance: vec![attendance("E001")],
            labor_costs: vec![LaborCostRecord {
                month: "2026-01".to_string(),
                grain: LaborCostGrain::EmployeeMonthly,
                employee_id: Some("E001".to_string()),
                department: Some("operations".to_string()),
                amount_yen: 300000,
            }],
        });

        assert_eq!(result.readiness_status, ReadinessStatus::Ready);
        assert!(result.issues.is_empty());
        assert!(result.joinable_employee_attendance);
        assert!(result.joinable_labor_cost_attendance);
        assert!(result
            .business_checks
            .iter()
            .any(|check| check.kind == BusinessCheckKind::Joinability));
    }

    #[test]
    fn missing_employee_and_retired_employee_are_master_issues_not_business_checks() {
        let result = analyze_workforce(WorkforceAnalysisInput {
            employees: vec![employee("E001", "operations", "退職")],
            attendance: vec![attendance("E001"), attendance("E999")],
            labor_costs: Vec::new(),
        });

        assert_eq!(result.readiness_status, ReadinessStatus::Partial);
        assert!(result.issues.iter().any(|issue| {
            issue.category == WorkforceIssueCategory::Master
                && issue.issue_code == "missing_employee"
                && issue.employee_id.as_deref() == Some("E999")
        }));
        assert!(result.issues.iter().any(|issue| {
            issue.category == WorkforceIssueCategory::Master
                && issue.issue_code == "retired_employee_has_attendance"
                && issue.employee_id.as_deref() == Some("E001")
        }));
        assert!(result
            .business_checks
            .iter()
            .all(|check| check.kind != BusinessCheckKind::MasterDataIssue));
    }

    #[test]
    fn labor_cost_without_employee_id_cannot_join_personal_attendance() {
        let result = analyze_workforce(WorkforceAnalysisInput {
            employees: vec![employee("E001", "operations", "在籍")],
            attendance: vec![attendance("E001")],
            labor_costs: vec![LaborCostRecord {
                month: "2026-01".to_string(),
                grain: LaborCostGrain::DepartmentMonthly,
                employee_id: None,
                department: Some("operations".to_string()),
                amount_yen: 300000,
            }],
        });

        assert_eq!(result.readiness_status, ReadinessStatus::Partial);
        assert!(!result.joinable_labor_cost_attendance);
        assert!(result.issues.iter().any(|issue| {
            issue.category == WorkforceIssueCategory::Joinability
                && issue.issue_code == "labor_cost_missing_employee_id"
        }));
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.category == WorkforceIssueCategory::Grain));
    }

    #[test]
    fn empty_core_datasets_block_readiness() {
        let result = analyze_workforce(WorkforceAnalysisInput {
            employees: Vec::new(),
            attendance: Vec::new(),
            labor_costs: Vec::new(),
        });

        assert_eq!(result.readiness_status, ReadinessStatus::Blocked);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.issue_code == "missing_core_dataset"));
    }
}
