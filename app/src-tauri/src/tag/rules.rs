//! Tag Selection's rules: what a choice made on the screen writes, and
//! which refusal wins when several apply. Nothing here reads the dive log,
//! the settings or Lightroom: those reads are handed in, as closures where
//! a rule decides whether they happen, so they happen exactly when they
//! must and the first refusal wins as before.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use divelens_core::collections::{Span, collection_name};
use divelens_core::profile::Sample;
use divelens_core::tags::{CollectionPlace, DiveTags, Kept, LocationTags, TagRequest, Units};
use divelens_core::{Dive, Error, Site};

use super::LocationChoice;
use super::selection::{InDive, PerPhoto, Refusal, Selection, applied_offsets, per_photo};
use crate::apply::errors::ApplyError;
use crate::apply::searches::Searches;
use crate::apply::site::another_site;

/// The dive `id` names (the first, should two share it).
pub(super) fn find_dive(dives: Vec<Dive>, id: &str) -> Result<Dive, ApplyError> {
    dives
        .into_iter()
        .find(|d| d.id == id)
        .ok_or(ApplyError::UnknownDive)
}

/// The location to write, if one is ticked: the dive's own site (a dive
/// must be chosen), or another site of the log with its position possibly
/// edited. The position is checked before `sites` reads the log's sites.
pub(super) fn location(
    choice: Option<&LocationChoice>,
    dive: Option<&Dive>,
    sites: impl FnOnce() -> Result<Vec<Site>, Error>,
) -> Result<Option<LocationTags>, ApplyError> {
    Ok(match choice {
        None => None,
        Some(LocationChoice::DiveSite) => {
            let dive = dive.ok_or(ApplyError::NothingChosen)?;
            Some(LocationTags::new(&dive.site, None, None))
        }
        Some(LocationChoice::Site { site_id, lat, lon }) => {
            Some(another_site(site_id, *lat, *lon, sites)?)
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
) -> Result<TagRequest, ApplyError> {
    if dive.is_none() && location.is_none() {
        return Err(ApplyError::NothingChosen);
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
) -> Result<Option<(u64, Vec<InDive>)>, ApplyError> {
    Ok(match search {
        None => None,
        Some(n) => {
            let selections = selections.lock().expect("selections");
            let selection = selections.get(n).ok_or(ApplyError::NoPhotos)?;
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
) -> Result<(BTreeMap<String, i32>, Option<Error>), ApplyError> {
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
        Err(Refusal::Describe) => Err(ApplyError::NoPhotos),
        Err(Refusal::Profile(detail)) => Err(ApplyError::Source { detail }),
    }
}

/// Where the selection goes when the collections are ticked: the collection
/// of the chosen dive, named as Create Collections names it. Without the
/// tick, or without a dive to name it, no collection. `settings` gives the
/// collection settings' parent and group, read only when a collection goes.
pub(super) fn collection_place(
    collections: bool,
    dive: Option<&Dive>,
    settings: impl FnOnce() -> (Vec<String>, Option<String>),
) -> Option<CollectionPlace> {
    let dive = dive.filter(|_| collections)?;
    let (parent, group) = settings();
    Some(CollectionPlace {
        name: collection_name(dive),
        parent,
        group,
    })
}

/// What `build_request` made: the request, the offsets applied (to remember
/// once written) and the error of a profile that could not be read.
pub(super) struct Built {
    pub request: TagRequest,
    pub applied: BTreeMap<String, i32>,
    pub unreadable: Option<Error>,
}

/// What the diver ticked, when the screen sends options: the parts of the
/// request kept, and where the dive's collection goes (`None`: no collection).
pub(super) struct Ticked {
    pub kept: Kept,
    pub collection: Option<CollectionPlace>,
}

/// The request Tag Selection sends, in the order the refusals win: the dive
/// data and location, then, for a dive with a profile, the per-photo values
/// (`described` gives the selection the screen shows, `profile` reads the
/// profile of a dive id: each only when needed), then the "depth to
/// altitude" setting (`depth_to_altitude`, read last, so a refusal before it
/// never reads it). A profile that cannot be read leaves the dive data alone.
/// `ticked` is what the diver ticked; `None` (the current screens): the
/// request is the one they always got. With options, nothing left to write
/// is refused before anything is read; with the dive details unticked, the
/// description still gives the offsets to remember, the profile is not read.
#[allow(clippy::too_many_arguments)] // the reads handed in as closures, then what is ticked
pub(super) fn build_request(
    dive: Option<Dive>,
    location: Option<LocationTags>,
    units: Units,
    source: &str,
    described: impl FnOnce(&Dive) -> Result<Option<(u64, Vec<InDive>)>, ApplyError>,
    profile: impl FnOnce(&str) -> Result<Vec<Sample>, Error>,
    depth_to_altitude: impl FnOnce() -> bool,
    ticked: Option<Ticked>,
) -> Result<Built, ApplyError> {
    let mut request = base_request(dive.as_ref(), location, units, source)?;
    // Without options nothing is removed: a location with no field at all is
    // still sent, as the current screens always did.
    let mut dive_kept = true;
    if let Some(Ticked { kept, collection }) = ticked {
        request.collection = collection;
        request.keep(kept);
        if request.is_empty() {
            return Err(ApplyError::NothingChosen);
        }
        dive_kept = kept.dive;
    }
    let mut applied = BTreeMap::new();
    let mut unreadable = None;
    if let Some(dive) = dive.filter(|d| d.has_profile) {
        let described = described(&dive)?;
        if dive_kept {
            let outcome = per_photo(&dive, described, units, || profile(&dive.id));
            (applied, unreadable) = apply_per_photo(&mut request, outcome)?;
        } else if let Some((_, in_dive)) = described {
            // Unticked dive details: nothing more in the request and the
            // profile never read, but the offsets of the cameras with a
            // photo in the dive are still remembered, as when ticked.
            applied = applied_offsets(&in_dive);
        }
    }
    with_altitude(&mut request, depth_to_altitude);
    Ok(Built {
        request,
        applied,
        unreadable,
    })
}

/// The "depth to altitude" setting applied to the whole request, as Create
/// Collections does: a request without per-photo values has nothing to copy.
/// The setting is read here only, so a refusal before it never reads it.
pub(super) fn with_altitude(req: &mut TagRequest, depth_to_altitude: impl FnOnce() -> bool) {
    if depth_to_altitude() {
        req.copy_depth_to_altitude();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

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
    fn unknown_dive_or_site_is_refused() {
        assert!(matches!(
            find_dive(vec![], "x"),
            Err(ApplyError::UnknownDive)
        ));
        // The one the id names; the first when two share it.
        let dives = vec![dive("B", 700), dive("A", 701), dive("A", 702)];
        assert_eq!(find_dive(dives, "A").unwrap().number, Some(701));
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
            Err(ApplyError::BadGps)
        ));
        // A whole position: the sites are read, then the site is looked for.
        let edited = LocationChoice::Site {
            site_id: "s1".into(),
            lat: Some(12.5),
            lon: Some(-45.25),
        };
        assert!(matches!(
            location(Some(&edited), None, || Err(garbled())),
            Err(ApplyError::Source {
                detail: Error::BadProfile(_)
            })
        ));
        assert!(matches!(
            location(Some(&edited), None, || Ok(vec![site("s2", "Blue Hole")])),
            Err(ApplyError::UnknownSite)
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
            Err(ApplyError::NothingChosen)
        ));
        assert!(matches!(
            base_request(None, None, Units::default(), "MacDive"),
            Err(ApplyError::NothingChosen)
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
            Err(ApplyError::NoPhotos)
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
            Err(ApplyError::NoPhotos)
        ));
        assert!(matches!(
            apply_per_photo(&mut request, Err(Refusal::Profile(garbled()))),
            Err(ApplyError::Source {
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
            with_altitude(&mut request, || on);
            assert!(unreadable.is_none());
            assert_eq!(applied, BTreeMap::from([("TG-7|A1".to_string(), 240)]));
            assert_eq!(request.selection, Some(42));
            assert_eq!(request.photos.len(), 1);
            let altitude = request.photos[0].gps_altitude;
            assert_eq!(altitude, on.then_some(-3.0));
        }
    }

    #[test]
    fn a_refusal_before_the_altitude_never_reads_the_setting() {
        let build = |described: Option<(u64, Vec<InDive>)>, profile: Result<Vec<Sample>, Error>| {
            build_request(
                Some(dive("A", 701)),
                None,
                Units::default(),
                "MacDive",
                |_| Ok(described),
                |_| profile,
                || panic!("setting read"),
                None,
            )
        };
        // The screen showed no selection, the profile reads: it must be described.
        assert!(matches!(build(None, Ok(vec![])), Err(ApplyError::NoPhotos)));
        // Photos shown, but the profile no longer reads.
        let photos = Some((42, vec![photo_in_dive()]));
        assert!(matches!(
            build(photos, Err(garbled())),
            Err(ApplyError::Source { .. })
        ));
        // An unknown description: refused by `described` itself.
        let refused = build_request(
            Some(dive("A", 701)),
            None,
            Units::default(),
            "MacDive",
            |_| Err(ApplyError::NoPhotos),
            |_| panic!("profile read"),
            || panic!("setting read"),
            None,
        );
        assert!(matches!(refused, Err(ApplyError::NoPhotos)));
        // Nothing chosen: refused before anything is read.
        let nothing = build_request(
            None,
            None,
            Units::default(),
            "MacDive",
            |_| panic!("described"),
            |_| panic!("profile read"),
            || panic!("setting read"),
            None,
        );
        assert!(matches!(nothing, Err(ApplyError::NothingChosen)));
    }

    /// A photo taken 25 minutes into the dive, with no clock offset.
    fn photo_in_dive() -> InDive {
        InDive {
            id: "1".into(),
            camera: "TG-7|A1".into(),
            offset: 0,
            corrected: NaiveDate::from_ymd_opt(2025, 12, 30)
                .unwrap()
                .and_hms_opt(9, 25, 0)
                .unwrap(),
        }
    }

    #[test]
    fn the_altitude_setting_decides_the_altitude_of_the_request_built() {
        let sample = |time_s, depth_m| Sample {
            time_s,
            depth_m,
            temp_c: None,
        };
        let profile = vec![sample(0, 0.0), sample(3000, 30.0)];
        for on in [false, true] {
            let reads = Cell::new(0);
            let built = build_request(
                Some(dive("A", 701)),
                None,
                Units::default(),
                "MacDive",
                |_| Ok(Some((42, vec![photo_in_dive()]))),
                |_| Ok(profile.clone()),
                || {
                    reads.set(reads.get() + 1);
                    on
                },
                None,
            )
            .unwrap();
            assert_eq!(reads.get(), 1);
            assert_eq!(built.request.selection, Some(42));
            assert_eq!(built.request.photos.len(), 1);
            assert_eq!(built.applied, BTreeMap::from([("TG-7|A1".to_string(), 0)]));
            let altitude = built.request.photos[0].gps_altitude;
            assert_eq!(altitude, on.then_some(-15.0), "setting on: {on}");
        }
        // A dive without a profile: no per-photo step, the setting is still
        // read, and there is nothing to copy.
        let mut flat = dive("A", 701);
        flat.has_profile = false;
        let reads = Cell::new(0);
        let built = build_request(
            Some(flat),
            None,
            Units::default(),
            "MacDive",
            |_| panic!("described"),
            |_| panic!("profile read"),
            || {
                reads.set(reads.get() + 1);
                true
            },
            None,
        )
        .unwrap();
        assert_eq!(reads.get(), 1);
        assert!(built.request.photos.is_empty());
        assert!(built.applied.is_empty());
    }

    fn place() -> CollectionPlace {
        CollectionPlace {
            name: "701 - Coral Garden".into(),
            parent: vec![],
            group: None,
        }
    }

    #[test]
    fn a_collection_only_with_the_tick_and_a_chosen_dive() {
        let d = dive("A", 701);
        let settings = || (vec!["Trips".to_string()], Some("Dive Lens".to_string()));
        assert_eq!(
            collection_place(true, Some(&d), settings),
            Some(CollectionPlace {
                name: collection_name(&d),
                parent: vec!["Trips".into()],
                group: Some("Dive Lens".into()),
            })
        );
        // No dive to name it, or the collections unticked: no collection,
        // and the settings are not read.
        assert_eq!(
            collection_place(true, None, || panic!("settings read")),
            None
        );
        assert_eq!(
            collection_place(false, Some(&d), || panic!("settings read")),
            None
        );
    }

    #[test]
    fn without_options_the_request_is_todays() {
        let d = dive("A", 701);
        let sample = |time_s, depth_m| Sample {
            time_s,
            depth_m,
            temp_c: None,
        };
        let profile = vec![sample(0, 0.0), sample(3000, 30.0)];
        let location = Some(LocationTags::new(&d.site, Some(12.5), Some(-45.25)));
        let built = build_request(
            Some(d.clone()),
            location.clone(),
            Units::default(),
            "MacDive",
            |_| Ok(Some((42, vec![photo_in_dive()]))),
            |_| Ok(profile.clone()),
            || true,
            None,
        )
        .unwrap();
        let request = built.request;
        assert_eq!(
            request.dive,
            Some(DiveTags::new(&d, Units::default(), "MacDive"))
        );
        assert_eq!(request.location, location);
        assert_eq!(request.collection, None);
        assert_eq!(request.selection, Some(42));
        assert_eq!(request.photos.len(), 1);
        assert_eq!(request.photos[0].gps_altitude, Some(-15.0));
        assert_eq!(built.applied, BTreeMap::from([("TG-7|A1".to_string(), 0)]));
        assert!(built.unreadable.is_none());
    }

    #[test]
    fn without_options_a_site_with_no_field_is_sent_as_today() {
        // A site with no name, country, place or position: its location is
        // sent all empty, never dropped.
        let mut d = dive("A", 701);
        d.site = Site::default();
        let empty = LocationTags::new(&d.site, None, None);
        let sample = |time_s, depth_m| Sample {
            time_s,
            depth_m,
            temp_c: None,
        };
        let profile = vec![sample(0, 0.0), sample(3000, 30.0)];
        let built = build_request(
            Some(d.clone()),
            Some(empty.clone()),
            Units::default(),
            "MacDive",
            |_| Ok(Some((42, vec![photo_in_dive()]))),
            |_| Ok(profile.clone()),
            || false,
            None,
        )
        .unwrap();
        assert_eq!(
            built.request.location,
            Some(LocationTags {
                country: None,
                city: None,
                location: None,
                lat: None,
                lon: None,
            })
        );
        assert_eq!(
            built.request.dive,
            Some(DiveTags::new(&d, Units::default(), "MacDive"))
        );
        // The location alone, without a dive: sent too, not refused.
        let alone = build_request(
            None,
            Some(empty.clone()),
            Units::default(),
            "MacDive",
            |_| panic!("described"),
            |_| panic!("profile read"),
            || false,
            None,
        );
        let Ok(alone) = alone else {
            panic!("refused: {:?}", alone.err());
        };
        assert_eq!(
            alone.request,
            TagRequest {
                location: Some(empty),
                ..TagRequest::default()
            }
        );
    }

    #[test]
    fn dive_details_unticked_keep_the_dives_site_collection_and_offsets() {
        let d = dive("A", 701);
        let kept = Kept {
            dive: false,
            place: true,
            gps: true,
            collections: true,
        };
        let built = build_request(
            Some(d.clone()),
            Some(LocationTags::new(&d.site, None, None)),
            Units::default(),
            "MacDive",
            |_| Ok(Some((42, vec![photo_in_dive()]))),
            |_| panic!("profile read"),
            || false,
            Some(Ticked {
                kept,
                collection: Some(place()),
            }),
        )
        .unwrap();
        assert!(built.request.dive.is_none());
        assert!(built.request.photos.is_empty());
        assert_eq!(built.request.selection, None);
        assert_eq!(
            built.request.location,
            Some(LocationTags::new(&d.site, None, None))
        );
        assert_eq!(built.request.collection, Some(place()));
        assert_eq!(built.applied, BTreeMap::from([("TG-7|A1".to_string(), 0)]));
    }

    /// Photo 1 (the TG-7, no offset) and photo 2 (a GoPro, one hour) taken
    /// during the dive.
    fn two_cameras_in_dive() -> Vec<InDive> {
        let gopro = InDive {
            id: "2".into(),
            camera: "GoPro|".into(),
            offset: 60,
            corrected: NaiveDate::from_ymd_opt(2025, 12, 30)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
        };
        vec![photo_in_dive(), gopro]
    }

    #[test]
    fn dive_details_unticked_still_remember_the_offsets_of_cameras_in_the_dive() {
        let build = |dive_ticked: bool,
                     described: Result<Option<(u64, Vec<InDive>)>, ApplyError>,
                     profile: &dyn Fn() -> Result<Vec<Sample>, Error>| {
            let d = dive("A", 701);
            build_request(
                Some(d.clone()),
                Some(LocationTags::new(&d.site, None, None)),
                Units::default(),
                "MacDive",
                |_| described,
                |_| profile(),
                || false,
                Some(Ticked {
                    kept: Kept {
                        dive: dive_ticked,
                        ..Kept::ALL
                    },
                    collection: None,
                }),
            )
        };
        let never = || -> Result<Vec<Sample>, Error> { panic!("profile read") };
        let unticked = build(false, Ok(Some((42, two_cameras_in_dive()))), &never).unwrap();
        // The offsets of the cameras with a photo in the dive, from the
        // selection the screen shows, the profile never read...
        let both = BTreeMap::from([("GoPro|".to_string(), 60), ("TG-7|A1".to_string(), 0)]);
        assert_eq!(unticked.applied, both);
        // ...and nothing of them in the request: no per-photo value, no
        // selection to check.
        assert_eq!(unticked.request.dive, None);
        assert_eq!(unticked.request.selection, None);
        assert!(unticked.request.photos.is_empty());
        assert!(unticked.unreadable.is_none());
        // Exactly what the dive details ticked remember.
        let read = || Ok(vec![]);
        let ticked = build(true, Ok(Some((42, two_cameras_in_dive()))), &read).unwrap();
        assert_eq!(ticked.applied, unticked.applied);
        // The screen showed no description: nothing remembered, as with the
        // dive details ticked.
        let none = build(false, Ok(None), &never).unwrap();
        assert!(none.applied.is_empty());
        // A description no longer kept: refused, as with the dive details
        // ticked.
        assert!(matches!(
            build(false, Err(ApplyError::NoPhotos), &never),
            Err(ApplyError::NoPhotos)
        ));
    }

    /// Selection description 2, numbered 42 by the plug-in: three photos
    /// from a TG-7 on dive time, at 09:10 and 09:30 during the dive (09:00
    /// to 09:50), and at 10:30, after it.
    fn three_photos_two_in_the_dive() -> Mutex<Searches<Selection>> {
        let photo = |id: &str, time: &str| FoundPhoto {
            id: id.into(),
            time: Some(time.into()),
            model: Some("TG-7".into()),
            serial: Some("A1".into()),
        };
        let found = vec![
            photo("1", "2025-12-30T09:10:00"),
            photo("2", "2025-12-30T09:30:00"),
            photo("3", "2025-12-30T10:30:00"),
        ];
        let selections = Mutex::new(Searches::default());
        assert!(selections.lock().unwrap().start(2));
        assert!(
            selections
                .lock()
                .unwrap()
                .finish(2, Selection::new(42, found, &Offsets::default()))
        );
        selections
    }

    /// The request for the described selection, the dive details ticked or
    /// not, everything else ticked.
    fn built_for_selection(dive_ticked: bool) -> Built {
        let d = dive("A", 701);
        let selections = three_photos_two_in_the_dive();
        let sample = |time_s, depth_m| Sample {
            time_s,
            depth_m,
            temp_c: None,
        };
        let profile = vec![sample(0, 0.0), sample(3000, 30.0)];
        build_request(
            Some(d.clone()),
            Some(LocationTags::new(&d.site, None, None)),
            Units::default(),
            "MacDive",
            |dive| described(&selections, Some(2), dive, &HashMap::new()),
            |_| {
                assert!(dive_ticked, "profile read with the dive details unticked");
                Ok(profile.clone())
            },
            || false,
            Some(Ticked {
                kept: Kept {
                    dive: dive_ticked,
                    ..Kept::ALL
                },
                collection: Some(place()),
            }),
        )
        .unwrap()
    }

    #[test]
    fn tag_applied_counts_photos_with_depth() {
        // Two of the three photos were taken during the dive: they get a
        // depth, the third the dive data alone.
        let ticked = built_for_selection(true);
        assert_eq!(ticked.request.photos.len(), 2);
        assert_eq!(ticked.request.with_depth(), 2);
        // The same photos, the dive details unticked: no per-photo value.
        let unticked = built_for_selection(false);
        assert_eq!(unticked.request.with_depth(), 0);
    }

    #[test]
    fn gps_alone_on_a_site_without_gps_is_refused() {
        let mut d = dive("A", 701);
        d.site.lat = None;
        d.site.lon = None;
        let kept = Kept {
            dive: false,
            place: false,
            gps: true,
            collections: false,
        };
        let built = build_request(
            Some(d.clone()),
            Some(LocationTags::new(&d.site, None, None)),
            Units::default(),
            "MacDive",
            |_| panic!("described"),
            |_| panic!("profile read"),
            || panic!("setting read"),
            Some(Ticked {
                kept,
                collection: Some(place()),
            }),
        );
        assert!(matches!(built, Err(ApplyError::NothingChosen)));
    }

    #[test]
    fn no_dive_and_another_site_writes_the_place_only() {
        let other = site("s2", "Blue Hole");
        let built = build_request(
            None,
            Some(LocationTags::new(&other, None, None)),
            Units::default(),
            "MacDive",
            |_| panic!("described"),
            |_| panic!("profile read"),
            || false,
            Some(Ticked {
                kept: Kept::ALL,
                collection: None,
            }),
        )
        .unwrap();
        assert_eq!(
            built.request.location,
            Some(LocationTags::new(&other, None, None))
        );
        assert_eq!(built.request.collection, None);
        assert!(built.request.dive.is_none());
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
