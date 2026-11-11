//! Decoding of a dive computer's raw download with libdivecomputer.
//!
//! Dive-log applications (MacDive, and others later) keep the bytes the dive
//! computer sent. libdivecomputer turns them into a profile, given the
//! computer model. Approach proven in DiverCity (GH #297): MacDive stores
//! libdivecomputer-ready bytes, except Shearwater logs, which are kept in
//! their transport compression and must be expanded first.

use std::ffi::{CStr, c_void};
use std::ptr;

use libdivecomputer_sys as dc;

use crate::profile::Sample;
use crate::{Error, Result};

/// A dive computer model known to libdivecomputer.
struct Descriptor {
    vendor: String,
    product: String,
    raw: *mut dc::dc_descriptor_t,
}

/// The list of dive computers libdivecomputer can decode.
pub struct Catalog {
    context: *mut dc::dc_context_t,
    descriptors: Vec<Descriptor>,
}

/// One computer model chosen from the [`Catalog`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Model(usize);

impl Catalog {
    pub fn new() -> Result<Self> {
        let mut context = ptr::null_mut();
        // SAFETY: plain out-parameter calls; every pointer obtained here is
        // released in `Drop`.
        unsafe {
            if dc::dc_context_new(&mut context) != dc::DC_STATUS_SUCCESS {
                return Err(Error::Decoder(
                    "cannot create libdivecomputer context".into(),
                ));
            }
            let mut catalog = Catalog {
                context,
                descriptors: Vec::new(),
            };
            let mut iterator = ptr::null_mut();
            if dc::dc_descriptor_iterator_new(&mut iterator, context) != dc::DC_STATUS_SUCCESS {
                return Err(Error::Decoder("cannot list dive computers".into()));
            }
            loop {
                let mut raw: *mut dc::dc_descriptor_t = ptr::null_mut();
                if dc::dc_iterator_next(iterator, &mut raw as *mut _ as *mut c_void)
                    != dc::DC_STATUS_SUCCESS
                {
                    break;
                }
                let text = |p: *const std::ffi::c_char| {
                    if p.is_null() {
                        String::new()
                    } else {
                        CStr::from_ptr(p).to_string_lossy().into_owned()
                    }
                };
                catalog.descriptors.push(Descriptor {
                    vendor: text(dc::dc_descriptor_get_vendor(raw)),
                    product: text(dc::dc_descriptor_get_product(raw)),
                    raw,
                });
            }
            dc::dc_iterator_free(iterator);
            Ok(catalog)
        }
    }

    /// Finds the computer model from the name a dive log records, such as
    /// "Shearwater Perdix 2" or "Mares Puck Pro (#001)". Exact match first,
    /// then the most specific model of the same brand containing the name (or
    /// contained in it). `None` for "No Computer", a brand alone, or unknown.
    pub fn find(&self, computer: &str) -> Option<Model> {
        // Drop a trailing "(…)" annotation such as a serial or "(ios)".
        let name = match computer.find('(') {
            Some(i) => &computer[..i],
            None => computer,
        };
        let name = name.trim().to_lowercase();
        let (vendor, model) = name.split_once(' ')?;
        let model = model.trim();
        if model.is_empty() {
            return None;
        }
        let same_vendor = || {
            self.descriptors
                .iter()
                .enumerate()
                .filter(move |(_, d)| d.vendor.to_lowercase() == vendor)
        };
        if let Some((i, _)) = same_vendor().find(|(_, d)| d.product.to_lowercase() == model) {
            return Some(Model(i));
        }
        same_vendor()
            .filter(|(_, d)| {
                let product = d.product.to_lowercase();
                product.contains(model) || model.contains(&product)
            })
            .max_by_key(|(_, d)| d.product.len())
            .map(|(i, _)| Model(i))
    }

    /// "Vendor Product" of a model, for logs and tests.
    pub fn name(&self, model: Model) -> String {
        let d = &self.descriptors[model.0];
        format!("{} {}", d.vendor, d.product)
    }

