//! The reference log: ten fictional dives and their sites, with the values
//! every reader must give for them. Each reader's tests write this log in the
//! application's own format; the conformance tests compare what the reader
//! gives with the values below, the only place they are written.
//!
//! Described for contributors, case by case, in
//! `docs/contributing/reference-log.md`: keep both in step.
//!
//! Values are what the dive application shows, in metric units. Times are
//! wall-clock times, exactly as the log holds them.

/// A dive site as the application shows it.
pub(crate) struct Site {
    pub name: Option<&'static str>,
    pub location: Option<&'static str>,
    pub country: Option<&'static str>,
    /// Latitude and longitude, decimal degrees.
    pub gps: Option<(f64, f64)>,
}

/// A dive as the application shows it.
pub(crate) struct Dive {
    /// Tells the dives of the reference log apart: only one has no number.
    pub number: Option<u32>,
    /// Start and end, wall clock, as the app receives them.
    pub start: &'static str,
    pub end: &'static str,
    pub duration_s: u32,
    pub max_depth_m: Option<f64>,
    pub temp_low_c: Option<f64>,
    pub temp_high_c: Option<f64>,
    pub rating: Option<u8>,
    pub site: Option<&'static Site>,
    pub notes: Option<&'static str>,
    /// Marked invalid, where the application has the notion (Subsurface).
    pub invalid: bool,
    /// Points the profile goes through: (seconds since the start, depth in
    /// metres, temperature in °C), at multiples of 10 s. Empty: the dive has
    /// no profile.
    pub profile: &'static [(u32, f64, f64)],
}

pub(crate) static CORAL_GARDEN: Site = Site {
    name: Some("Coral Garden"),
    location: Some("North Reef"),
    country: Some("Atlantis"),
    gps: Some((12.5, -45.25)),
};

pub(crate) static GLACIER_WALL: Site = Site {
    name: Some("Glacier Wall"),
    location: Some("Frostville"),
    country: Some("Atlantis"),
    gps: Some((-64.125, -60.5)),
};

/// No place, no country, no GPS; a "/" in its name.
pub(crate) static SHARK_POINT: Site = Site {
    name: Some("Shark Point / Bay"),
    location: None,
    country: None,
    gps: None,
};

/// A three-part name kept whole: the application has a country for it.
pub(crate) static TEMPLE: Site = Site {
    name: Some("Lemuria / Sunken City / Temple"),
    location: None,
    country: Some("Lemuria"),
    gps: Some((-8.25, 115.5)),
};

/// A place but no country and no GPS; two dives.
pub(crate) static PEARL_BANK: Site = Site {
    name: Some("Pearl Bank"),
    location: Some("Oyster Town"),
    country: None,
    gps: None,
};

/// No dive yet.
pub(crate) static BLUE_LAGOON: Site = Site {
    name: Some("Blue Lagoon"),
    location: None,
    country: None,
    gps: Some((1.25, 2.5)),
};

/// Accents and an apostrophe.
pub(crate) static EMERALD_REEF: Site = Site {
    name: Some("Récif d'Émeraude"),
    location: Some("Île Verte"),
    country: Some("Atlantide"),
    gps: Some((-20.5, 57.25)),
};

/// GPS only, no name: on its dive, not in the list of sites.
pub(crate) static GPS_ONLY: Site = Site {
    name: None,
    location: None,
    country: None,
    gps: Some((25.0, -77.5)),
};

/// Every named site of the log: what `sites()` lists.
pub(crate) static SITES: [&Site; 7] = [
    &CORAL_GARDEN,
    &GLACIER_WALL,
    &SHARK_POINT,
    &TEMPLE,
    &PEARL_BANK,
    &BLUE_LAGOON,
    &EMERALD_REEF,
];

