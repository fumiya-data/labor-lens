import LaborLens.Core.Dataset

namespace LaborLens

structure LaborCostInput where
  employeeId : Option EmployeeId
  month : String
deriving DecidableEq, Repr

structure AttendanceInput where
  employeeId : EmployeeId
  workDate : String
deriving DecidableEq, Repr

structure EmployeeMaster where
  employees : List EmployeeId
deriving Repr

structure IssueRecord where
  category : IssueCategory
  code : String
  employeeId : Option EmployeeId
deriving DecidableEq, Repr

def canJoinLaborCostToPersonalAttendance
  (cost : LaborCostInput)
  (_attendance : AttendanceInput) : Prop :=
  cost.employeeId ≠ none

def missingEmployee
  (master : EmployeeMaster)
  (attendance : AttendanceInput) : Prop :=
  attendance.employeeId ∉ master.employees

def masterIssueForMissingEmployee
  (attendance : AttendanceInput) : IssueRecord :=
  {
    category := IssueCategory.masterIssue
    code := "missing_employee"
    employeeId := some attendance.employeeId
  }

def issueRecordAppearsAsPublicField
  (_issue : IssueRecord)
  (_field : PublicField) : Prop :=
  False

end LaborLens
