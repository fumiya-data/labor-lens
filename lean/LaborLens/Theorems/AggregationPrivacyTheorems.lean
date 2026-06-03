import LaborLens.Spec.AggregationPrivacy

namespace LaborLens

theorem suppressUnsafeAggregateGroup
  (group : AggregateGroup)
  (summary : String)
  (unsafeGroup : safeAggregateGroup group = false) :
  isSuppressedAggregateGroup group (aggregateGroup group summary) ∧
  ¬ displayablePublicAggregationResult (aggregateGroup group summary) := by
  constructor
  · unfold isSuppressedAggregateGroup
    exists SuppressionReason.smallGroup
    unfold aggregateGroup
    simp [unsafeGroup]
  · unfold aggregateGroup
    simp [unsafeGroup, displayablePublicAggregationResult]

end LaborLens