    /// Decodes a raw dive download into profile samples, sorted by time.
    pub fn profile(&self, model: Model, raw: &[u8]) -> Result<Vec<Sample>> {
        let d = &self.descriptors[model.0];
        let expanded;
        let data = if d.vendor.eq_ignore_ascii_case("shearwater") {
            expanded = shearwater_decompress(raw);
            expanded.as_deref().unwrap_or(raw)
        } else {
            raw
        };
        let mut collector = Collector::default();
        // SAFETY: `data` outlives the parser, which is destroyed before
        // returning; the callback only touches `collector` through `userdata`.
        unsafe {
            let mut parser = ptr::null_mut();
            let status =
                dc::dc_parser_new2(&mut parser, self.context, d.raw, data.as_ptr(), data.len());
            if status != dc::DC_STATUS_SUCCESS || parser.is_null() {
                return Err(Error::Decoder(format!(
                    "{} data not recognised (status {status})",
                    self.name(model)
                )));
            }
            let status = dc::dc_parser_samples_foreach(
                parser,
                Some(on_sample),
                &mut collector as *mut Collector as *mut c_void,
            );
            dc::dc_parser_destroy(parser);
            if status != dc::DC_STATUS_SUCCESS {
                return Err(Error::Decoder(format!(
                    "{} profile unreadable (status {status})",
                    self.name(model)
                )));
            }
        }
        let mut samples = collector.finish();
        samples.sort_by_key(|s| s.time_s);
        Ok(samples)
    }
}

impl Drop for Catalog {
    fn drop(&mut self) {
        // SAFETY: each pointer was obtained from libdivecomputer and is
        // released exactly once.
        unsafe {
            for d in &self.descriptors {
                dc::dc_descriptor_free(d.raw);
            }
            dc::dc_context_free(self.context);
        }
    }
}

/// Samples arrive as a stream: a time, then the values recorded at that time.
#[derive(Default)]
struct Collector {
    samples: Vec<Sample>,
    current: Option<(u32, Option<f64>, Option<f64>)>,
}

impl Collector {
    fn flush(&mut self) {
        if let Some((time_s, Some(depth_m), temp_c)) = self.current.take() {
            self.samples.push(Sample {
                time_s,
                depth_m,
                temp_c,
            });
        }
    }

    fn finish(mut self) -> Vec<Sample> {
        self.flush();
        self.samples
    }
}

unsafe extern "C" fn on_sample(
    kind: dc::dc_sample_type_t,
    value: *const dc::dc_sample_value_t,
    userdata: *mut c_void,
) {
    // SAFETY: `userdata` is the `Collector` passed to
    // `dc_parser_samples_foreach`; `value` points to a union whose member for
    // `kind` starts at offset 0 (see libdivecomputer-sys).
    unsafe {
        let c = &mut *(userdata as *mut Collector);
        match kind {
            dc::DC_SAMPLE_TIME => {
                c.flush();
                let ms = *(value as *const u32);
                c.current = Some(((ms + 500) / 1000, None, None));
            }
            dc::DC_SAMPLE_DEPTH => {
                if let Some(cur) = c.current.as_mut() {
                    cur.1 = Some(*(value as *const f64));
                }
            }
            dc::DC_SAMPLE_TEMPERATURE => {
                if let Some(cur) = c.current.as_mut() {
                    cur.2 = Some(*(value as *const f64));
                }
            }
            _ => {}
        }
    }
}

/// Real Shearwater logs expand at most ~3x; beyond 32x it is not a dive log.
const MAX_EXPANSION: usize = 32;

