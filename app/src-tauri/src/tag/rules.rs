//! Tag Selection's rules: what a choice made on the screen writes, and
//! which refusal wins when several apply. Nothing here reads the dive log,
//! the settings or Lightroom: those reads are handed in, as closures where
//! a rule decides whether they happen, so they happen exactly when they
//! must and the first refusal wins as before.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use std::sync::mpsc::RecvError;

use divelens_core::collections::Span;
use divelens_core::tags::{DiveTags, LocationTags, TagRequest, Units, valid_gps};
use divelens_core::{Dive, Error, Site};

use super::selection::{InDive, PerPhoto, Refusal, Selection};
use super::{LocationChoice, TagApplied, TagError};
use crate::apply::searches::Searches;
use crate::lightroom::ApplyOutcome;

/// A position edited on the screen is whole and valid, or absent: half a
/// position, or one off the globe, is refused.
pub(super) fn check_gps(lat: Option<f64>, lon: Option<f64>) -> Result<(), TagError> {
    match (lat, lon) {
        (Some(a), Some(b)) if !valid_gps(a, b) => Err(TagError::BadGps),
        (Some(_), None) | (None, Some(_)) => Err(TagError::BadGps),
        _ => Ok(()),
    }
}

/// The dive `id` names (the first, should two share it).
pub(super) fn find_dive(dives: Vec<Dive>, id: &str) -> Result<Dive, TagError> {
    dives
        .into_iter()
        .find(|d| d.id == id)
        .ok_or(TagError::UnknownDive)
}

/// The site `site_id` names (the first, should two share it).
pub(super) fn find_site(sites: Vec<Site>, site_id: &str) -> Result<Site, TagError> {
    sites
        .into_iter()
        .find(|s| s.id.as_deref() == Some(site_id))
        .ok_or(TagError::UnknownSite)
}

/// The location to write, if one is ticked: the dive's own site (a dive
/// must be chosen), or another site of the log with its position possibly
/// edited. The position is checked before `sites` reads the log's sites.
pub(super) fn location(
    choice: Option<&LocationChoice>,
    dive: Option<&Dive>,
    sites: impl FnOnce() -> Result<Vec<Site>, Error>,
) -> Result<Option<LocationTags>, TagError> {
    Ok(match choice {
        None => None,
        Some(LocationChoice::DiveSite) => {
            let dive = dive.ok_or(TagError::NothingChosen)?;
            Some(LocationTags::new(&dive.site, None, None))
        }
        Some(LocationChoice::Site { site_id, lat, lon }) => {
            check_gps(*lat, *lon)?;
            let sites = sites().map_err(|detail| TagError::Source { detail })?;
            let site = find_site(sites, site_id)?;
            Some(LocationTags::new(&site, *lat, *lon))
        }
    })
}

/// The dive data and location to write, before any per-photo value: at
/// least one of them. `source` names the dive application on the photos.
pub(super) fn base_request(
    dive: Option<&Dive>,
    location: Option<LocationTags>,
    units: Units,
    source: &str,
) -> Result<TagRequest, TagError> {
    if dive.is_none() && location.is_none() {
        return Err(TagError::NothingChosen);
    }
    Ok(TagRequest {
        dive: dive.map(|d| DiveTags::new(d, units, source)),
        location,
        ..TagRequest::default()
    })
}

/// The selection description `search` the screen shows, as the plugin's
/// number for it and its photos taken during `dive` with the offsets shown;
/// `None` when the screen showed none (it could not read the profile). The
/// descriptions are locked only when one is named, and only meanwhile.
pub(super) fn described(
    selections: &Mutex<Searches<Selection>>,
    search: Option<u64>,
    dive: &Dive,
    offsets: &HashMap<String, i32>,
) -> Result<Option<(u64, Vec<InDive>)>, TagError> {
    Ok(match search {
        None => None,
        Some(n) => {
            let selections = selections.lock().expect("selections");
            let selection = selections.get(n).ok_or(TagError::NoPhotos)?;
            Some((
                selection.described,
                selection.in_dive(Span::from(dive), offsets),
            ))
        }
    })
}

