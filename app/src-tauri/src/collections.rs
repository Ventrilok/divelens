//! "Create Collections" (spec F5): the commands the screen calls. The plan
//! (photos found, live counts, what to write) is in `plan`.

pub(crate) mod plan;

use std::sync::{Mutex, mpsc};

use divelens_core::tags::{CollectionsRequest, LocationTags};
use divelens_core::{Dive, Error, Site};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::apply::errors::ApplyError;
use crate::apply::offsets::Offsets;
use crate::apply::options::WriteOptions;
use crate::apply::searches::{Searches, keep_search};
use crate::apply::site::another_site;
use crate::apply::{ask, send, write_and_remember};
use crate::dive_log::{LogId, Writes, open, source_info};
use crate::files::config_dir;
use crate::lightroom::{CollectionSet, Command, WrittenCollection};
use crate::settings::{Settings, units_in_use};
use crate::tag::LocationChoice;
use plan::{Choices, Period, Preview, Session};

/// Photos found for "Create Collections", kept until the next search.
#[derive(Default)]
pub(crate) struct CollectionsState {
    searches: Mutex<Searches<Session>>,
}

/// Finds in Lightroom every photo of the chosen dives' period (whole
/// catalog) and their cameras. The dives and their profiles are read now,
/// from `log`, the dive log the screen shows, and kept for "Apply".
/// `search` numbers the search, higher than any the screen asked for before.
#[tauri::command]
pub(crate) async fn find_collection_photos(
    app: AppHandle,
    log: LogId,
    search: u64,
    dive_ids: Vec<String>,
) -> Result<Period, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CollectionsState>();
        let number = search;
        if !state.searches.lock().expect("collections").start(number) {
            return Err(ApplyError::NoPhotos);
        }
        // A search a newer one replaced stops at once.
        let replaced = || {
            !state
                .searches
                .lock()
                .expect("collections")
                .is_latest(number)
        };
        let info = source_info(&app);
        log.check(&info)?;
        let source = open(&app, &info)
            .map_err(|detail| ApplyError::Source { detail })?
            .log;
        let dives: Vec<Dive> = source
            .dives()
            .map_err(|detail| ApplyError::Source { detail })?
            .into_iter()
            .filter(|d| dive_ids.contains(&d.id))
            .collect();
        let (from, to) = plan::search_days(&dives).ok_or(ApplyError::NothingChosen)?;
        // Which dives get photos depends on the offsets, not known yet: every
        // profile is read. One that does not read fails "Apply" only if it
        // is needed, as before.
        let mut profiles = std::collections::HashMap::new();
        for dive in dives.iter().filter(|d| d.has_profile) {
            if replaced() {
                return Err(ApplyError::NoPhotos);
            }
            profiles.insert(dive.id.clone(), source.profile(&dive.id));
        }
        // Lightroom keeps the photos of its last search only. Checked and
        // sent under the search lock: a newer search cannot start in between
        // and reach Lightroom first.
        let (reply, answer) = mpsc::channel();
        {
            let searches = state.searches.lock().expect("collections");
            if !searches.is_latest(number) {
                return Err(ApplyError::NoPhotos);
            }
            send(
                &app,
                Command::FindPhotos {
                    from: from.clone(),
                    to: to.clone(),
                    reply,
                },
            )?;
        }
        let found = answer.recv().map_err(|_| ApplyError::LinkLost)??;
        let offsets = Offsets::load(&config_dir(&app), info.app);
        let session = Session::new(found.search, dives, found.photos, &offsets)
            .read_from(log.clone(), profiles);
        let period = session.period(number, from, to);
        // Checked again once done: the log may have changed, or the user may
        // have gone back and started a newer search, during this one.
        keep_search(&state.searches, number, &log, &source_info(&app), session)?;
        Ok(period)
    })
    .await
    .expect("photo search task failed")
}

/// Live counts of photo search `search` for the current choices.
#[tauri::command]
pub(crate) fn collections_preview(
    state: State<CollectionsState>,
    search: u64,
    choices: Choices,
) -> Result<Preview, ApplyError> {
    let searches = state.searches.lock().expect("collections");
    let session = searches.get(search).ok_or(ApplyError::NoPhotos)?;
    Ok(session.preview(&choices))
}

