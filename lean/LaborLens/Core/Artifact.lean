import LaborLens.Core.Types

namespace LaborLens

structure InputRef where
  inputId : String
  fileHashSha256 : String
  schemaVersion : String
deriving DecidableEq, Repr

structure NormalizedRef where
  normalizedDatasetId : String
  normalizationRuleVersion : String
  columnMappingVersion : String
deriving DecidableEq, Repr

structure PolicyRef where
  suppressionPolicyVersion : String
  inferenceThresholdK : Nat
  ragIndexVersion : String
  accessPolicyVersion : String
deriving DecidableEq, Repr

structure OutputRef where
  artifactId : String
  outputHashSha256 : String
deriving DecidableEq, Repr

structure AuditRef where
  actorId : String
  actorRole : String
  executionReason : String
  accessLogRef : String
deriving DecidableEq, Repr

structure RunArtifact where
  runId : RunId
  tenantId : TenantId
  inputRef : InputRef
  normalizedRef : NormalizedRef
  policyRef : PolicyRef
  outputRef : OutputRef
  auditRef : AuditRef
deriving Repr

def hasRunArtifactRefs (artifact : RunArtifact) : Prop :=
  artifact.inputRef.inputId ≠ "" ∧
  artifact.inputRef.fileHashSha256 ≠ "" ∧
  artifact.normalizedRef.normalizedDatasetId ≠ "" ∧
  artifact.policyRef.suppressionPolicyVersion ≠ "" ∧
  artifact.outputRef.artifactId ≠ "" ∧
  artifact.auditRef.accessLogRef ≠ ""

def traceableRunArtifact (artifact : RunArtifact) : Prop :=
  hasRunArtifactRefs artifact

end LaborLens
