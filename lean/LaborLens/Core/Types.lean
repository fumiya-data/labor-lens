namespace LaborLens

structure EmployeeId where
  value : String
deriving DecidableEq, Repr

structure DepartmentId where
  value : String
deriving DecidableEq, Repr

structure RunId where
  value : String
deriving DecidableEq, Repr

structure TenantId where
  value : String
deriving DecidableEq, Repr

structure FatigueValue where
  score : Nat
deriving DecidableEq, Repr

structure SleepDuration where
  minutes : Nat
deriving DecidableEq, Repr

structure FatigueComment where
  value : String
deriving DecidableEq, Repr

inductive SensitiveKind where
  | fatigueValue
  | sleepDuration
  | fatigueComment
deriving DecidableEq, Repr

inductive SensitiveField where
  | fatigueValue : FatigueValue -> SensitiveField
  | sleepDuration : SleepDuration -> SensitiveField
  | fatigueComment : FatigueComment -> SensitiveField
deriving DecidableEq, Repr

inductive ReadinessStatus where
  | ready
  | partialReady
  | blockedStatus
deriving DecidableEq, Repr

inductive IssueCategory where
  | schemaIssue
  | dataQualityIssue
  | masterIssue
  | grainIssue
  | joinIssue
  | privacyIssue
  | processingIssue
deriving DecidableEq, Repr

inductive IssueSeverity where
  | critical
  | high
  | medium
  | low
deriving DecidableEq, Repr

end LaborLens