/// Suggested clock offset of the given cameras of photo search `search`
/// (`None`: no offset fits).
#[tauri::command]
pub(crate) async fn suggest_offsets(
    app: AppHandle,
    search: u64,
    cameras: Vec<String>,
) -> Result<std::collections::HashMap<String, Option<i32>>, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<CollectionsState>();
        let searches = state.searches.lock().expect("collections");
        let session = searches.get(search).ok_or(ApplyError::NoPhotos)?;
        Ok(session.suggest(&cameras))
    })
    .await
    .expect("suggestion task failed")
}

/// What "Create Collections › Apply" wrote.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CollectionsResult {
    /// The collections written: the groups that have a name.
    collections: Vec<WrittenCollection>,
    photos: usize,
    with_depth: usize,
    /// Chosen dives that got no photo (no collection).
    empty_dives: Vec<String>,
}

/// Writes the collections, dive data and per-photo values in one Lightroom
/// Undo step, then remembers each camera's offset (also when only the GPS
/// and altitude could not be written: the offsets are on the photos).
/// Writes the dives and profiles read with photo search `search`, the one
/// the screen shows; refused when it is no longer kept, or when `log`, the
/// dive log the screen shows, is not theirs or no longer the configured one.
/// `options`, what the diver ticked, and `other_site`, a site of the log for
/// every photo instead of each dive's own, are both absent for today's
/// screen: everything is written and the altitude follows the setting.
#[tauri::command]
pub(crate) async fn apply_collections(
    app: AppHandle,
    log: LogId,
    search: u64,
    choices: Choices,
    options: Option<WriteOptions>,
    other_site: Option<LocationChoice>,
) -> Result<CollectionsResult, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        let writes = app.state::<Writes>();
        let (_writing, info) = writes.begin(&log, || source_info(&app))?;
        let units = units_in_use(&app, &info);
        // Another site alone means everything ticked, as today's screen.
        let chosen = chosen_options(options, other_site.as_ref());
        let (request, preview, applied, altitude_setting) = {
            let state = app.state::<CollectionsState>();
            let searches = state.searches.lock().expect("collections");
            let session = searches.get(search).ok_or(ApplyError::NoPhotos)?;
            let mut request = session
                .write_request(&log, &choices, units)?
                .map_err(|detail| ApplyError::Source { detail })?;
            let settings = Settings::load(&config_dir(&app));
            request.parent = settings.collections.parent.clone();
            request.group = settings.collections.group_name();
            // Today's path copies the altitude here; with options or another
            // site it is decided below, with the rest.
            if chosen.is_none() && settings.depth_to_altitude {
                request.copy_depth_to_altitude();
            }
            (
                request,
                session.preview(&choices),
                session.applied(&choices),
                settings.depth_to_altitude,
            )
        };
        let other = match &chosen {
            Some(options) => other_location(other_site.as_ref(), options, || {
                open(&app, &info).and_then(|opened| opened.log.sites())
            })?,
            None => None,
        };
        let Planned {
            request,
            empty_dives,
            with_depth,
        } = planned(request, &preview, chosen.as_ref(), other.as_ref(), || {
            altitude_setting
        })?;
        let written = write_and_remember(
            &app,
            &info,
            |reply| Command::ApplyCollections { request, reply },
            &applied,
            |outcome| log::warn!("collections refused by Lightroom: {outcome:?}"),
        )
        .inspect_err(|e| {
            // `ask` (inside) fails only with these two; a Lightroom
            // refusal is logged by `on_refused` above.
            if matches!(e, ApplyError::NotConnected | ApplyError::LinkLost) {
                log::warn!("collections not written: {e:?}");
            }
        })?;
        let photos: usize = written.iter().map(|c| c.count).sum();
        let collections: Vec<WrittenCollection> = written
            .into_iter()
            .filter(|c| c.collection.is_some())
            .collect();
        log::info!(
            "collections written: {} collections, {photos} photos ({with_depth} with depth)",
            collections.len(),
        );
        Ok(CollectionsResult {
            photos,
            collections,
            with_depth,
            empty_dives,
        })
    })
    .await
    .expect("collections task failed")
}

/// The options in force: those sent, else everything ticked when another
/// site is sent alone, else none (today's path: nothing sent).
fn chosen_options(
    options: Option<WriteOptions>,
    other_site: Option<&LocationChoice>,
) -> Option<WriteOptions> {
    options.or_else(|| other_site.map(|_| WriteOptions::EVERYTHING))
}

