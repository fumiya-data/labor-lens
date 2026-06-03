import LaborLens.Spec.Privacy

namespace LaborLens

theorem publicReportDoesNotExposeSensitiveKind
  (report : PublicReport)
  (kind : SensitiveKind) :
  doesNotExposeSensitiveKind report kind := by
  unfold doesNotExposeSensitiveKind
  unfold reportExposesSensitiveKind
  intro h
  rcases h with ⟨field, _fieldInReport, fieldExposesSensitiveKind⟩
  unfold publicFieldExposesSensitiveKind at fieldExposesSensitiveKind
  exact fieldExposesSensitiveKind

theorem hideFatigueValueFromPublicReport
  (dataset : InternalDataset) :
  doesNotExposeFatigueValue (privacyFilter dataset) := by
  exact publicReportDoesNotExposeSensitiveKind
    (privacyFilter dataset)
    SensitiveKind.fatigueValue

theorem hideSleepDurationFromPublicReport
  (dataset : InternalDataset) :
  doesNotExposeSleepDuration (privacyFilter dataset) := by
  exact publicReportDoesNotExposeSensitiveKind
    (privacyFilter dataset)
    SensitiveKind.sleepDuration

theorem hideFatigueCommentFromPublicReport
  (dataset : InternalDataset) :
  doesNotExposeFatigueComment (privacyFilter dataset) := by
  exact publicReportDoesNotExposeSensitiveKind
    (privacyFilter dataset)
    SensitiveKind.fatigueComment

theorem hideSuppressedSensitiveKindFromPublicReport
  (dataset : InternalDataset)
  (kind : SensitiveKind) :
  doesNotExposeSensitiveKind (privacyFilter dataset) kind := by
  exact publicReportDoesNotExposeSensitiveKind (privacyFilter dataset) kind

end LaborLens
