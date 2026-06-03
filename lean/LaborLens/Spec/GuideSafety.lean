import LaborLens.Core.Dataset

namespace LaborLens

inductive GuideContext where
  | publicReport : PublicReport -> GuideContext
  | ruleExplanation : String -> GuideContext
deriving Repr

def guideContextContainsRawDataset
  (_context : GuideContext)
  (_dataset : InternalDataset) : Prop :=
  False

end LaborLens
