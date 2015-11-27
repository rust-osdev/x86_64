use phf;
use super::description::IntelPerformanceCounterDescription;
use super::description::Counter;
use super::description::PebsType;
use super::description::Tuple;
use super::description::MSRIndex;

#[cfg(not(feature = "cached_counters"))]
include!(concat!(env!("OUT_DIR"), "/counters.rs"));

#[cfg(feature = "cached_counters")]
include!("generated.rs");
