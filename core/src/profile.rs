//! Dive profile and per-photo values.
//!
//! Spec: depth, temperature and elapsed time at the moment a photo was taken,
//! linearly interpolated between two profile samples. Empty when the dive has
//! no profile or the photo falls outside the recorded profile.

use serde::{Deserialize, Serialize};

/// One profile sample, as recorded by the dive computer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    /// Seconds since the start of the dive.
    pub time_s: u32,
    /// Depth in metres (positive, below the surface).
    pub depth_m: f64,
    /// Water temperature in °C, when the sample carries one.
    pub temp_c: Option<f64>,
}

/// Values written on a photo taken during a dive.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhotoPoint {
    pub elapsed_s: u32,
    pub depth_m: f64,
    pub temp_c: Option<f64>,
}

/// Values at `elapsed_s` seconds into the dive, or `None` when the profile is
/// empty or the time lies outside it. `samples` must be sorted by time.
pub fn at(samples: &[Sample], elapsed_s: u32) -> Option<PhotoPoint> {
    let first = samples.first()?;
    let last = samples.last()?;
    if elapsed_s < first.time_s || elapsed_s > last.time_s {
        return None;
    }
    // Index of the first sample at or after `elapsed_s`.
    let i = samples.partition_point(|s| s.time_s < elapsed_s);
    let b = samples[i];
    if b.time_s == elapsed_s || i == 0 {
        return Some(PhotoPoint {
            elapsed_s,
            depth_m: b.depth_m,
            temp_c: b.temp_c,
        });
    }
    let a = samples[i - 1];
    let f = f64::from(elapsed_s - a.time_s) / f64::from(b.time_s - a.time_s);
    let lerp = |x: f64, y: f64| x + (y - x) * f;
    let temp_c = match (a.temp_c, b.temp_c) {
        (Some(x), Some(y)) => Some(lerp(x, y)),
        (x, y) => x.or(y),
    };
    Some(PhotoPoint {
        elapsed_s,
        depth_m: lerp(a.depth_m, b.depth_m),
        temp_c,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(time_s: u32, depth_m: f64, temp_c: Option<f64>) -> Sample {
        Sample {
            time_s,
            depth_m,
            temp_c,
        }
    }

    #[test]
    fn interpolates_between_samples() {
        let p = [s(0, 0.0, Some(26.0)), s(10, 10.0, Some(24.0))];
        let v = at(&p, 5).unwrap();
        assert!((v.depth_m - 5.0).abs() < 1e-9);
        assert!((v.temp_c.unwrap() - 25.0).abs() < 1e-9);
    }

    #[test]
    fn exact_sample_is_returned_as_is() {
        let p = [s(0, 0.0, None), s(10, 12.3, Some(22.0)), s(20, 15.0, None)];
        assert_eq!(at(&p, 10).unwrap().depth_m, 12.3);
    }

    #[test]
    fn missing_temperature_falls_back_to_neighbour() {
        let p = [s(0, 0.0, None), s(10, 10.0, Some(24.0))];
        assert_eq!(at(&p, 5).unwrap().temp_c, Some(24.0));
    }

    #[test]
    fn outside_profile_or_empty_is_none() {
        let p = [s(0, 0.0, None), s(10, 10.0, None)];
        assert_eq!(at(&p, 11), None);
        assert_eq!(at(&[], 0), None);
    }

    #[test]
    fn handles_negative_temperatures() {
        let p = [s(0, 0.0, Some(-1.0)), s(10, 10.0, Some(-1.8))];
        assert!((at(&p, 5).unwrap().temp_c.unwrap() + 1.4).abs() < 1e-9);
    }
}
