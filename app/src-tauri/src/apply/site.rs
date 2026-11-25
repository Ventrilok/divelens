//! Another site of the dive log for the photos (both paths of the journey
//! offer it): checked and found in one place, so both refuse in the same
//! order.

use divelens_core::tags::{LocationTags, valid_gps};
use divelens_core::{Error, Site};

use super::errors::ApplyError;

/// A position edited on the screen is whole and valid, or absent: half a
/// position, or one off the globe, is refused.
pub(crate) fn check_gps(lat: Option<f64>, lon: Option<f64>) -> Result<(), ApplyError> {
    match (lat, lon) {
        (Some(a), Some(b)) if !valid_gps(a, b) => Err(ApplyError::BadGps),
        (Some(_), None) | (None, Some(_)) => Err(ApplyError::BadGps),
        _ => Ok(()),
    }
}

/// The site `site_id` names (the first, should two share it).
pub(crate) fn find_site(sites: Vec<Site>, site_id: &str) -> Result<Site, ApplyError> {
    sites
        .into_iter()
        .find(|s| s.id.as_deref() == Some(site_id))
        .ok_or(ApplyError::UnknownSite)
}

/// The location of another site of the log, its position possibly edited.
/// The position is checked first, then `sites` reads the log's sites, then
/// the site is looked up: the first refusal wins.
pub(crate) fn another_site(
    site_id: &str,
    lat: Option<f64>,
    lon: Option<f64>,
    sites: impl FnOnce() -> Result<Vec<Site>, Error>,
) -> Result<LocationTags, ApplyError> {
    check_gps(lat, lon)?;
    let sites = sites().map_err(|detail| ApplyError::Source { detail })?;
    let site = find_site(sites, site_id)?;
    Ok(LocationTags::new(&site, lat, lon))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn half_a_gps_position_is_refused() {
        assert!(matches!(
            check_gps(Some(12.5), None),
            Err(ApplyError::BadGps)
        ));
        assert!(matches!(
            check_gps(None, Some(-45.25)),
            Err(ApplyError::BadGps)
        ));
        assert!(matches!(
            check_gps(Some(91.0), Some(0.0)),
            Err(ApplyError::BadGps)
        ));
        assert!(check_gps(Some(12.5), Some(-45.25)).is_ok());
        assert!(check_gps(None, None).is_ok());
    }

    #[test]
    fn unknown_site_is_refused() {
        assert!(matches!(
            find_site(vec![], "x"),
            Err(ApplyError::UnknownSite)
        ));
        // The one the id names; the first when two share it.
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
    fn the_first_refusal_wins() {
        // A bad position is refused before the log is read.
        let refused = another_site("s1", Some(12.5), None, || panic!("sites read"));
        assert!(matches!(refused, Err(ApplyError::BadGps)));
        let unread = another_site("s1", None, None, || Err(Error::BadProfile("x".into())));
        assert!(matches!(unread, Err(ApplyError::Source { .. })));
        let unknown = another_site("x", None, None, || Ok(vec![site("s1", "Coral Garden")]));
        assert!(matches!(unknown, Err(ApplyError::UnknownSite)));
        let found = another_site("s1", Some(1.0), Some(2.0), || {
            Ok(vec![site("s1", "Coral Garden")])
        });
        assert_eq!(found.unwrap().lat, Some(1.0));
    }
}
