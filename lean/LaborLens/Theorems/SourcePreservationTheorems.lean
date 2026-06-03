import LaborLens.Spec.SourcePreservation

namespace LaborLens

theorem preserveInputHashAfterSourceSave
  (source : SourceInput) :
  sourceSavePreservesInputHash source (saveSource source) := by
  unfold sourceSavePreservesInputHash
  constructor
  · unfold inputHashUnchanged
    unfold saveSource
    rfl
  · unfold savedSourceRefKeepsOriginalHash
    unfold saveSource
    rfl

theorem preserveInputHashInRunArtifactInputRef
  (source : SourceInput)
  (result : SourceSaveResult)
  (artifact : RunArtifact)
  (usesSavedInputRef : artifact.inputRef = result.inputRef)
  (keepsHash : savedSourceRefKeepsOriginalHash source result) :
  runArtifactInputRefKeepsOriginalHash source artifact := by
  unfold runArtifactInputRefKeepsOriginalHash
  unfold savedSourceRefKeepsOriginalHash at keepsHash
  rw [usesSavedInputRef]
  exact keepsHash

end LaborLens
