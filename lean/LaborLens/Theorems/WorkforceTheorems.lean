import LaborLens.Spec.Workforce

namespace LaborLens

theorem laborCostWithoutEmployeeIdCannotJoinPersonalAttendance
  (cost : LaborCostInput)
  (attendance : AttendanceInput)
  (missingEmployeeId : cost.employeeId = none) :
  ¬ canJoinLaborCostToPersonalAttendance cost attendance := by
  unfold canJoinLaborCostToPersonalAttendance
  intro canJoin
  exact canJoin missingEmployeeId

theorem missingEmployeeCreatesMasterIssue
  (master : EmployeeMaster)
  (attendance : AttendanceInput)
  (_missing : missingEmployee master attendance) :
  (masterIssueForMissingEmployee attendance).category = IssueCategory.masterIssue := by
  unfold masterIssueForMissingEmployee
  rfl

theorem issueRecordIsSeparatedFromPublicReportField
  (issue : IssueRecord)
  (field : PublicField) :
  ¬ issueRecordAppearsAsPublicField issue field := by
  unfold issueRecordAppearsAsPublicField
  intro h
  exact h

end LaborLens
