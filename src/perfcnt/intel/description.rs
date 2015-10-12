use std::fmt;

pub enum PebsType {
    Regular,
    PebsOrRegular,
    PebsOnly
}

impl fmt::Debug for PebsType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            PebsType::Regular => "Regular",
            PebsType::PebsOrRegular => "PebsOrRegular",
            PebsType::PebsOnly => "PebsOnly",
        };
        write!(f, "PebsType::{}", name)
    }
}

pub enum Tuple {
    One(u8),
    Two(u8,u8)
}

impl fmt::Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Tuple::One(a) => write!(f, "Tuple::One({})", a),
            Tuple::Two(a, b) => write!(f, "Tuple::Two({}, {})", a, b),
        }
    }
}

pub enum MSRIndex {
    None,
    One(u8),
    Two(u8, u8)
}

impl fmt::Debug for MSRIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MSRIndex::None => write!(f, "MSRIndex::None"),
            MSRIndex::One(a) => write!(f, "MSRIndex::One({})", a),
            MSRIndex::Two(a, b) => write!(f, "MSRIndex::Two({}, {})", a, b),
        }
    }
}

pub enum Counter {
    /// Bit-mask containing the fixed counters
    /// usable with the corresponding performance event.
    Fixed(u8),

    /// Bit-mask containing the programmable counters
    /// usable with the corresponding performance event.
    Programmable(u8),
}

impl fmt::Debug for Counter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Counter::Fixed(a) => write!(f, "Counter::Fixed({})", a),
            Counter::Programmable(a) => write!(f, "Counter::Programmable({})", a),
        }
    }
}

#[derive(Debug)]
pub struct IntelPerformanceCounterDescription {

    /// This field maps to the Event Select field in the IA32_PERFEVTSELx[7:0]MSRs.
    ///
    /// The set of values for this field is defined architecturally.
    /// Each value corresponds to an event logic unit and should be used with a unit
    /// mask value to obtain an architectural performance event.
    pub event_code: Tuple,

    /// This field maps to the Unit Mask filed in the IA32_PERFEVTSELx[15:8] MSRs.
    ///
    /// It further qualifies the event logic unit selected in the event select
    /// field to detect a specific micro-architectural condition.
    pub umask: Tuple,

    /// It is a string of characters to identify the programming of an event.
    pub event_name: &'static str,

    /// This field contains a description of what is being counted by a particular event.
    pub brief_description: &'static str,

    /// In some cases, this field will contain a more detailed description of what is counted by an event.
    pub public_description: Option<&'static str>,

    /// This field lists the fixed (PERF_FIXED_CTRX) or programmable (IA32_PMCX)
    /// counters that can be used to count the event.
    pub counter: Counter,

    /// This field lists the counters where this event can be sampled
    /// when Intel® Hyper-Threading Technology (Intel® HT Technology) is
    /// disabled.
    ///
    /// When Intel® HT Technology is disabled, some processor cores gain access to
    /// the programmable counters of the second thread, making a total of eight
    /// programmable counters available. The additional counters will be
    /// numbered 4,5,6,7. Fixed counter behavior remains unaffected.
    pub counter_ht_off: Counter,

    /// This field is only relevant to PEBS events.
    ///
    /// It lists the counters where the event can be sampled when it is programmed as a PEBS event.
    pub pebs_counters: Option<Counter>,

    /// Sample After Value (SAV) is the value that can be preloaded
    /// into the counter registers to set the point at which they will overflow.
    ///
    /// To make the counter overflow after N occurrences of the event,
    /// it should be loaded with (0xFF..FF – N) or –(N-1). On overflow a
    /// hardware interrupt is generated through the Local APIC and additional
    /// architectural state can be collected in the interrupt handler.
    /// This is useful in event-based sampling. This field gives a recommended
    /// default overflow value, which may be adjusted based on workload or tool preference.
    pub sample_after_value: u64,

    /// Additional MSRs may be required for programming certain events.
    /// This field gives the address of such MSRS.
    pub msr_index: MSRIndex,

    /// When an MSRIndex is used (indicated by the MSRIndex column), this field will
    /// contain the value that needs to be loaded into the
    /// register whose address is given in MSRIndex column.
    ///
    /// For example, in the case of the load latency events, MSRValue defines the
    /// latency threshold value to write into the MSR defined in MSRIndex (0x3F6).
    pub msr_value: u64,

