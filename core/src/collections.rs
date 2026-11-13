//! "Create Collections": which dive each photo belongs to, the clock offset
//! to suggest for a camera, and the values written on each photo.
//!
//! Spec F5/F6. A photo's reference time is its original capture time, a
//! wall-clock time like the dives'. The camera's clock offset is added to it
//! to get the dive computer's time ("corrected time"):
//!
//! - start ≤ corrected ≤ end: the photo is **in the dive** and gets
//!   everything, per-photo values included;
//! - within the margin before the start or after the end: the photo is **at
//!   the surface**; it joins the dive's collection and gets the dive data but
//!   no per-photo values;
//! - a photo that could go to two dives goes to the nearest one.

use chrono::{Duration, NaiveDateTime};

use crate::model::Dive;
use crate::profile::{self, Sample};
use crate::tags::{PhotoTags, Units};

/// Clock offsets are whole minutes, at most ±24 h (as in the previous plugin).
pub const MAX_OFFSET_MIN: i32 = 24 * 60;
/// "Suggest" tries every quarter of an hour between −14 h and +14 h, which
/// covers every time zone.
pub const SUGGEST_STEP_MIN: i32 = 15;
pub const SUGGEST_RANGE_MIN: i32 = 14 * 60;

/// Start and end of one dive, wall-clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

impl From<&Dive> for Span {
    fn from(d: &Dive) -> Self {
        Self {
            start: d.start,
            end: d.end,
        }
    }
}

/// Where a photo goes; the index is the dive's position in the slice given
/// to [`place`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    InDive(usize),
    Surface(usize),
}

impl Placement {
    pub fn dive(self) -> usize {
        match self {
            Placement::InDive(i) | Placement::Surface(i) => i,
        }
    }
}

/// Camera time corrected by the clock offset.
pub fn corrected(time: NaiveDateTime, offset_min: i32) -> NaiveDateTime {
    time + Duration::minutes(offset_min.into())
}

/// The dive a photo taken at `corrected` (dive computer time) belongs to, or
/// `None`. `margin_min` widens each dive at both ends for surface photos.
/// Ties go to the first dive of the slice.
pub fn place(dives: &[Span], corrected: NaiveDateTime, margin_min: u32) -> Option<Placement> {
    let margin = Duration::minutes(margin_min.into());
    let mut best: Option<(Duration, usize)> = None;
    for (i, d) in dives.iter().enumerate() {
        let distance = if corrected < d.start {
            d.start - corrected
        } else if corrected > d.end {
            corrected - d.end
        } else {
            Duration::zero()
        };
        if distance > margin {
            continue;
        }
        if best.is_none_or(|(b, _)| distance < b) {
            best = Some((distance, i));
        }
    }
    best.map(|(distance, i)| {
        if distance.is_zero() {
            Placement::InDive(i)
        } else {
            Placement::Surface(i)
        }
    })
}

/// Number of photos inside a dive (margin excluded) with this offset.
fn in_dives(dives: &[Span], times: &[NaiveDateTime], offset_min: i32) -> usize {
    times
        .iter()
        .filter(|t| {
            matches!(
                place(dives, corrected(**t, offset_min), 0),
                Some(Placement::InDive(_))
            )
        })
        .count()
}

/// How "round" an offset is: whole hours first, then half hours (time zones).
fn roundness(offset_min: i32) -> u8 {
    if offset_min % 60 == 0 {
        0
    } else if offset_min % 30 == 0 {
        1
    } else {
        2
    }
}

/// The offset that puts the most photos of one camera inside the dives,
/// trying every quarter of an hour between −14 h and +14 h. Among equally
/// good offsets: a whole hour, then a half hour, then the one closest to
/// zero. `None` when no offset puts any photo in a dive.
pub fn suggest_offset(dives: &[Span], times: &[NaiveDateTime]) -> Option<i32> {
    let mut best: Option<(usize, i32)> = None;
    let mut offset = -SUGGEST_RANGE_MIN;
    while offset <= SUGGEST_RANGE_MIN {
        let n = in_dives(dives, times, offset);
        let better = match best {
            None => n > 0,
            Some((b, o)) => {
                n > b || (n == b && (roundness(offset), offset.abs()) < (roundness(o), o.abs()))
            }
        };
        if better {
            best = Some((n, offset));
        }
        offset += SUGGEST_STEP_MIN;
    }
    best.map(|(_, o)| o)
}

