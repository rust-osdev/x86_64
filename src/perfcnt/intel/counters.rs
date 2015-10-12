use phf;
use super::description::IntelPerformanceCounterDescription;
use super::description::Counter;
use super::description::PebsType;
use super::description::Tuple;
use super::description::MSRIndex;

include!(concat!(env!("OUT_DIR"), "/counters.rs"));