#![feature(convert)]

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

use serde_json::Value;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/perfcnt/intel/description.rs"));

/// We need to convert parsed strings to static because we're reusing
/// the struct definition which declare strings (rightfully) as
/// static in the generated code.
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

            let mut do_insert: bool = false;

            let pcn = entry.as_object().unwrap();
            for (key, value) in pcn.iter() {
                if !value.is_string() {
                    panic!("Not a string");
                }
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
                        let split_parts: Vec<u64> = split_str_parts.iter()
                            .map(|x| { assert!(x.starts_with("0x")); u64::from_str_radix(&x[2..], 16).unwrap() })
                            .collect();

                        match split_parts.len() {
                            1 => event_code = Tuple::One(split_parts[0] as u8),
                            2 => event_code = Tuple::Two(split_parts[0] as u8, split_parts[1] as u8),
                            _ => panic!("More than two event codes?")
                        }
                    },

                    "UMask" => {
                        let split_parts: Vec<u64> = split_str_parts.iter()
                            .map(|x| { assert!(x.starts_with("0x")); u64::from_str_radix(&x[2..], 16).unwrap() })
                            .collect();

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

                    "Counter" => {
                        if value_str.starts_with("Fixed counter") {
                            let mask: u64 = value_str["Fixed counter".len()..]
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter = Counter::Fixed(mask as u8);
                        }
                        else {
                            let mask: u64 = value_str
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter = Counter::Programmable(mask as u8);
                        }
                    },

                    "CounterHTOff" => {
                        if value_str.starts_with("Fixed counter") {
                            let mask: u64 = value_str["Fixed counter".len()..]
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter_ht_off = Counter::Fixed(mask as u8);
                        }
                        else {
                            let mask: u64 = value_str
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter_ht_off = Counter::Programmable(mask as u8);
                        }
                    },

                    "PEBScounters" => {
                        if value_str.starts_with("Fixed counter") {
                            let mask: u64 = value_str["Fixed counter".len()..]
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            pebs_counters = Some(Counter::Fixed(mask as u8));
                        }
                        else {
                            let mask: u64 = value_str
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            pebs_counters = Some(Counter::Programmable(mask as u8));
                        }
                    },

                    "SampleAfterValue" => sample_after_value = u64::from_str_radix(&value_str, 10).unwrap(),

                    "MSRIndex" => {
                        let split_parts: Vec<u64> = value_str
                            .split(",")
                            .map(|x| x.trim())
                            .map(|x| {
                                if x.len() > 2 && x[..2].starts_with("0x") {
                                    u64::from_str_radix(&x[2..], 16).unwrap()
                                }
                                else {
                                    u64::from_str_radix(&x, 10).unwrap()
                                }
                            })
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
                    "MSRValue" => {
                        msr_value = if value_str.len() > 2 && value_str[..2].starts_with("0x") {
                            u64::from_str_radix(&value_str[2..], 16).unwrap()
                        }
                        else {
                            u64::from_str_radix(&value_str, 10).unwrap()
                        }
                    },
                    "TakenAlone" => {
                        taken_alone = parse_bool(value_str);
                    },
                    "CounterMask" => {
                        counter_mask = if value_str.len() > 2 && value_str[..2].starts_with("0x") {
                            u8::from_str_radix(&value_str[2..], 16).unwrap()
                        }
                        else {
                            u8::from_str_radix(&value_str, 10).unwrap()
                        }
                    },
                    "Invert" => {
                        invert = parse_bool(value_str);
                    }
                    "AnyThread" => any_thread = parse_bool(value_str),
                    "EdgeDetect" => edge_detect = parse_bool(value_str),
                    "PEBS" => {
                        pebs = match value_str.trim() {
                            "0" => PebsType::Regular,
                            "1" => PebsType::PebsOrRegular,
                            "2" => PebsType::PebsOnly,
                            _ => panic!("Unknown PEBS type: {}", value_str),
                        }
                    },
                    "PRECISE_STORE" => precise_store = parse_bool(value_str),
                    "Data_LA" => data_la = parse_bool(value_str),
                    "L1_Hit_Indication" => l1_hit_indication = parse_bool(value_str),
                    "Errata" => {
                        errata = if value_str != "null" {
                            Some(value_str)
                        }
                        else {
                            None
                        };
                    },
                    "Offcore" => offcore = parse_bool(value_str),
                    "ELLC" => {
                        // Ignored due to missing documentation.
                    },
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
                offcore
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


    write!(file, "pub static {}: phf::Map<&'static str, IntelPerformanceCounterDescription> = ", variable).unwrap();
    builder.build(file).unwrap();
    write!(file, ";\n").unwrap();
}

fn make_file_name<'a>(path: &'a Path) -> (String, String) {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    // File name without _core*.json
    println!("{:?}", stem);
    let core_start = stem.find("_core").unwrap();
    let (output_file, _) = stem.split_at(core_start);

    // File name without _V*.json at the end:
    let version_start = stem.find("_V").unwrap();
    let (variable, _) = stem.split_at(version_start);
    let uppercase = variable.to_ascii_uppercase();
    let variable_clean = uppercase.replace("-", "_");
    let variable_upper = variable_clean.as_str();

    (output_file.to_string(), variable_upper.to_string())
}

fn main() {
    let mut rdr = csv::Reader::from_file("./x86data/perfmon_data/mapfile.csv").unwrap();
    let mut data_files = HashMap::new();

    // Parse CSV
    for record in rdr.decode() {
        let (family_model, version, file_name, event_type): (String, String, String, String) = record.unwrap();
        // TODO: Parse offcore counter descriptions.
        if file_name.contains("_core_") && !data_files.contains_key(&file_name) {
            data_files.insert(file_name.clone(), (family_model, version, event_type));
        }
    }

    // build hash-table to select performance counter per CPU architecture
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("counters.rs");
    let mut filewriter = BufWriter::new(File::create(&path).unwrap());

    let mut builder = phf_codegen::Map::new();
    for (file, data) in &data_files {
        let (ref family_model, _, _): (String, String, String) = *data;
        let path = Path::new(file.as_str());
        let (_, ref variable_upper) = make_file_name(&path);

        builder.entry(family_model.as_str(), format!("&{}", variable_upper.as_str()).as_str());
    }

    write!(&mut filewriter, "pub static {}: phf::Map<&'static str, &'static phf::Map<&'static str, IntelPerformanceCounterDescription>> = ", "COUNTER_MAP").unwrap();
    builder.build(&mut filewriter).unwrap();
    write!(&mut filewriter, ";\n").unwrap();

    // Parse all json files and write hash-tables from it
    // TODO: Parse offcore counter descriptions.
    for (file, data) in &data_files {
        if file.contains("_core_") {
            let (ref family_model, ref version, ref event_type): (String, String, String) = *data;
            println!("Processing {:?} {} {} {}", file, family_model, version, event_type);

            let path = Path::new(file.as_str());
            let (_, ref variable_upper) = make_file_name(&path);
            parse_performance_counters(format!("x86data/perfmon_data{}", file).as_str(),
                                       variable_upper, &mut filewriter);
        }
    }

}
