use super::cpuid;
use phf;

pub mod intel;

use core::fmt::{Write, Result, Error};
use core::str;

const MODEL_LEN: usize = 20;

#[derive(Default)]
struct ModelWriter {
    buffer: [u8; MODEL_LEN],
    index: usize
}

impl ModelWriter {
    fn as_str(&self) -> &str {
        str::from_utf8(&self.buffer[..self.index]).unwrap()
    }
}

impl Write for ModelWriter {
    fn write_str(&mut self, s: &str) -> Result {
        // TODO: There exists probably a more efficient way of doing this:
        for c in s.chars() {
            if self.index >= self.buffer.len() {
                return Err(Error)
            }
            self.buffer[self.index] = c as u8;
            self.index += 1;
        }
        Ok(())
    }
}

/// Return performance counter description for the running micro-architecture.
pub fn available_counters() -> Option<&'static phf::Map<&'static str, intel::description::IntelPerformanceCounterDescription>> {

    let cpuid = cpuid::CpuId::new();

    cpuid.get_vendor_info().map_or(None, |vf| {
        cpuid.get_feature_info().map_or(None, |fi| {
            let vendor = vf.as_string();
            let (family, extended_model, model) = (fi.family_id(), fi.extended_model_id(), fi.model_id());

            let mut writer: ModelWriter = Default::default();
            // Should work as long as it fits in MODEL_LEN bytes:
            write!(writer, "{}-{}-{:X}{:X}", vendor, family, extended_model, model).unwrap();
            let key = writer.as_str();

            intel::counters::COUNTER_MAP.get(key)
        })
    })
}
