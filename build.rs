#![cfg_attr(feature = "performance-counter", feature(convert))]

#[cfg(not(feature = "performance-counter"))]
fn main() {}

#[cfg(feature = "performance-counter")]
fn main() {
    performance_counter::main();
}

#[cfg(feature = "performance-counter")]
mod performance_counter {

extern crate phf_codegen;
extern crate serde_json;
extern crate csv;

use std::ascii::AsciiExt;
use std::env;
use std::fs::File;
use std::io::{BufWriter, BufReader, Write};
use std::path::Path;
use std::collections::HashMap;
use std::mem;

use self::serde_json::Value;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/perfcnt/intel/description.rs"));

/// HACK: We need to convert parsed strings to static because we're reusing
/// the struct definition which declare strings as static in the generated code.
fn string_to_static_str<'a>(s: &'a str) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

fn parse_bool(input: &str) -> bool {
    match input.trim() {
        "0" => false,
        "1" => true,
        _ => panic!("Unknown boolean value {}", input),
    }
}

fn parse_hex_numbers(split_str_parts: Vec<&str>) -> Vec<u64> {
    split_str_parts.iter()
        .map(|x| {
            assert!(x.starts_with("0x"));
            match u64::from_str_radix(&x[2..], 16) {
                Ok(u) => u,
                Err(e) => panic!("{}: Can not parse {}", e, x)
            }
        }).collect()
}

fn parse_number(value_str: &str) -> u64 {
    if value_str.len() > 2 && value_str[..2].starts_with("0x") {
        match u64::from_str_radix(&value_str[2..], 16) {
            Ok(u) => u,
            Err(e) => panic!("{}: Can not parse {}", e, value_str)
        }
    }
    else {
        match u64::from_str_radix(&value_str, 10) {
            Ok(u) => u,
            Err(e) => panic!("{}: Can not parse {}", e, value_str)
        }
    }
}

fn parse_counter_values(value_str: &str) -> u64 {
    value_str.split(",").map(|x| x.trim())
        .filter(|x| x.len() > 0)
        .map(|x| {
            match u64::from_str_radix(&x, 10) {
                Ok(u) => u,
                Err(e) => panic!("{}: Can not parse {} in {}", e, x, value_str)
            }
        }).fold(0, |acc, c| { assert!(c < 8); acc | 1 << c })
}

fn parse_null_string(value_str: &str) -> Option<&str> {
    if value_str != "null" {
        Some(value_str)
    }
    else {
        None
    }
}

fn parse_counters(value_str: &str) -> Counter {
    if value_str.to_lowercase().starts_with("fixed counter") {
        let mask: u64 = parse_counter_values(&value_str["fixed counter".len()..]);
        Counter::Fixed(mask as u8)
    }
    else if value_str.to_lowercase().starts_with("fixed") {
        let mask: u64 = parse_counter_values(&value_str["fixed".len()..]);
        Counter::Fixed(mask as u8)
    }
    else {
        let mask: u64 = parse_counter_values(value_str);
        Counter::Programmable(mask as u8)
    }
}

fn parse_pebs(value_str: &str) -> PebsType {
    match value_str.trim() {
        "0" => PebsType::Regular,
        "1" => PebsType::PebsOrRegular,
        "2" => PebsType::PebsOnly,
        _ => panic!("Unknown PEBS type: {}", value_str),
    }
}

