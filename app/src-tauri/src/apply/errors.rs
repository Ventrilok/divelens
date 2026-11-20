use divelens_core::Error;
use serde::Serialize;

use crate::dive_log::LogChanged;
use crate::lightroom::ApplyOutcome;

/// Why "Create Collections" could not go on.
#[derive(Debug, Serialize)]
#[serde(tag = "error", rename_all = "camelCase")]
pub(crate) enum CollectionsError {
    NothingChosen,
    NotConnected,
    LinkLost,
    /// No photo search to work on (link reconnected, app restarted, or a
    /// newer search replaced it).
    NoPhotos,
    /// No photo goes to any dive.
    NothingToWrite,
    /// The screen shows another dive log than the configured one, or the
    /// dives were read from another one.
    LogChanged,
    Source {
        detail: Error,
    },
    /// Lightroom refused or failed; `reason` as in [`ApplyOutcome::Failed`].
    #[serde(rename_all = "camelCase")]
    Lightroom {
        reason: String,
        detail: Option<String>,
    },
}

impl From<LogChanged> for CollectionsError {
    fn from(_: LogChanged) -> Self {
        Self::LogChanged
    }
}

impl From<ApplyOutcome> for CollectionsError {
    fn from(outcome: ApplyOutcome) -> Self {
        match outcome {
            ApplyOutcome::Failed { reason, detail } => Self::Lightroom { reason, detail },
            ApplyOutcome::Applied { .. } => Self::Lightroom {
                reason: "error".into(),
                detail: Some("unexpected answer".into()),
            },
        }
    }
}
