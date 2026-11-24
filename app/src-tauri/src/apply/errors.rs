use divelens_core::Error;
use serde::Serialize;

use crate::dive_log::LogChanged;
use crate::lightroom::ApplyOutcome;

/// Why "Create Collections" or "Tag Selection" could not go on (also
/// `gpsNotWritten` as a Lightroom failure: everything written but the GPS
/// and altitude).
#[derive(Debug, Serialize)]
#[serde(tag = "error", rename_all = "camelCase")]
pub(crate) enum ApplyError {
    NothingChosen,
    NotConnected,
    /// The link dropped while Lightroom was writing: the result is unknown.
    LinkLost,
    BadGps,
    UnknownDive,
    UnknownSite,
    /// No photo search to work on (link reconnected, app restarted, or a
    /// newer search replaced it). In Tag Selection: the dive has a profile
    /// but the selection the screen shows is no longer kept, it must be
    /// described again.
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

impl From<LogChanged> for ApplyError {
    fn from(_: LogChanged) -> Self {
        Self::LogChanged
    }
}

impl From<ApplyOutcome> for ApplyError {
    fn from(outcome: ApplyOutcome) -> Self {
        let ApplyOutcome::Failed { reason, detail } = outcome;
        Self::Lightroom { reason, detail }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;
    use crate::apply::{answered, heard};
    use crate::tag::TagApplied;

    #[test]
    fn a_refused_write_is_an_error_as_the_screen_reads_it() {
        let seen = std::cell::RefCell::new(Vec::new());
        let on_refused = |o: &ApplyOutcome| seen.borrow_mut().push(o.clone());
        assert!(matches!(answered(Ok(3), on_refused), Ok(3)));
        assert!(seen.borrow().is_empty(), "never called on a written answer");
        let busy = ApplyOutcome::Failed {
            reason: "busy".into(),
            detail: None,
        };
        let refused = answered(Err::<usize, _>(busy.clone()), on_refused).map(TagApplied::from);
        assert_eq!(*seen.borrow(), vec![busy], "called once with the outcome");
        let refused = refused.unwrap_err();
        assert!(matches!(
            refused,
            ApplyError::Lightroom { ref reason, detail: None } if reason == "busy"
        ));
        assert_eq!(
            serde_json::to_value(refused).unwrap(),
            serde_json::json!({ "error": "lightroom", "reason": "busy", "detail": null })
        );
        // As Tag Selection returns a written answer.
        let tagged = answered(Ok(2), |_| ()).map(TagApplied::from).unwrap();
        assert_eq!(
            serde_json::to_value(tagged).unwrap(),
            serde_json::json!({ "count": 2, "collection": null })
        );
    }

    #[test]
    fn a_lost_link_during_a_write_is_link_lost() {
        let (reply, answer) = mpsc::channel::<Result<usize, ApplyOutcome>>();
        drop(reply);
        assert!(matches!(heard(answer.recv()), Err(ApplyError::LinkLost)));
        let (reply, answer) = mpsc::channel::<Result<usize, ApplyOutcome>>();
        reply.send(Ok(4)).unwrap();
        assert!(matches!(heard(answer.recv()), Ok(Ok(4))));
    }

    /// The JSON the screens read, one line per error.
    #[test]
    fn every_error_keeps_its_json() {
        let json = |e: ApplyError| serde_json::to_value(e).unwrap();
        for (error, tag) in [
            (ApplyError::NothingChosen, "nothingChosen"),
            (ApplyError::NotConnected, "notConnected"),
            (ApplyError::LinkLost, "linkLost"),
            (ApplyError::BadGps, "badGps"),
            (ApplyError::UnknownDive, "unknownDive"),
            (ApplyError::UnknownSite, "unknownSite"),
            (ApplyError::LogChanged, "logChanged"),
            (ApplyError::NoPhotos, "noPhotos"),
            (ApplyError::NothingToWrite, "nothingToWrite"),
        ] {
            assert_eq!(json(error), serde_json::json!({ "error": tag }));
        }
        assert_eq!(
            json(ApplyError::Source {
                detail: Error::Database("x".into())
            })["error"],
            "source"
        );
    }
}
