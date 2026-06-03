import LaborLens.Core.Dataset

namespace LaborLens

def minimumSafeAggregateGroupSize : Nat := 10

structure AggregateGroup where
  departmentId : DepartmentId
  memberCount : Nat
deriving DecidableEq, Repr

inductive SuppressionReason where
  | smallGroup
  | reidentificationRisk
deriving DecidableEq, Repr

inductive AggregationResult where
  | publicResult : AggregateGroup -> String -> AggregationResult
  | suppressedResult : AggregateGroup -> SuppressionReason -> AggregationResult
deriving Repr

def safeAggregateGroup (group : AggregateGroup) : Bool :=
  decide (minimumSafeAggregateGroupSize ≤ group.memberCount)

def aggregateGroup (group : AggregateGroup) (summary : String) : AggregationResult :=
  if safeAggregateGroup group then
    AggregationResult.publicResult group summary
  else
    AggregationResult.suppressedResult group SuppressionReason.smallGroup

def isSuppressedAggregateGroup
  (group : AggregateGroup)
  (result : AggregationResult) : Prop :=
  ∃ reason, result = AggregationResult.suppressedResult group reason

def displayablePublicAggregationResult (result : AggregationResult) : Prop :=
  match result with
  | AggregationResult.publicResult group _ => safeAggregateGroup group = true
  | AggregationResult.suppressedResult _ _ => False

end LaborLens
