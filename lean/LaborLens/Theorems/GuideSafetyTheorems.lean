import LaborLens.Spec.GuideSafety

namespace LaborLens

theorem guideContextDoesNotExposeRawDataset
  (context : GuideContext)
  (dataset : InternalDataset) :
  ¬ guideContextContainsRawDataset context dataset := by
  unfold guideContextContainsRawDataset
  intro h
  exact h

end LaborLens