/// The location of another site of the log for every group, when the diver
/// picked one (`other_site`) and kept its place or GPS; `None` for each
/// dive's own site, already on its group, and when neither is kept: nothing
/// of the site would be written, so it is neither checked nor looked for.
/// `sites` reads the log's sites, only when needed.
fn other_location(
    other_site: Option<&LocationChoice>,
    options: &WriteOptions,
    sites: impl FnOnce() -> Result<Vec<Site>, Error>,
) -> Result<Option<LocationTags>, ApplyError> {
    match other_site {
        Some(LocationChoice::Site { site_id, lat, lon }) if options.place || options.gps => {
            Ok(Some(another_site(site_id, *lat, *lon, sites)?))
        }
        Some(LocationChoice::Site { .. } | LocationChoice::DiveSite) | None => Ok(None),
    }
}

/// What "Apply" sends, and what its result tells besides Lightroom's answer.
#[derive(Debug)]
struct Planned {
    request: CollectionsRequest,
    /// Chosen dives that got no photo (no collection).
    empty_dives: Vec<String>,
    /// Photos with per-photo values and a depth (0 with the dive details
    /// unticked).
    with_depth: usize,
}

/// The write of `request`, built as today's screen sends it, `preview` being
/// the live counts of the same choices. The chosen dives without photo are
/// read from the counts first, before anything is removed: a dive whose
/// group the options drop, nothing being left to write on it, is not
/// "without photo". Then what the diver ticked (`chosen`, `other` the
/// location of another site, `setting` the altitude setting) is applied
/// (`with_options`; none: today's path). Refused when nothing is left to
/// write.
fn planned(
    mut request: CollectionsRequest,
    preview: &Preview,
    chosen: Option<&WriteOptions>,
    other: Option<&LocationTags>,
    setting: impl FnOnce() -> bool,
) -> Result<Planned, ApplyError> {
    let empty_dives = preview
        .dives
        .iter()
        .filter(|d| d.photos == 0)
        .map(|d| d.id.clone())
        .collect();
    if let Some(options) = chosen {
        with_options(&mut request, options, other, setting)?;
    }
    if request.is_empty() {
        return Err(ApplyError::NothingToWrite);
    }
    Ok(Planned {
        with_depth: request.with_depth(),
        request,
        empty_dives,
    })
}

/// What the diver ticked, applied to the request built as today: another
/// site on every group, what is not ticked removed, then the depth copied to
/// the altitude (only with the dive details; `setting` decides when the
/// options do not say). Refused when nothing is left to write.
fn with_options(
    request: &mut CollectionsRequest,
    options: &WriteOptions,
    other: Option<&LocationTags>,
    setting: impl FnOnce() -> bool,
) -> Result<(), ApplyError> {
    request.keep(options.kept(), other);
    if request.is_empty() {
        return Err(ApplyError::NothingToWrite);
    }
    if options.copies_altitude(setting) {
        request.copy_depth_to_altitude();
    }
    Ok(())
}

