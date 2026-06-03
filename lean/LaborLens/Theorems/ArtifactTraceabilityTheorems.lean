import LaborLens.Core.Artifact

namespace LaborLens

theorem attachTraceRefsToRunArtifact
  (artifact : RunArtifact)
  (inputIdPresent : artifact.inputRef.inputId ≠ "")
  (inputHashPresent : artifact.inputRef.fileHashSha256 ≠ "")
  (normalizedDatasetPresent : artifact.normalizedRef.normalizedDatasetId ≠ "")
  (suppressionPolicyPresent : artifact.policyRef.suppressionPolicyVersion ≠ "")
  (outputArtifactPresent : artifact.outputRef.artifactId ≠ "")
  (auditLogPresent : artifact.auditRef.accessLogRef ≠ "") :
  traceableRunArtifact artifact := by
  unfold traceableRunArtifact
  unfold hasRunArtifactRefs
  exact ⟨
    inputIdPresent,
    inputHashPresent,
    normalizedDatasetPresent,
    suppressionPolicyPresent,
    outputArtifactPresent,
    auditLogPresent
  ⟩

end LaborLens
