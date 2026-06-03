import LaborLens.Core.Artifact

namespace LaborLens

structure SourceInput where
  inputId : String
  fileHashSha256 : String
  schemaVersion : String
deriving DecidableEq, Repr

structure SourceSaveResult where
  savedSource : SourceInput
  inputRef : InputRef
deriving DecidableEq, Repr

def saveSource (source : SourceInput) : SourceSaveResult :=
  {
    savedSource := source
    inputRef := {
      inputId := source.inputId
      fileHashSha256 := source.fileHashSha256
      schemaVersion := source.schemaVersion
    }
  }

def inputHashUnchanged
  (before : SourceInput)
  (after : SourceInput) : Prop :=
  before.fileHashSha256 = after.fileHashSha256

def savedSourceRefKeepsOriginalHash
  (source : SourceInput)
  (result : SourceSaveResult) : Prop :=
  result.inputRef.fileHashSha256 = source.fileHashSha256

def sourceSavePreservesInputHash
  (source : SourceInput)
  (result : SourceSaveResult) : Prop :=
  inputHashUnchanged source result.savedSource ∧
  savedSourceRefKeepsOriginalHash source result

def runArtifactInputRefKeepsOriginalHash
  (source : SourceInput)
  (artifact : RunArtifact) : Prop :=
  artifact.inputRef.fileHashSha256 = source.fileHashSha256

end LaborLens
