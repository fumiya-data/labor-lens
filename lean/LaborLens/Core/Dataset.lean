import LaborLens.Core.Types

namespace LaborLens

structure InternalRecord where
  employeeId : EmployeeId
  departmentId : DepartmentId
  sensitiveFields : List SensitiveField
deriving Repr

structure InternalDataset where
  records : List InternalRecord
deriving Repr

inductive PublicField where
  | aggregateText : String -> PublicField
  | suppressionNotice : String -> PublicField
  | sourceNote : String -> PublicField
  | ruleExplanation : String -> String -> PublicField
deriving DecidableEq, Repr

structure PublicReport where
  fields : List PublicField
deriving Repr

end LaborLens