pub(crate) static DIVES: [Dive; 10] = [
    // Night dive across midnight and the new year; a dive computer, no
    // profile recorded.
    Dive {
        number: Some(100),
        start: "2024-12-31T23:40:00",
        end: "2025-01-01T00:12:00",
        duration_s: 32 * 60,
        max_depth_m: Some(14.2),
        temp_low_c: Some(21.0),
        temp_high_c: Some(21.0),
        rating: Some(3),
        site: Some(&CORAL_GARDEN),
        notes: Some("Night dive, lobsters everywhere"),
        invalid: false,
        profile: &[],
    },
    // Everything filled in; notes with a line break and characters to escape.
    Dive {
        number: Some(101),
        start: "2025-06-14T09:30:00",
        end: "2025-06-14T10:12:30",
        duration_s: 42 * 60 + 30,
        max_depth_m: Some(18.4),
        temp_low_c: Some(24.0),
        temp_high_c: Some(26.0),
        rating: Some(4),
        site: Some(&CORAL_GARDEN),
        notes: Some("Turtle at the wall & a \"reef\" shark.\nSecond line"),
        invalid: false,
        profile: &[
            (10, 2.0, 26.0),
            (120, 10.0, 25.0),
            (300, 18.4, 24.0),
            (600, 18.0, 24.0),
            (1200, 15.0, 24.5),
            (1800, 10.0, 24.5),
            (2280, 5.0, 25.5),
            (2460, 5.0, 25.5),
            (2550, 0.5, 25.5),
            (2580, 0.0, 25.5),
        ],
    },
    // Two dive computers in Subsurface: the first one's values. Unrated.
    Dive {
        number: Some(102),
        start: "2025-06-14T14:00:00",
        end: "2025-06-14T14:35:00",
        duration_s: 35 * 60,
        max_depth_m: Some(12.0),
        temp_low_c: Some(22.0),
        temp_high_c: Some(23.0),
        rating: None,
        site: Some(&GLACIER_WALL),
        notes: None,
        invalid: false,
        profile: &[
            (20, 3.0, 23.0),
            (600, 12.0, 22.0),
            (1500, 9.0, 22.5),
            (2100, 0.0, 22.5),
        ],
    },
    // Entered by hand: no profile.
    Dive {
        number: Some(103),
        start: "2025-06-15T08:00:00",
        end: "2025-06-15T08:50:00",
        duration_s: 50 * 60,
        max_depth_m: Some(20.0),
        temp_low_c: Some(27.0),
        temp_high_c: Some(27.0),
        rating: Some(2),
        site: Some(&SHARK_POINT),
        notes: Some("Entered\nby hand"),
        invalid: false,
        profile: &[],
    },
    // No profile and no temperature.
    Dive {
        number: Some(104),
        start: "2025-06-15T11:00:00",
        end: "2025-06-15T11:30:00",
        duration_s: 30 * 60,
        max_depth_m: Some(15.0),
        temp_low_c: None,
        temp_high_c: None,
        rating: None,
        site: Some(&TEMPLE),
        notes: None,
        invalid: false,
        profile: &[],
    },
    // No number, no site, invalid; 0 °C and below.
    Dive {
        number: None,
        start: "2025-06-16T07:00:00",
        end: "2025-06-16T07:12:00",
        duration_s: 12 * 60,
        max_depth_m: Some(9.0),
        temp_low_c: Some(-1.5),
        temp_high_c: Some(0.0),
        rating: None,
        site: None,
        notes: None,
        invalid: true,
        profile: &[(10, 1.0, 0.0), (360, 9.0, -1.5), (720, 0.5, -1.5)],
    },
    // 106 and 107 start at the same second.
    Dive {
        number: Some(106),
        start: "2025-06-17T10:00:00",
        end: "2025-06-17T10:25:00",
        duration_s: 25 * 60,
        max_depth_m: Some(8.0),
        temp_low_c: Some(28.0),
        temp_high_c: Some(28.5),
        rating: None,
        site: Some(&PEARL_BANK),
        notes: None,
        invalid: false,
        profile: &[(20, 2.0, 28.5), (1500, 0.5, 28.5)],
    },
    Dive {
        number: Some(107),
        start: "2025-06-17T10:00:00",
        end: "2025-06-17T10:20:00",
        duration_s: 20 * 60,
        max_depth_m: Some(7.0),
        temp_low_c: Some(28.0),
        temp_high_c: Some(28.0),
        rating: Some(5),
        site: Some(&PEARL_BANK),
        notes: None,
        invalid: false,
        profile: &[(10, 2.0, 28.0), (1200, 0.5, 28.0)],
    },
    // Over an hour; accents in the site and the notes.
    Dive {
        number: Some(108),
        start: "2025-06-18T09:00:00",
        end: "2025-06-18T10:04:00",
        duration_s: 64 * 60,
        max_depth_m: Some(31.2),
        temp_low_c: Some(25.0),
        temp_high_c: Some(27.0),
        rating: Some(3),
        site: Some(&EMERALD_REEF),
        notes: Some("Manta ray at 30 m — café afterwards."),
        invalid: false,
        profile: &[
            (10, 3.0, 27.0),
            (300, 25.0, 26.0),
            (600, 31.2, 25.0),
            (1800, 22.0, 25.0),
            (3000, 12.0, 26.0),
            (3600, 5.0, 26.5),
            (3840, 0.0, 26.5),
        ],
    },
    // At a site known only by its GPS position.
    Dive {
        number: Some(109),
        start: "2025-06-18T15:30:00",
        end: "2025-06-18T15:54:00",
        duration_s: 24 * 60,
        max_depth_m: Some(16.0),
        temp_low_c: Some(25.0),
        temp_high_c: Some(26.0),
        rating: None,
        site: Some(&GPS_ONLY),
        notes: None,
        invalid: false,
        profile: &[
            (10, 2.0, 26.0),
            (480, 16.0, 25.0),
            (1200, 5.0, 25.0),
            (1440, 0.0, 25.5),
        ],
    },
];