/// Values written on one photo, from its dive's profile. A surface photo
/// gets only the clock offset: its per-photo fields are cleared.
pub fn photo_tags(
    id: String,
    dive: &Dive,
    profile: &[Sample],
    corrected: NaiveDateTime,
    placement: Placement,
    offset_min: i32,
    units: Units,
) -> PhotoTags {
    let point = match placement {
        Placement::InDive(_) => {
            let elapsed = (corrected - dive.start).num_seconds().max(0);
            profile::at(profile, u32::try_from(elapsed).unwrap_or(u32::MAX))
        }
        Placement::Surface(_) => None,
    };
    PhotoTags::new(id, point, offset_min, units)
}

/// Name of a dive's collection: "<number> - <site>", as the previous plugin
/// named them. A dive without a number or a site uses what it has; one with
/// neither is named after its start.
pub fn collection_name(dive: &Dive) -> String {
    match (dive.number, dive.site.name.as_deref()) {
        (Some(n), Some(site)) => format!("{n} - {site}"),
        (Some(n), None) => n.to_string(),
        (None, Some(site)) => format!("{} - {site}", dive.start.format("%Y-%m-%d %H:%M")),
        (None, None) => dive.start.format("%Y-%m-%d %H:%M").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Site;
    use chrono::NaiveDate;

    fn at(d: u32, h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2025, 12, d)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    fn span(d: u32, h: u32, m: u32, minutes: i64) -> Span {
        let start = at(d, h, m);
        Span {
            start,
            end: start + Duration::minutes(minutes),
        }
    }

    #[test]
    fn photo_in_dive_or_at_the_surface() {
        let dives = [span(30, 9, 0, 50)];
        assert_eq!(place(&dives, at(30, 9, 0), 0), Some(Placement::InDive(0)));
        assert_eq!(place(&dives, at(30, 9, 50), 0), Some(Placement::InDive(0)));
        assert_eq!(place(&dives, at(30, 9, 51), 0), None);
        assert_eq!(
            place(&dives, at(30, 8, 55), 10),
            Some(Placement::Surface(0))
        );
        assert_eq!(
            place(&dives, at(30, 10, 0), 10),
            Some(Placement::Surface(0))
        );
        assert_eq!(place(&dives, at(30, 10, 1), 10), None);
    }

    #[test]
    fn photo_between_two_dives_goes_to_the_nearest() {
        // Dive 0 ends 10:00, dive 1 starts 10:40; 20 min margin overlaps.
        let dives = [span(30, 9, 10, 50), span(30, 10, 40, 50)];
        assert_eq!(
            place(&dives, at(30, 10, 15), 20),
            Some(Placement::Surface(0))
        );
        assert_eq!(
            place(&dives, at(30, 10, 25), 20),
            Some(Placement::Surface(1))
        );
        // Inside a dive beats the margin of another.
        let close = [span(30, 9, 0, 30), span(30, 9, 35, 30)];
        assert_eq!(place(&close, at(30, 9, 36), 20), Some(Placement::InDive(1)));
    }

    #[test]
    fn suggests_the_time_zone_difference() {
        // Spec F5: camera on French time, dives in the Maldives (+4 h).
        let dives = [span(30, 9, 0, 50), span(30, 14, 0, 45), span(31, 8, 30, 55)];
        let camera: Vec<_> = [
            (30, 5, 2),
            (30, 5, 20),
            (30, 5, 48),
            (30, 10, 3),
            (30, 10, 40),
            (31, 4, 33),
            (31, 5, 20),
        ]
        .iter()
        .map(|&(d, h, m)| at(d, h, m))
        .collect();
        assert_eq!(suggest_offset(&dives, &camera), Some(240));
        // A camera already on dive time: no change.
        let right: Vec<_> = camera.iter().map(|t| corrected(*t, 240)).collect();
        assert_eq!(suggest_offset(&dives, &right), Some(0));
    }

    #[test]
    fn suggestion_prefers_round_offsets_then_small_ones() {
        // One photo in the middle of a long dive: many offsets fit.
        let dives = [span(30, 9, 0, 120)];
        assert_eq!(suggest_offset(&dives, &[at(30, 10, 0)]), Some(0));
        // Fits from 6 h 15 to 6 h 45: no whole hour, the half hour wins.
        let dives = [span(30, 9, 0, 30)];
        assert_eq!(suggest_offset(&dives, &[at(30, 2, 45)]), Some(390));
    }

    #[test]
    fn no_suggestion_when_nothing_fits() {
        let dives = [span(30, 9, 0, 50)];
        assert_eq!(suggest_offset(&dives, &[at(1, 9, 0)]), None);
        assert_eq!(suggest_offset(&dives, &[]), None);
    }

    fn dive() -> Dive {
        let start = at(30, 9, 0);
        Dive {
            id: "A".into(),
            number: Some(700),
            start,
            end: start + Duration::minutes(50),
            duration_s: 3000,
            max_depth_m: Some(20.0),
            temp_high_c: None,
            temp_low_c: None,
            rating: None,
            site: Site {
                name: Some("Coral Garden".into()),
                ..Site::default()
            },
            has_profile: true,
            invalid: false,
            notes: None,
        }
    }

    fn sample(time_s: u32, depth_m: f64, temp_c: Option<f64>) -> Sample {
        Sample {
            time_s,
            depth_m,
            temp_c,
        }
    }

    #[test]
    fn per_photo_values_in_dive_and_at_the_surface() {
        let profile = [sample(10, 2.0, Some(26.0)), sample(310, 18.0, Some(24.0))];
        let t = at(30, 9, 2) + Duration::seconds(40); // 160 s into the dive
        let tags = photo_tags(
            "p1".into(),
            &dive(),
            &profile,
            t,
            Placement::InDive(0),
            241,
            Units::default(),
        );
        assert_eq!(tags.id, "p1");
        assert_eq!(tags.photo_depth.as_deref(), Some("10.0 m"));
        assert_eq!(tags.photo_temp.as_deref(), Some("25.0°C"));
        assert_eq!(tags.photo_elapsed.as_deref(), Some("02:40"));
        assert_eq!(tags.clock_offset, "+04:01");

        let surface = photo_tags(
            "p2".into(),
            &dive(),
            &profile,
            at(30, 8, 55),
            Placement::Surface(0),
            -30,
            Units::default(),
        );
        assert_eq!(
            (
                surface.photo_depth,
                surface.photo_temp,
                surface.photo_elapsed
            ),
            (None, None, None)
        );
        assert_eq!(surface.clock_offset, "-00:30");

        // No profile: fields stay empty.
        let none = photo_tags(
            "p3".into(),
            &dive(),
            &[],
            t,
            Placement::InDive(0),
            0,
            Units::default(),
        );
        assert_eq!((none.photo_depth, none.photo_elapsed), (None, None));
    }

    #[test]
    fn collection_names() {
        let mut d = dive();
        assert_eq!(collection_name(&d), "700 - Coral Garden");
        d.site.name = None;
        assert_eq!(collection_name(&d), "700");
        d.number = None;
        assert_eq!(collection_name(&d), "2025-12-30 09:00");
        d.site.name = Some("Coral Garden".into());
        assert_eq!(collection_name(&d), "2025-12-30 09:00 - Coral Garden");
    }

    #[test]
    fn offsets_are_formatted_with_their_sign() {
        assert_eq!(crate::tags::clock_offset(0), "+00:00");
        assert_eq!(crate::tags::clock_offset(241), "+04:01");
        assert_eq!(crate::tags::clock_offset(-30), "-00:30");
        assert_eq!(crate::tags::clock_offset(-MAX_OFFSET_MIN), "-24:00");
    }
}
