//! What the diver ticked to write: the options every write sends, on both
//! paths. They alone decide; the engine reads no setting for them.

use divelens_core::tags::Kept;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WriteOptions {
    pub dive: bool,
    /// Copy each photo's depth to its GPS altitude (with the dive details).
    pub altitude: bool,
    pub place: bool,
    pub gps: bool,
    pub collections: bool,
}

impl WriteOptions {
    pub(crate) fn kept(&self) -> Kept {
        Kept {
            dive: self.dive,
            place: self.place,
            gps: self.gps,
            collections: self.collections,
        }
    }

    /// True only when the dive details are written: the depth comes with them.
    pub(crate) fn copies_altitude(&self) -> bool {
        self.dive && self.altitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(dive: bool, altitude: bool) -> WriteOptions {
        WriteOptions {
            dive,
            altitude,
            place: true,
            gps: true,
            collections: true,
        }
    }

    #[test]
    fn the_altitude_is_copied_only_with_the_dive_details() {
        assert!(!options(true, false).copies_altitude());
        assert!(options(true, true).copies_altitude());
        assert!(!options(false, true).copies_altitude());
        assert!(!options(false, false).copies_altitude());
    }
}