fn parse_performance_counters(input: &str, variable: &str, file: &mut BufWriter<File>) {
    let mut builder = phf_codegen::Map::new();
    let f = File::open(input).unwrap();
    let reader = BufReader::new(f);
    let data: Value = serde_json::from_reader(reader).unwrap();
    let mut all_events = HashMap::new();

    if data.is_array() {
        let entries = data.as_array().unwrap();
        for entry in entries.iter() {

            if !entry.is_object() {
                panic!("Expected JSON object.");
            }
            let pcn = entry.as_object().unwrap();

            let mut event_code = Tuple::One(0);
            let mut umask = Tuple::One(0);
            let mut event_name = "";
            let mut brief_description = "";
            let mut public_description = None;
            let mut counter = Counter::Fixed(0);
            let mut counter_ht_off = Counter::Fixed(0);
            let mut pebs_counters = None;
            let mut sample_after_value = 0;
            let mut msr_index = MSRIndex::None;
            let mut msr_value = 0;
            let mut taken_alone = false;
            let mut counter_mask = 0;
            let mut invert = false;
            let mut any_thread = false;
            let mut edge_detect = false;
            let mut pebs = PebsType::Regular;
            let mut precise_store = false;
            let mut data_la = false;
            let mut l1_hit_indication = false;
            let mut errata = None;
            let mut offcore = false;
            let mut unit = None;
            let mut filter = None;
            let mut extsel = false;

            let mut do_insert: bool = false;

            for (key, value) in pcn.iter() {
                if !value.is_string() {
                    panic!("Not a string");
                }

                println!("key = {} value = {}", key, value.as_string().unwrap());
                let value_string = value.as_string().unwrap();
                let value_str = string_to_static_str(value_string).trim();
                let split_str_parts: Vec<&str> = value_string.split(",").map(|x| x.trim()).collect();

                match key.as_str() {
                    "EventName" => {
                        if !all_events.contains_key(value_str.clone()) {
                            all_events.insert(value_str, 0);
                            assert!(all_events.contains_key(value_str));
                            do_insert = true;
                        }
                        else {
                            do_insert = false;
                            println!("WARN: Key {} already exists.", value_str);
                        }
                        event_name = value_str;
                    }
                    "EventCode" => {
                        let split_parts: Vec<u64> = parse_hex_numbers(split_str_parts);
                        match split_parts.len() {
                            1 => event_code = Tuple::One(split_parts[0] as u8),
                            2 => event_code = Tuple::Two(split_parts[0] as u8, split_parts[1] as u8),
                            _ => panic!("More than two event codes?")
                        }
                    },
                    "UMask" => {
                        let split_parts: Vec<u64> = parse_hex_numbers(split_str_parts);
                        match split_parts.len() {
                            1 => umask = Tuple::One(split_parts[0] as u8),
                            2 => umask = Tuple::Two(split_parts[0] as u8, split_parts[1] as u8),
                            _ => panic!("More than two event codes?")
                        }
                    },
                    "BriefDescription" => brief_description = value_str,
                    "PublicDescription" => {
                        if brief_description != value_str && value_str != "tbd" {
                            public_description = Some(value_str);
                        }
                        else {
                            public_description = None;
                        }
                    },
                    "Counter" => counter = parse_counters(value_str),
                    "CounterHTOff" => counter_ht_off = parse_counters(value_str),
                    "PEBScounters" => pebs_counters = Some(parse_counters(value_str)),
                    "SampleAfterValue" => sample_after_value = parse_number(value_str),
                    "MSRIndex" => {
                        let split_parts: Vec<u64> = value_str
                            .split(",")
                            .map(|x| x.trim())
                            .map(|x| parse_number(x))
                            .collect();

                            msr_index = match split_parts.len() {
                                1 => {
                                    if split_parts[0] != 0 {
                                        MSRIndex::One(split_parts[0] as u8)
                                    }
                                    else {
                                        MSRIndex::None
                                    }
                                },
                                2 => MSRIndex::Two(split_parts[0] as u8, split_parts[1] as u8),
                                _ => panic!("More than two MSR indexes?")
                            }
                    },
                    "MSRValue" => msr_value = parse_number(value_str),
                    "TakenAlone" => taken_alone = parse_bool(value_str),
                    "CounterMask" => counter_mask = parse_number(value_str) as u8,
                    "Invert" => invert = parse_bool(value_str),
                    "AnyThread" => any_thread = parse_bool(value_str),
                    "EdgeDetect" => edge_detect = parse_bool(value_str),
                    "PEBS" => pebs = parse_pebs(value_str),
                    "PRECISE_STORE" => precise_store = parse_bool(value_str),
                    "Data_LA" => data_la = parse_bool(value_str),
                    "L1_Hit_Indication" => l1_hit_indication = parse_bool(value_str),
                    "Errata" => errata = parse_null_string(value_str),
                    "Offcore" => offcore = parse_bool(value_str),
                    "Unit" => unit = parse_null_string(value_str),
                    "Filter" => filter = parse_null_string(value_str),
                    "ExtSel" => extsel = parse_bool(value_str),
                    "ELLC" => {/* Ignored due to missing documentation. */ },
                    _ => panic!("Unknown member: {} in file {}", key, input),
                };
            }

            let ipcd = IntelPerformanceCounterDescription::new(
                event_code,
                umask,
                event_name,
                brief_description,
                public_description,
                counter,
                counter_ht_off,
                pebs_counters,
                sample_after_value,
                msr_index,
                msr_value,
                taken_alone,
                counter_mask,
                invert,
                any_thread,
                edge_detect,
                pebs,
                precise_store,
                data_la,
                l1_hit_indication,
                errata,
                offcore,
                unit,
                filter,
                extsel
            );

            //println!("{:?}", ipcd.event_name);
            if do_insert == true {
                builder.entry(ipcd.event_name, format!("{:?}", ipcd).as_str());
            }
        }
    }
    else {
        panic!("JSON data is not an array.");
    }


    write!(file, "pub const {}: phf::Map<&'static str, IntelPerformanceCounterDescription> = ", variable).unwrap();
    builder.build(file).unwrap();
    write!(file, ";\n").unwrap();
}

