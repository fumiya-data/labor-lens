//! Workforce analysis domain terms.
//!
//! Owns readiness, joinability, labor-time signals, labor-cost links, and
//! aggregate analysis concepts. It must not make legal, medical, or personnel
//! evaluation decisions.

use crate::contexts::ingest::domain::{AttendanceRecord, EmployeeRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    Ready,
    Partial,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaborCostGrain {
    EmployeeMonthly,
    DepartmentMonthly,
    EmploymentTypeMonthly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaborCostRecord {
    pub month: String,
    pub grain: LaborCostGrain,
    pub employee_id: Option<String>,
    pub department: Option<String>,
    pub amount_yen: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkforceIssueCategory {
    Master,
    Joinability,
    Grain,
    Processing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkforceIssue {
    pub issue_id: String,
    pub issue_code: String,
    pub category: WorkforceIssueCategory,
    pub severity: String,
    pub employee_id: Option<String>,
    pub dataset_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessCheckKind {
    Readiness,
    Joinability,
    MasterDataIssue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessCheckStatus {
    Passed,
    Warning,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessCheck {
    pub check_id: String,
    pub kind: BusinessCheckKind,
    pub status: BusinessCheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkforceAnalysisInput {
    pub employees: Vec<EmployeeRecord>,
    pub attendance: Vec<AttendanceRecord>,
    pub labor_costs: Vec<LaborCostRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkforceAnalysisResult {
    pub readiness_status: ReadinessStatus,
    pub joinable_employee_attendance: bool,
    pub joinable_labor_cost_attendance: bool,
    pub issues: Vec<WorkforceIssue>,
    pub business_checks: Vec<BusinessCheck>,
}