    /// This field is set for an event which can only be sampled or counted by itself,
    /// meaning that when this event is being collected,
    /// the remaining programmable counters are not available to count any other events.
    pub taken_alone: bool,

    /// This field maps to the Counter Mask (CMASK) field in IA32_PERFEVTSELx[31:24] MSR.
    pub counter_mask: u8,

    /// This field corresponds to the Invert Counter Mask (INV) field in IA32_PERFEVTSELx[23] MSR.
    pub invert: bool,

    /// This field corresponds to the Any Thread (ANY) bit of IA32_PERFEVTSELx[21] MSR.
    pub any_thread: bool,

    /// This field corresponds to the Edge Detect (E) bit of IA32_PERFEVTSELx[18] MSR.
    pub edge_detect: bool,

    /// A '0' in this field means that the event cannot be programmed as a PEBS event.
    /// A '1' in this field means that the event is a  precise event and can be programmed
    /// in one of two ways – as a regular event or as a PEBS event.
    /// And a '2' in this field means that the event can only be programmed as a PEBS event.
    pub pebs: PebsType,

    /// A '1' in this field means the event uses the Precise Store feature and Bit 3 and
    /// bit 63 in IA32_PEBS_ENABLE MSR must be set to enable IA32_PMC3 as a PEBS counter
    /// and enable the precise store facility respectively.
    ///
    /// Processors based on SandyBridge and IvyBridge micro-architecture offer a
    /// precise store capability that provides a means to profile store memory
    /// references in the system.
    pub precise_store: bool,

    /// A '1' in this field means that when the event is configured as a PEBS event,
    /// the Data Linear Address facility is supported.
    ///
    /// The Data Linear Address facility is a new feature added to Haswell as a
    /// replacement or extension of the precise store facility in SNB.
    pub data_la: bool,

    /// A '1' in this field means that when the event is configured as a PEBS event,
    /// the DCU hit field of the PEBS record is set to 1 when the store hits in the
    /// L1 cache and 0 when it misses.
    pub l1_hit_indication: bool,

    /// This field lists the known bugs that apply to the events.
    ///
    /// For the latest, up to date errata refer to the following links:
    ////
    /// * Haswell:
    ///   http://www.intel.com/content/dam/www/public/us/en/documents/specification-updates/4th-gen-core-family-mobile-specification-update.pdf
    ///
    /// * IvyBridge:
    ///   https://www-ssl.intel.com/content/dam/www/public/us/en/documents/specification-updates/3rd-gen-core-desktop-specification-update.pdf
    ///
    /// * SandyBridge:
    ///   https://www-ssl.intel.com/content/dam/www/public/us/en/documents/specification-updates/2nd-gen-core-family-mobile-specification-update.pdf
    pub errata: Option<&'static str>,

    /// There is only 1 file for core and offcore events in this format.
    /// This field is set to 1 for offcore events and 0 for core events.
    pub offcore: bool,
}

impl IntelPerformanceCounterDescription {

    #[allow(dead_code)]
    fn new(event_code: Tuple, umask: Tuple, event_name: &'static str,
           brief_description: &'static str, public_description: Option<&'static str>,
           counter: Counter, counter_ht_off: Counter, pebs_counters: Option<Counter>,
           sample_after_value: u64, msr_index: MSRIndex, msr_value: u64, taken_alone: bool,
           counter_mask: u8, invert: bool, any_thread: bool, edge_detect: bool, pebs:
           PebsType, precise_store: bool, data_la: bool, l1_hit_indication: bool,
           errata: Option<&'static str>, offcore: bool) -> IntelPerformanceCounterDescription {

        IntelPerformanceCounterDescription {
            event_code: event_code,
            umask: umask,
            event_name: event_name,
            brief_description: brief_description,
            public_description: public_description,
            counter: counter,
            counter_ht_off: counter_ht_off,
            pebs_counters: pebs_counters,
            sample_after_value: sample_after_value,
            msr_index: msr_index,
            msr_value: msr_value,
            taken_alone: taken_alone,
            counter_mask: counter_mask,
            invert: invert,
            any_thread: any_thread,
            edge_detect: edge_detect,
            pebs: pebs,
            precise_store: precise_store,
            data_la: data_la,
            l1_hit_indication: l1_hit_indication,
            errata: errata,
            offcore: offcore
        }
    }
}
