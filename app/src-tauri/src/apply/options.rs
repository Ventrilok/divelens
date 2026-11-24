//! What the diver ticked to write when tagging the selection: the options a
//! screen may send. Without them, the current screens' behaviour holds.

use divelens_core::tags::Kept;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WriteOptions {
    pub dive: bool,
    /// `None`: the "depth copied to altitude" setting decides (current screens).
    pub altitude: Option<bool>,
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

    /// True only when the dive details are written; `setting` is read only
    /// when `altitude` is `None`.
    pub(crate) fn copies_altitude(&self, setting: impl FnOnce() -> bool) -> bool {
        self.dive && self.altitude.unwrap_or_else(setting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(dive: bool, altitude: Option<bool>) -> WriteOptions {
        WriteOptions {
            dive,
            altitude,
            place: true,
            gps: true,
            collections: true,
        }
    }

    #[test]
    fn the_altitude_follows_the_setting_only_when_unset() {
        assert!(options(true, None).copies_altitude(|| true));
        assert!(!options(true, None).copies_altitude(|| false));
        assert!(!options(true, Some(false)).copies_altitude(|| panic!("setting read")));
        assert!(options(true, Some(true)).copies_altitude(|| panic!("setting read")));
        assert!(!options(false, None).copies_altitude(|| panic!("setting read")));
        assert!(!options(false, Some(true)).copies_altitude(|| panic!("setting read")));
    }
}