fn make_file_name<'a>(path: &'a Path) -> (String, String) {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    // File name without _core*.json
    println!("{:?}", path);
    let mut core_start = stem.find("_core");
    if core_start.is_none() {
        core_start = stem.find("_uncore");
    }
    assert!(!core_start.is_none());
    let (output_file, _) = stem.split_at(core_start.unwrap());

    // File name without _V*.json at the end:
    let version_start = stem.find("_V").unwrap();
    let (variable, _) = stem.split_at(version_start);
    let uppercase = variable.to_ascii_uppercase();
    let variable_clean = uppercase.replace("-", "_");
    let variable_upper = variable_clean.as_str();

    (output_file.to_string(), variable_upper.to_string())
}

pub fn get_file_suffix(file_name: String) -> &'static str {
    if file_name.contains("_core_") {
        "core"
    }
    else if file_name.contains("_uncore_") {
        "uncore"
    }
    else if file_name.contains("_matrix_") {
        "matrix"
    }
    else if file_name.contains("_FP_ARITH_INST_") {
        "fparith"
    }
    else {
        panic!("Unknown suffix {}", file_name);
    }
}

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=x86data/perfmon_data");

    let mut rdr = csv::Reader::from_file("./x86data/perfmon_data/mapfile.csv").unwrap();
    let mut data_files = HashMap::new();

    // Parse CSV
    for record in rdr.decode() {
        let (family_model, version, file_name, event_type): (String, String, String, String) = record.unwrap();
        // TODO: Parse offcore counter descriptions.
        if !data_files.contains_key(&file_name) {
            let suffix = get_file_suffix(file_name.clone());
            if suffix == "core" || suffix == "uncore" {
                data_files.insert(file_name.clone(), (family_model + "-" + suffix, version, event_type));
            }
        }
    }

    // Build hash-table to select performance counter for each architecture
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("counters.rs");
    let mut filewriter = BufWriter::new(File::create(&path).unwrap());

    let mut builder = phf_codegen::Map::new();
    for (file, data) in &data_files {
        let (ref family_model, _, _): (String, String, String) = *data;
        let path = Path::new(file.as_str());
        let (_, ref variable_upper) = make_file_name(&path);
        builder.entry(family_model.as_str(), format!("{}", variable_upper.as_str()).as_str());
    }

    write!(&mut filewriter, "pub static {}: phf::Map<&'static str, phf::Map<&'static str, IntelPerformanceCounterDescription>> = ", "COUNTER_MAP").unwrap();
    builder.build(&mut filewriter).unwrap();
    write!(&mut filewriter, ";\n").unwrap();

    // Parse all JSON files and write the hash-tables
    for (file, data) in &data_files {
        let suffix = get_file_suffix(file.clone());
        if suffix == "core" || suffix == "uncore" {
            let (ref family_model, ref version, ref event_type): (String, String, String) = *data;
            println!("Processing {:?} {} {} {}", file, family_model, version, event_type);

            let path = Path::new(file.as_str());
            let (_, ref variable_upper) = make_file_name(&path);
            parse_performance_counters(format!("x86data/perfmon_data{}", file).as_str(),
                                       variable_upper, &mut filewriter);
        }
    }

}
}