/// The catalog's collection sets (Settings › Collections).
#[tauri::command]
pub(crate) async fn collection_sets(app: AppHandle) -> Result<Vec<CollectionSet>, ApplyError> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(ask(&app, |reply| Command::ListSets { reply })??)
    })
    .await
    .expect("collection sets task failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::options::WriteOptions;
    use divelens_core::profile::PhotoPoint;
    use divelens_core::tags::{
        CollectionGroup, CollectionsRequest, DiveTags, LocationTags, PhotoTags, Units,
    };
    use divelens_core::{Dive, Site};

    fn dive(id: &str, site: &str) -> Dive {
        let start = chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(7, 14, 0)
            .unwrap();
        Dive {
            id: id.into(),
            number: Some(700),
            start,
            end: start + chrono::Duration::seconds(2100),
            duration_s: 2100,
            max_depth_m: Some(31.2),
            temp_high_c: Some(28.0),
            temp_low_c: None,
            rating: None,
            site: Site {
                id: Some(site.into()),
                name: Some(site.into()),
                location: None,
                country: Some("Atlantis".into()),
                lat: Some(12.5),
                lon: Some(-45.25),
            },
            has_profile: true,
            invalid: false,
            notes: None,
        }
    }

    /// Two groups, each with a photo taken at 12 m.
    fn request() -> CollectionsRequest {
        let point = PhotoPoint {
            elapsed_s: 160,
            depth_m: 12.0,
            temp_c: Some(25.0),
        };
        let group = |id: &str, site: &str| {
            let dive = dive(id, site);
            CollectionGroup {
                collection: Some(format!("700 - {site}")),
                dive: Some(DiveTags::new(&dive, Units::default(), "Subsurface")),
                location: Some(LocationTags::new(&dive.site, None, None)),
                photos: vec![PhotoTags::new(
                    format!("photo {id}"),
                    Some(point),
                    0,
                    Units::default(),
                )],
            }
        };
        CollectionsRequest {
            search: 3,
            parent: Vec::new(),
            group: None,
            groups: vec![group("A", "Coral Garden"), group("B", "Blue Hole")],
        }
    }

    fn options(
        dive: bool,
        altitude: Option<bool>,
        place: bool,
        gps: bool,
        collections: bool,
    ) -> WriteOptions {
        WriteOptions {
            dive,
            altitude,
            place,
            gps,
            collections,
        }
    }

    #[test]
    fn all_options_and_the_setting_give_todays_request() {
        let mut expected = request();
        expected.copy_depth_to_altitude();
        let mut got = request();
        with_options(
            &mut got,
            &options(true, None, true, true, true),
            None,
            || true,
        )
        .unwrap();
        assert_eq!(got, expected);
        // Setting off: no altitude, as today.
        let mut got = request();
        with_options(
            &mut got,
            &options(true, None, true, true, true),
            None,
            || false,
        )
        .unwrap();
        assert_eq!(got, request());
    }

    #[test]
    fn another_site_goes_on_every_group() {
        let other = LocationTags::new(&dive("C", "Shark Reef").site, None, None);
        let mut got = request();
        with_options(
            &mut got,
            &options(true, Some(false), true, true, true),
            Some(&other),
            || panic!("setting read"),
        )
        .unwrap();
        assert!(
            got.groups
                .iter()
                .all(|g| g.location.as_ref() == Some(&other))
        );
        // The collection names stay those of each dive.
        assert_eq!(
            got.groups[0].collection.as_deref(),
            Some("700 - Coral Garden")
        );
        assert_eq!(got.groups[1].collection.as_deref(), Some("700 - Blue Hole"));
    }

    #[test]
    fn nothing_ticked_is_refused() {
        let mut got = request();
        let none = options(false, Some(true), false, false, false);
        let refused = with_options(&mut got, &none, None, || panic!("setting read"));
        assert!(matches!(refused, Err(ApplyError::NothingToWrite)));
    }

    #[test]
    fn another_site_alone_means_everything_ticked() {
        let diving = Some(LocationChoice::DiveSite);
        assert!(chosen_options(None, None).is_none());
        let chosen = chosen_options(None, diving.as_ref()).expect("options");
        let mut got = request();
        let other = LocationTags::new(&dive("C", "Shark Reef").site, None, None);
        with_options(&mut got, &chosen, Some(&other), || true).unwrap();
        // Everything kept, the other site everywhere, the altitude from the setting.
        let mut expected = request();
        expected
            .groups
            .iter_mut()
            .for_each(|g| g.location = Some(other.clone()));
        expected.copy_depth_to_altitude();
        assert_eq!(got, expected);
        // Options sent win.
        let sent = options(true, None, false, false, false);
        assert_eq!(
            chosen_options(Some(sent), diving.as_ref()).map(|o| o.place),
            Some(false)
        );
    }

    #[test]
    fn another_site_without_place_or_gps_reads_no_site() {
        let unknown = LocationChoice::Site {
            site_id: "Nowhere Reef".into(),
            lat: None,
            lon: None,
        };
        // Neither the place nor the GPS ticked: nothing of the site would be
        // written, so the log is not read and the site not looked for.
        let collections_only = options(false, None, false, false, true);
        let other = other_location(Some(&unknown), &collections_only, || panic!("sites read"));
        assert!(matches!(other, Ok(None)), "{other:?}");
        // The collections still go, with nothing else.
        let mut got = request();
        with_options(&mut got, &collections_only, None, || panic!("setting read")).unwrap();
        assert_eq!(got.groups.len(), 2);
        assert!(
            got.groups
                .iter()
                .all(|g| g.collection.is_some() && g.location.is_none() && g.dive.is_none())
        );
        // The place or the GPS ticked: the site is looked for, and refused
        // when the log has no such site.
        for (place, gps) in [(true, false), (false, true), (true, true)] {
            let ticked = options(false, None, place, gps, true);
            let other = other_location(Some(&unknown), &ticked, || {
                Ok(vec![dive("C", "Shark Reef").site])
            });
            assert!(matches!(other, Err(ApplyError::UnknownSite)), "{other:?}");
        }
    }

    /// Dive A at 07:00 on a named site and dive B at 10:00 on none, each
    /// with a profile and one photo taken during it, and dive C at 14:00
    /// with no photo: the request for everything ticked and its live counts.
    fn three_dives() -> (CollectionsRequest, Preview) {
        use crate::apply::offsets::Offsets;
        use crate::lightroom::FoundPhoto;
        use divelens_core::profile::Sample;
        let at = |id: &str, site: Option<&str>, hour: u32| {
            let mut d = dive(id, site.unwrap_or_default());
            d.start = d.start.date().and_hms_opt(hour, 0, 0).unwrap();
            d.end = d.start + chrono::Duration::seconds(2100);
            if site.is_none() {
                d.site = Site::default();
            }
            d
        };
        let dives = vec![
            at("A", Some("Coral Garden"), 7),
            at("B", None, 10),
            at("C", Some("Blue Hole"), 14),
        ];
        let photo = |id: &str, time: &str| FoundPhoto {
            id: id.into(),
            time: Some(time.into()),
            model: Some("GoPro".into()),
            serial: None,
        };
        let found = vec![
            photo("1", "2025-12-31T07:20:00"),
            photo("2", "2025-12-31T10:10:00"),
        ];
        let session = Session::new(3, dives, found, &Offsets::default());
        let choices = Choices::default();
        let ramp = |_: &Dive| {
            Ok(vec![
                Sample {
                    time_s: 0,
                    depth_m: 0.0,
                    temp_c: None,
                },
                Sample {
                    time_s: 2100,
                    depth_m: 21.0,
                    temp_c: None,
                },
            ])
        };
        let request = session
            .request(&choices, Units::default(), "Subsurface", ramp)
            .unwrap();
        (request, session.preview(&choices))
    }

    #[test]
    fn a_group_left_with_nothing_is_not_an_empty_dive() {
        // Everything ticked: both dives with a photo written, with depth;
        // dive C without photo.
        let (request, preview) = three_dives();
        let all = options(true, Some(false), true, true, true);
        let written = planned(request, &preview, Some(&all), None, || {
            panic!("setting read")
        });
        let written = written.unwrap();
        assert_eq!(written.request.groups.len(), 2);
        assert_eq!(written.with_depth, 2);
        assert_eq!(written.empty_dives, ["C"]);
        // The site details alone: dive B has no site, nothing is left to
        // write on its photo and its group goes. It still had a photo: it
        // is not a dive without photo; dive C still is.
        let (request, preview) = three_dives();
        let site_only = options(false, Some(false), true, true, false);
        let written = planned(request, &preview, Some(&site_only), None, || {
            panic!("setting read")
        })
        .unwrap();
        let locations: Vec<_> = written
            .request
            .groups
            .iter()
            .map(|g| g.location.as_ref().and_then(|l| l.location.as_deref()))
            .collect();
        assert_eq!(locations, [Some("Coral Garden")]);
        assert_eq!(written.empty_dives, ["C"]);
        // The dive details unticked: no photo with depth.
        assert_eq!(written.with_depth, 0);
        // Today's path (no option): the same dives without photo.
        let (request, preview) = three_dives();
        let today = planned(request, &preview, None, None, || panic!("setting read")).unwrap();
        assert_eq!(today.empty_dives, ["C"]);
        assert_eq!(today.request.groups.len(), 2);
    }

    #[test]
    fn another_site_without_a_position_and_only_gps_ticked_is_refused() {
        let mut bare = dive("C", "Shark Reef").site;
        bare.lat = None;
        bare.lon = None;
        let other = LocationTags::new(&bare, None, None);
        let mut got = request();
        let only_gps = options(false, None, false, true, false);
        let refused = with_options(&mut got, &only_gps, Some(&other), || true);
        assert!(matches!(refused, Err(ApplyError::NothingToWrite)));
    }
}