/// Undoes Shearwater's transport compression: a stream of 9-bit values (bit 8
/// set = literal byte, 0 = end, otherwise a run of that many zero bytes),
/// then each byte XOR-ed with the one 32 bytes earlier. `None` when the data
/// is not in that form.
pub fn shearwater_decompress(data: &[u8]) -> Option<Vec<u8>> {
    let nbits = data.len() * 8;
    if nbits == 0 || !nbits.is_multiple_of(9) {
        return None;
    }
    let limit = data.len() * MAX_EXPANSION;
    let mut out = Vec::with_capacity(data.len() * 3);
    let mut offset = 0;
    while offset + 9 <= nbits {
        let byte = offset / 8;
        let bit = offset % 8;
        let hi = u32::from(data[byte]);
        let lo = u32::from(*data.get(byte + 1).unwrap_or(&0));
        let value = (((hi << 8) | lo) >> (16 - (bit + 9))) & 0x1FF;
        if value & 0x100 != 0 {
            out.push((value & 0xFF) as u8);
        } else if value == 0 {
            break;
        } else {
            if out.len() + value as usize > limit {
                return None;
            }
            out.resize(out.len() + value as usize, 0);
        }
        offset += 9;
    }
    if out.is_empty() {
        return None;
    }
    for i in 32..out.len() {
        out[i] ^= out[i - 32];
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Packs 9-bit values MSB-first, as Shearwater does.
    fn pack(values: &[u16]) -> Vec<u8> {
        let mut bits = Vec::new();
        for v in values {
            for b in (0..9).rev() {
                bits.push((v >> b) & 1 == 1);
            }
        }
        while bits.len() % 72 != 0 {
            bits.push(false); // pad with end-of-stream zeros to a 9-byte multiple
        }
        bits.chunks(8)
            .map(|c| c.iter().fold(0u8, |a, &b| (a << 1) | b as u8))
            .collect()
    }

    #[test]
    fn finds_computer_models() {
        let c = Catalog::new().unwrap();
        assert!(c.descriptors.len() > 100);
        let name = |s: &str| c.find(s).map(|m| c.name(m));
        assert_eq!(
            name("Shearwater Perdix 2").as_deref(),
            Some("Shearwater Perdix 2")
        );
        assert_eq!(name("Mares Genius").as_deref(), Some("Mares Genius"));
        assert_eq!(
            name("mares puck pro (#001)").as_deref(),
            Some("Mares Puck Pro")
        );
        assert_eq!(name("No Computer"), None);
        assert_eq!(name("Shearwater"), None);
        assert_eq!(name(""), None);
        assert_eq!(name("Acme Wonder 3000"), None);
    }

    #[test]
    fn garbage_is_an_error_not_a_crash() {
        let c = Catalog::new().unwrap();
        for computer in ["Mares Genius", "Shearwater Perdix 2", "Suunto D5"] {
            let m = c.find(computer).unwrap();
            for data in [&[][..], &[0u8; 3][..], &[0xFFu8; 64][..]] {
                if let Ok(p) = c.profile(m, data) {
                    assert!(p.len() < 10);
                }
            }
        }
    }

    #[test]
    fn shearwater_runs_literals_and_xor() {
        // Literal 1, 2, a run of 30 zeros, then 33 literal 7s.
        let mut values = vec![0x101, 0x102, 30];
        values.extend(std::iter::repeat_n(0x107, 33));
        let out = shearwater_decompress(&pack(&values)).unwrap();
        assert_eq!(out.len(), 65);
        // Before XOR: [1, 2, 0 x30, 7 x33]; byte 32 is XOR-ed with byte 0, and so on.
        assert_eq!(&out[..3], &[1, 2, 0]);
        assert_eq!(out[32], 7 ^ 1);
        assert_eq!(out[33], 7 ^ 2);
        assert_eq!(out[34], 7);
        assert_eq!(out[64], 1); // 7 ^ out[32] = 7 ^ (7 ^ 1)
    }

    #[test]
    fn shearwater_rejects_non_compressed_data() {
        assert_eq!(shearwater_decompress(&[]), None);
        assert_eq!(shearwater_decompress(&[1, 2, 3, 4]), None); // not a multiple of 9 bits
        assert_eq!(shearwater_decompress(&[0; 9]), None); // ends immediately
        // A run that would expand far beyond a real log.
        assert_eq!(shearwater_decompress(&pack(&[255; 8])), None);
    }
}