/// Adds the per-photo values to `req`; returns the offsets applied, to
/// remember once written, and the error of a profile that could not be read
/// (the dive data are then written alone, the caller logs it).
pub(super) fn apply_per_photo(
    req: &mut TagRequest,
    outcome: Result<PerPhoto, Refusal>,
) -> Result<(BTreeMap<String, i32>, Option<Error>), TagError> {
    match outcome {
        Ok(PerPhoto::Values {
            described,
            photos,
            applied,
        }) => {
            req.selection = Some(described);
            req.photos = photos;
            Ok((applied, None))
        }
        Ok(PerPhoto::Unreadable(e)) => Ok((BTreeMap::new(), Some(e))),
        Err(Refusal::Describe) => Err(TagError::NoPhotos),
        Err(Refusal::Profile(detail)) => Err(TagError::Source { detail }),
    }
}

/// What Lightroom answered: the photos written, or a refusal as the
/// screen reads it.
pub(super) fn applied(answer: Result<usize, ApplyOutcome>) -> Result<TagApplied, TagError> {
    answer
        .map(|count| TagApplied { count })
        .map_err(TagError::from)
}

/// The answer of the plugin link, or "link lost" when its reply channel
/// closed without one (the link dropped while Lightroom was writing).
pub(super) fn heard<T>(received: Result<T, RecvError>) -> Result<T, TagError> {
    received.map_err(|_| TagError::LinkLost)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{Duration, NaiveDate};
    use divelens_core::profile::PhotoPoint;
    use divelens_core::tags::PhotoTags;

    use crate::apply::offsets::Offsets;
    use crate::lightroom::FoundPhoto;

    /// A fictional site.
    fn site(id: &str, name: &str) -> Site {
        Site {
            id: Some(id.into()),
            name: Some(name.into()),
            country: Some("Nowhere Islands".into()),
            lat: Some(-8.5),
            lon: Some(115.25),
            ..Site::default()
        }
    }

    /// A fictional dive, 50 minutes from 09:00 on 30 December 2025, with a
    /// profile.
    fn dive(id: &str, number: u32) -> Dive {
        let start = NaiveDate::from_ymd_opt(2025, 12, 30)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();
        Dive {
            id: id.into(),
            number: Some(number),
            start,
            end: start + Duration::minutes(50),
            duration_s: 3000,
            max_depth_m: Some(30.0),
            temp_high_c: Some(28.0),
            temp_low_c: Some(26.0),
            rating: None,
            site: site("s1", "Coral Garden"),
            has_profile: true,
            invalid: false,
            notes: None,
        }
    }

    fn garbled() -> Error {
        Error::BadProfile("garbled".into())
    }

    #[test]
    fn half_a_gps_position_is_refused() {
        assert!(matches!(check_gps(Some(12.5), None), Err(TagError::BadGps)));
        assert!(matches!(
            check_gps(None, Some(-45.25)),
            Err(TagError::BadGps)
        ));
        assert!(matches!(
            check_gps(Some(91.0), Some(0.0)),
            Err(TagError::BadGps)
        ));
        assert!(check_gps(Some(12.5), Some(-45.25)).is_ok());
        assert!(check_gps(None, None).is_ok());
    }

    #[test]
    fn unknown_dive_or_site_is_refused() {
        assert!(matches!(find_dive(vec![], "x"), Err(TagError::UnknownDive)));
        assert!(matches!(find_site(vec![], "x"), Err(TagError::UnknownSite)));
        // The one the id names; the first when two share it.
        let dives = vec![dive("B", 700), dive("A", 701), dive("A", 702)];
        assert_eq!(find_dive(dives, "A").unwrap().number, Some(701));
        let sites = vec![
            Site::default(),
            site("s2", "Blue Hole"),
            site("s1", "Coral Garden"),
            site("s1", "Shark Point"),
        ];
        assert_eq!(
            find_site(sites, "s1").unwrap().name.as_deref(),
            Some("Coral Garden")
        );
    }

    #[test]
    fn bad_gps_wins_before_the_sites_are_read() {
        let half = LocationChoice::Site {
            site_id: "s1".into(),
            lat: Some(1.0),
            lon: None,
        };
        assert!(matches!(
            location(Some(&half), None, || panic!("sites read")),
            Err(TagError::BadGps)
        ));
        // A whole position: the sites are read, then the site is looked for.
        let edited = LocationChoice::Site {
            site_id: "s1".into(),
            lat: Some(12.5),
            lon: Some(-45.25),
        };
        assert!(matches!(
            location(Some(&edited), None, || Err(garbled())),
            Err(TagError::Source {
                detail: Error::BadProfile(_)
            })
        ));
        assert!(matches!(
            location(Some(&edited), None, || Ok(vec![site("s2", "Blue Hole")])),
            Err(TagError::UnknownSite)
        ));
        let found = location(Some(&edited), None, || Ok(vec![site("s1", "Coral Garden")]));
        assert_eq!(
            found.unwrap(),
            Some(LocationTags::new(
                &site("s1", "Coral Garden"),
                Some(12.5),
                Some(-45.25)
            ))
        );
        // No location ticked: nothing read.
        assert!(matches!(
            location(None, None, || panic!("sites read")),
            Ok(None)
        ));
    }

    #[test]
    fn the_dive_site_needs_a_dive() {
        assert!(matches!(
            location(Some(&LocationChoice::DiveSite), None, || panic!(
                "sites read"
            )),
            Err(TagError::NothingChosen)
        ));
        assert!(matches!(
            base_request(None, None, Units::default(), "MacDive"),
            Err(TagError::NothingChosen)
        ));
        // With a dive: its own site, without reading the others.
        let d = dive("A", 701);
        let site_of_dive = location(Some(&LocationChoice::DiveSite), Some(&d), || {
            panic!("sites read")
        })
        .unwrap();
        assert_eq!(site_of_dive, Some(LocationTags::new(&d.site, None, None)));
        let request = base_request(Some(&d), None, Units::default(), "MacDive").unwrap();
        assert_eq!(
            request,
            TagRequest {
                dive: Some(DiveTags::new(&d, Units::default(), "MacDive")),
                ..TagRequest::default()
            }
        );
        let request = base_request(None, site_of_dive.clone(), Units::default(), "MacDive");
        assert_eq!(request.unwrap().location, site_of_dive);
    }

    #[test]
    fn an_unknown_description_is_no_photos() {
        let d = dive("A", 701);
        // Description 2 kept: a TG-7 photo 4 h behind the dive computer.
        let selections = Mutex::new(Searches::default());
        let found = vec![FoundPhoto {
            id: "1".into(),
            time: Some("2025-12-30T05:05:00".into()),
            model: Some("TG-7".into()),
            serial: Some("A1".into()),
        }];
        assert!(selections.lock().unwrap().start(2));
        assert!(
            selections
                .lock()
                .unwrap()
                .finish(2, Selection::new(42, found, &Offsets::default()))
        );
        let offsets = HashMap::from([("TG-7|A1".to_string(), 240)]);
        assert!(matches!(
            described(&selections, Some(3), &d, &offsets),
            Err(TagError::NoPhotos)
        ));
        assert!(matches!(
            described(&selections, None, &d, &offsets),
            Ok(None)
        ));
        let Ok(Some((number, in_dive))) = described(&selections, Some(2), &d, &offsets) else {
            panic!("description 2 is kept");
        };
        assert_eq!(number, 42);
        let ids: Vec<_> = in_dive.iter().map(|p| (p.id.as_str(), p.offset)).collect();
        assert_eq!(ids, [("1", 240)]);
    }

    #[test]
    fn per_photo_values_are_added_to_the_request() {
        let mut request = TagRequest::default();
        assert!(matches!(
            apply_per_photo(&mut request, Err(Refusal::Describe)),
            Err(TagError::NoPhotos)
        ));
        assert!(matches!(
            apply_per_photo(&mut request, Err(Refusal::Profile(garbled()))),
            Err(TagError::Source {
                detail: Error::BadProfile(_)
            })
        ));
        // A profile that cannot be read: the dive data alone, the error
        // handed back to be logged.
        let unreadable = apply_per_photo(&mut request, Ok(PerPhoto::Unreadable(garbled())));
        assert!(matches!(
            unreadable,
            Ok((ref applied, Some(Error::BadProfile(_)))) if applied.is_empty()
        ));
        assert_eq!(request, TagRequest::default());

        // Per-photo values: the depth goes to the altitude only when the
        // setting is on, which the caller applies to the whole request.
        let values = || {
            Ok(PerPhoto::Values {
                described: 42,
                photos: vec![PhotoTags::new(
                    "1".into(),
                    Some(PhotoPoint {
                        elapsed_s: 300,
                        depth_m: 3.0,
                        temp_c: Some(27.8),
                    }),
                    240,
                    Units::default(),
                )],
                applied: BTreeMap::from([("TG-7|A1".to_string(), 240)]),
            })
        };
        for on in [false, true] {
            let mut request = TagRequest::default();
            let (applied, unreadable) = apply_per_photo(&mut request, values()).unwrap();
            if on {
                request.copy_depth_to_altitude();
            }
            assert!(unreadable.is_none());
            assert_eq!(applied, BTreeMap::from([("TG-7|A1".to_string(), 240)]));
            assert_eq!(request.selection, Some(42));
            assert_eq!(request.photos.len(), 1);
            let altitude = request.photos[0].gps_altitude;
            assert_eq!(altitude, on.then_some(-3.0));
        }
    }

    #[test]
    fn a_refused_tagging_is_an_error() {
        assert_eq!(applied(Ok(3)).unwrap().count, 3);
        let refused = applied(Err(ApplyOutcome::Failed {
            reason: "busy".into(),
            detail: None,
        }));
        assert!(matches!(
            refused,
            Err(TagError::Lightroom { ref reason, detail: None }) if reason == "busy"
        ));
        // As the screen reads it: the same shape as Create Collections.
        assert_eq!(
            serde_json::to_value(refused.unwrap_err()).unwrap(),
            serde_json::json!({ "error": "lightroom", "reason": "busy", "detail": null })
        );
        assert_eq!(
            serde_json::to_value(applied(Ok(2)).unwrap()).unwrap(),
            serde_json::json!({ "count": 2 })
        );
    }

    #[test]
    fn a_lost_link_during_tagging_is_link_lost() {
        let (reply, answer) = std::sync::mpsc::channel::<Result<usize, ApplyOutcome>>();
        drop(reply);
        assert!(matches!(heard(answer.recv()), Err(TagError::LinkLost)));
        let (reply, answer) = std::sync::mpsc::channel::<Result<usize, ApplyOutcome>>();
        reply.send(Ok(4)).unwrap();
        assert!(matches!(heard(answer.recv()), Ok(Ok(4))));
    }

    #[test]
    fn the_altitude_option_leaves_a_request_without_photo_values_unchanged() {
        let d = dive("A", 701);
        let location = Some(LocationTags::new(&d.site, Some(12.5), Some(-45.25)));
        let request = base_request(Some(&d), location, Units::default(), "MacDive").unwrap();
        let mut copied = request.clone();
        copied.copy_depth_to_altitude();
        assert_eq!(copied, request);
    }
}
