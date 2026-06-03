import LaborLens.Core.Dataset

namespace LaborLens

def publicFieldExposesSensitiveKind
  (_field : PublicField)
  (_kind : SensitiveKind) : Prop :=
  False

def reportExposesSensitiveKind
  (report : PublicReport)
  (kind : SensitiveKind) : Prop :=
  ∃ field, field ∈ report.fields ∧ publicFieldExposesSensitiveKind field kind

def doesNotExposeSensitiveKind
  (report : PublicReport)
  (kind : SensitiveKind) : Prop :=
  ¬ reportExposesSensitiveKind report kind

def doesNotExposeFatigueValue (report : PublicReport) : Prop :=
  doesNotExposeSensitiveKind report SensitiveKind.fatigueValue

def doesNotExposeSleepDuration (report : PublicReport) : Prop :=
  doesNotExposeSensitiveKind report SensitiveKind.sleepDuration

def doesNotExposeFatigueComment (report : PublicReport) : Prop :=
  doesNotExposeSensitiveKind report SensitiveKind.fatigueComment

def privacyFilter (_dataset : InternalDataset) : PublicReport :=
  {
    fields := [
      PublicField.suppressionNotice
        "個人の疲労値、睡眠時間、疲労コメントは抑制される"
    ]
  }

end LaborLens
