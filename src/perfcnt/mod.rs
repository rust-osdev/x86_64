use super::cpuid;
use phf;

pub mod intel;

use core::fmt::{Write, Result};
use core::str;

/*
pub trait Write {

    fn write_char(&mut self, c: char) -> Result { ... }
    fn write_fmt(&mut self, args: Arguments) -> Result { ... }
}*/

#[derive(Default)]
struct ModelWriter {
    buffer: [u8; 20],
    index: usize
}

impl ModelWriter {
    fn as_str(&self) -> &str {
        str::from_utf8(&self.buffer).unwrap()
    }
}

impl Write for ModelWriter {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            self.buffer[self.index] = c as u8;
            self.index += 1;
        }
        Ok(())
    }
}

/// Return performance counter description for the running micro-architecture.
pub fn available_counters() -> Option<&'static phf::Map<&'static str, intel::description::IntelPerformanceCounterDescription>> {

    let cpuid = cpuid::CpuId::new();

    cpuid.get_vendor_info().map(|vf| {
        cpuid.get_feature_info().map(|fi| {
            let vendor = vf.as_string();
            let (family, extended_model, model) = (fi.family_id(), fi.extended_model_id(), fi.model_id());

            let mut writer: ModelWriter = Default::default();
            let res = write!(writer, "{}-{}-{:X}{:X}", vendor, family, extended_model, model);
            let key = writer.as_str();

            match intel::counters::COUNTER_MAP.contains_key(key) {
                true => return Some(intel::counters::COUNTER_MAP[key]),
                false => return None
            };
        });
    });

    None
}

#[cfg(test)]
#[test]
fn list_mine() {
    for counter in available_counters() {
        println!("{:?}", counter);
    }
}