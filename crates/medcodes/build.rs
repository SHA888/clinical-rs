//! Build script for medcodes crate - generates ICD-10-CM and CCSR data

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unwrap_or_default)]

use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

fn main() {
    // Only check for data changes if data directory exists
    // This allows publishing to crates.io without including large data files
    setup_rerun_if_changed();

    // Generate all code system data
    generate_all_data();
}

fn setup_rerun_if_changed() {
    let data_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
    if Path::new(data_dir).exists() {
        println!("cargo:rerun-if-changed=data/april-1-2026-code-tables-tabular-and-index.zip");
        println!("cargo:rerun-if-changed=data/DXCCSR-v2026-1.zip");
        println!("cargo:rerun-if-changed=data/icd9cm/");
        println!("cargo:rerun-if-changed=data/ccs/");
        println!("cargo:rerun-if-changed=data/atc/");
        println!("cargo:rerun-if-changed=data/ndc/");
        println!("cargo:rerun-if-changed=data/loinc/LoincTable/Loinc.csv");
        println!(
            "cargo:rerun-if-changed=data/loinc/AccessoryFiles/ComponentHierarchyBySystem/ComponentHierarchyBySystem.csv"
        );
    }
}

fn generate_all_data() {
    generate_icd10cm_data();
    generate_ccsr_data();
    generate_ccs_data();
    generate_icd9cm_data();
    generate_atc_data();
    generate_ndc_data();
    generate_ndc_to_atc_data();
    generate_ndc_to_rxnorm_data();
    // LOINC generation must run after other data to keep ordering consistent
    generate_loinc_data();
}

fn generate_icd10cm_data() {
    // Generate ICD-10-CM data
    let icd10_data_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/Table and Index/icd10cm_tabular_2026.xml"
    );

    if Path::new(icd10_data_path).exists() {
        eprintln!("Found CMS data file at {icd10_data_path}");
        match parse_cms_xml(icd10_data_path) {
            Ok((codes, parents, children)) => {
                eprintln!("Successfully parsed CMS data, generating maps...");
                generate_phf_maps(&codes, &parents, &children);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse CMS XML: {e}");
                eprintln!("Using empty maps. Ensure the CMS data is extracted properly.");
                generate_empty_maps();
            }
        }
    } else {
        eprintln!("Warning: CMS data file not found at {icd10_data_path}");
        eprintln!("Extract the ZIP file to populate ICD-10-CM codes, or use with-data feature.");
        generate_empty_maps();
    }
}

fn generate_ccsr_data() {
    // Generate CCSR mapping data
    let ccsr_csv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/DXCCSR_v2026-1.csv");

    if Path::new(ccsr_csv_path).exists() {
        eprintln!("Found CCSR data file at {ccsr_csv_path}");
        match parse_ccsr_csv(ccsr_csv_path) {
            Ok(mappings) => {
                eprintln!("Successfully parsed CCSR data, generating mappings...");
                generate_ccsr_maps(&mappings);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse CCSR CSV: {e}");
                eprintln!("Using empty CCSR mappings.");
                generate_empty_ccsr_maps();
            }
        }
    } else {
        eprintln!("Warning: CCSR data file not found at {ccsr_csv_path}");
        eprintln!("Extract the ZIP file to populate CCSR mappings, or use with-data feature.");
        generate_empty_ccsr_maps();
    }
}

#[cfg(not(feature = "loinc"))]
const fn generate_loinc_data() {
    // no-op when loinc feature is disabled
}

fn generate_ccs_data() {
    // Generate CCS mapping data
    let icd10cm_ccs_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/ccs/icd10cm_mappings.csv");
    let icd9cm_ccs_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/ccs/icd9cm_mappings.csv");

    let mut icd10cm_ccs_mappings = Vec::new();
    let mut icd9cm_ccs_mappings = Vec::new();

    if Path::new(icd10cm_ccs_path).exists() {
        match parse_ccs_csv(icd10cm_ccs_path) {
            Ok(mappings) => {
                eprintln!("Successfully parsed ICD-10-CM to CCS data");
                icd10cm_ccs_mappings = mappings;
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse ICD-10-CM CCS CSV: {e}");
            }
        }
    } else {
        eprintln!("Warning: ICD-10-CM CCS data file not found at {icd10cm_ccs_path}");
    }

    if Path::new(icd9cm_ccs_path).exists() {
        match parse_ccs_csv(icd9cm_ccs_path) {
            Ok(mappings) => {
                eprintln!("Successfully parsed ICD-9-CM to CCS data");
                icd9cm_ccs_mappings = mappings;
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse ICD-9-CM CCS CSV: {e}");
            }
        }
    } else {
        eprintln!("Warning: ICD-9-CM CCS data file not found at {icd9cm_ccs_path}");
    }

    // Generate CCS data if we have any mappings
    if !icd10cm_ccs_mappings.is_empty() || !icd9cm_ccs_mappings.is_empty() {
        generate_ccs_maps(&icd10cm_ccs_mappings, &icd9cm_ccs_mappings);
    } else {
        eprintln!("Using empty CCS mappings.");
        generate_empty_ccs_maps();
    }
}

// LOINC data generation
#[cfg(feature = "loinc")]
fn generate_loinc_data() {
    // Use the full official LOINC table from the Loinc_2.82 distribution.
    // Column layout (full Loinc.csv, 40 columns):
    //   0: LOINC_NUM  1: COMPONENT  2: PROPERTY  3: TIME_ASPCT  4: SYSTEM
    //   5: SCALE_TYP  6: METHOD_TYP  25: LONG_COMMON_NAME
    let csv_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/loinc/LoincTable/Loinc.csv"
    );
    if !Path::new(csv_path).exists() {
        eprintln!("Warning: LOINC data file not found at {csv_path}. Using empty LOINC maps.");
        generate_empty_loinc_maps();
        return;
    }

    let mut reader = csv::Reader::from_path(csv_path).expect("Failed to read LOINC CSV");

    let mut descriptions: HashMap<String, String> = HashMap::new();
    let mut components: HashMap<String, (String, String, String, String, String, Option<String>)> =
        HashMap::new();

    for result in reader.records() {
        let record = result.expect("Failed to parse CSV record");

        let loinc_num = record.get(0).unwrap_or("").trim();
        if loinc_num.is_empty() {
            continue;
        }

        let component = record.get(1).unwrap_or("").trim();
        let property = record.get(2).unwrap_or("").trim();
        let timing = record.get(3).unwrap_or("").trim();
        let system = record.get(4).unwrap_or("").trim();
        let scale = record.get(5).unwrap_or("").trim();
        let method = {
            let m = record.get(6).unwrap_or("").trim();
            if m.is_empty() {
                None
            } else {
                Some(m.to_string())
            }
        };
        let description = record.get(25).unwrap_or("").trim();

        let loinc_num = loinc_num.to_string();
        let _ = descriptions.insert(loinc_num.clone(), description.to_string());
        let _ = components.insert(
            loinc_num,
            (
                component.to_string(),
                property.to_string(),
                timing.to_string(),
                system.to_string(),
                scale.to_string(),
                method,
            ),
        );
    }

    eprintln!("Parsed {} LOINC codes", descriptions.len());

    // The multiaxial hierarchy (parent/child edges + Part/group display names)
    // lives in a separate accessory file and is optional: if it is missing we
    // still generate real term data with empty hierarchy maps.
    let hierarchy_csv_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/loinc/AccessoryFiles/ComponentHierarchyBySystem/ComponentHierarchyBySystem.csv"
    );
    let (parents, children, part_names) = if Path::new(hierarchy_csv_path).exists() {
        let (parents, children, part_names) = parse_loinc_hierarchy(hierarchy_csv_path);
        eprintln!(
            "Parsed LOINC hierarchy: {} parent edges, {} parent groups, {} part names",
            parents.len(),
            children.len(),
            part_names.len()
        );
        (parents, children, part_names)
    } else {
        eprintln!(
            "Warning: LOINC hierarchy file not found at {hierarchy_csv_path}. Using empty LOINC hierarchy maps."
        );
        (HashMap::new(), HashMap::new(), HashMap::new())
    };

    generate_loinc_maps(&descriptions, &components, &parents, &children, &part_names);
}

/// Parse the LOINC multiaxial hierarchy file (`ComponentHierarchyBySystem.csv`).
///
/// Column layout: `PATH_TO_ROOT, SEQUENCE, IMMEDIATE_PARENT, CODE, CODE_TEXT`.
/// `IMMEDIATE_PARENT` is always a LOINC Part number (`LP...`); `CODE` is either
/// a Part number (branch node) or a LOINC term number (leaf). A small fraction
/// of codes appear under more than one parent (true polyhierarchy) — the first
/// occurrence in the file is kept as the canonical parent, matching the
/// single-parent tree model already used by the ICD-10-CM hierarchy.
#[cfg(feature = "loinc")]
fn parse_loinc_hierarchy(
    csv_path: &str,
) -> (
    HashMap<String, String>,
    HashMap<String, Vec<String>>,
    HashMap<String, String>,
) {
    let mut parents: HashMap<String, String> = HashMap::new();
    let mut part_names: HashMap<String, String> = HashMap::new();

    let mut reader = csv::Reader::from_path(csv_path).expect("Failed to read LOINC hierarchy CSV");
    for result in reader.records() {
        let record = result.expect("Failed to parse LOINC hierarchy CSV record");

        let code = record.get(3).unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        let immediate_parent = record.get(2).unwrap_or("").trim();
        let code_text = record.get(4).unwrap_or("").trim();

        if code.starts_with("LP") {
            let _ = part_names
                .entry(code.to_string())
                .or_insert_with(|| code_text.to_string());
        }
        if !immediate_parent.is_empty() {
            let _ = parents
                .entry(code.to_string())
                .or_insert_with(|| immediate_parent.to_string());
        }
    }

    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for (child, parent) in &parents {
        children
            .entry(parent.clone())
            .or_default()
            .push(child.clone());
    }
    for kids in children.values_mut() {
        kids.sort();
    }

    (parents, children, part_names)
}

/// Write a u16-length-prefixed string to `writer`.
fn write_loinc_str(writer: &mut impl std::io::Write, s: &str) {
    let len = u16::try_from(s.len()).expect("LOINC field exceeds u16");
    writer.write_all(&len.to_le_bytes()).expect("write failed");
    writer.write_all(s.as_bytes()).expect("write failed");
}

/// Write a u8-length-prefixed LOINC code string to `writer` (codes are short:
/// term numbers and Part numbers are both well under 255 bytes).
fn write_loinc_code(writer: &mut impl std::io::Write, s: &str) {
    writer
        .write_all(&[u8::try_from(s.len()).expect("LOINC code too long")])
        .expect("write failed");
    writer.write_all(s.as_bytes()).expect("write failed");
}

/// Generate LOINC data by writing compact binary blobs + a tiny .rs shim.
///
/// Avoids generating hundreds of thousands of `map.insert()` lines that
/// would OOM rustc when compiling the 109 K-entry LOINC table.
#[allow(dead_code, clippy::too_many_lines)]
fn generate_loinc_maps(
    descriptions: &HashMap<String, String>,
    components: &HashMap<String, (String, String, String, String, String, Option<String>)>,
    parents: &HashMap<String, String>,
    children: &HashMap<String, Vec<String>>,
    part_names: &HashMap<String, String>,
) {
    use std::io::Write;

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");

    // --- loinc_descriptions.bin ---
    // format: [u32 count] then per entry: [u8 key_len][key][u16 val_len][val]
    {
        let path = std::path::Path::new(&out_dir).join("loinc_descriptions.bin");
        let f = std::fs::File::create(&path).expect("create loinc_descriptions.bin");
        let mut w = std::io::BufWriter::new(f);
        w.write_all(
            &u32::try_from(descriptions.len())
                .expect("too many entries")
                .to_le_bytes(),
        )
        .expect("write");
        let mut entries: Vec<_> = descriptions.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        for (key, val) in &entries {
            w.write_all(&[u8::try_from(key.len()).expect("LOINC key too long")])
                .expect("write");
            w.write_all(key.as_bytes()).expect("write");
            write_loinc_str(&mut w, val);
        }
        w.flush().expect("flush");
    }

    // --- loinc_components.bin ---
    // format: [u32 count] then per entry:
    //   [u8 key_len][key]
    //   [u16 comp_len][comp] [u16 prop_len][prop] [u16 tim_len][tim]
    //   [u16 sys_len][sys]   [u16 scl_len][scl]
    //   [u8 has_method]  (if 1: [u16 meth_len][meth])
    {
        let path = std::path::Path::new(&out_dir).join("loinc_components.bin");
        let f = std::fs::File::create(&path).expect("create loinc_components.bin");
        let mut w = std::io::BufWriter::new(f);
        w.write_all(
            &u32::try_from(components.len())
                .expect("too many entries")
                .to_le_bytes(),
        )
        .expect("write");
        let mut entries: Vec<_> = components.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        for (key, (comp, prop, tim, sys, scl, meth)) in &entries {
            w.write_all(&[u8::try_from(key.len()).expect("LOINC key too long")])
                .expect("write");
            w.write_all(key.as_bytes()).expect("write");
            write_loinc_str(&mut w, comp);
            write_loinc_str(&mut w, prop);
            write_loinc_str(&mut w, tim);
            write_loinc_str(&mut w, sys);
            write_loinc_str(&mut w, scl);
            match meth {
                None => w.write_all(&[0u8]).expect("write"),
                Some(m) => {
                    w.write_all(&[1u8]).expect("write");
                    write_loinc_str(&mut w, m);
                }
            }
        }
        w.flush().expect("flush");
    }

    // --- loinc_parents.bin ---
    // format: [u32 count] then per entry: [u8 key_len][key][u8 has_parent] (if 1: [u8 parent_len][parent])
    {
        let path = std::path::Path::new(&out_dir).join("loinc_parents.bin");
        let f = std::fs::File::create(&path).expect("create loinc_parents.bin");
        let mut w = std::io::BufWriter::new(f);
        w.write_all(
            &u32::try_from(parents.len())
                .expect("too many entries")
                .to_le_bytes(),
        )
        .expect("write");
        let mut entries: Vec<_> = parents.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        for (key, parent) in &entries {
            write_loinc_code(&mut w, key);
            w.write_all(&[1u8]).expect("write");
            write_loinc_code(&mut w, parent);
        }
        w.flush().expect("flush");
    }

    // --- loinc_children.bin ---
    // format: [u32 parent_count] then per entry:
    //   [u8 key_len][key] [u16 num_children] then [u8 child_len][child] * num_children
    {
        let path = std::path::Path::new(&out_dir).join("loinc_children.bin");
        let f = std::fs::File::create(&path).expect("create loinc_children.bin");
        let mut w = std::io::BufWriter::new(f);
        w.write_all(
            &u32::try_from(children.len())
                .expect("too many entries")
                .to_le_bytes(),
        )
        .expect("write");
        let mut entries: Vec<_> = children.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        for (key, kids) in &entries {
            write_loinc_code(&mut w, key);
            w.write_all(
                &u16::try_from(kids.len())
                    .expect("too many children")
                    .to_le_bytes(),
            )
            .expect("write");
            for child in *kids {
                write_loinc_code(&mut w, child);
            }
        }
        w.flush().expect("flush");
    }

    // --- loinc_part_names.bin ---
    // format: identical to loinc_descriptions.bin: [u32 count] then [u8 key_len][key][u16 val_len][val]
    {
        let path = std::path::Path::new(&out_dir).join("loinc_part_names.bin");
        let f = std::fs::File::create(&path).expect("create loinc_part_names.bin");
        let mut w = std::io::BufWriter::new(f);
        w.write_all(
            &u32::try_from(part_names.len())
                .expect("too many entries")
                .to_le_bytes(),
        )
        .expect("write");
        let mut entries: Vec<_> = part_names.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        for (key, val) in &entries {
            write_loinc_code(&mut w, key);
            write_loinc_str(&mut w, val);
        }
        w.flush().expect("flush");
    }

    // --- loinc_data.rs: tiny shim that embeds the blobs and parses at runtime ---
    let out_path = std::path::Path::new(&out_dir).join("loinc_data.rs");
    let rs_code = r#"// Generated by build.rs — do not edit.
// Data lives in binary blobs; only the tiny parser below is compiled by rustc.
use once_cell::sync::Lazy;
use std::collections::HashMap;

static LOINC_DESCRIPTIONS_BIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/loinc_descriptions.bin"));
static LOINC_COMPONENTS_BIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/loinc_components.bin"));
static LOINC_PARENTS_BIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/loinc_parents.bin"));
static LOINC_CHILDREN_BIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/loinc_children.bin"));
static LOINC_PART_NAMES_BIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/loinc_part_names.bin"));

// These parsers only ever read build.rs's own embedded, build-time-generated
// blobs (not user input), so a malformed blob is a build.rs bug, not a
// runtime failure mode — panicking via unwrap() is deliberate here, matching
// build.rs's own #![allow(clippy::expect_used, clippy::panic)] philosophy.
#[allow(clippy::unwrap_used)]
fn loinc_parse_descriptions(data: &'static [u8]) -> HashMap<&'static str, &'static str> {
    let mut pos = 0usize;
    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let mut map = HashMap::with_capacity(count);
    for _ in 0..count {
        let kl = data[pos] as usize;
        pos += 1;
        let key = std::str::from_utf8(&data[pos..pos + kl]).unwrap();
        pos += kl;
        let vl = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2;
        let val = std::str::from_utf8(&data[pos..pos + vl]).unwrap();
        pos += vl;
        let _ = map.insert(key, val);
    }
    map
}

#[allow(clippy::unwrap_used)]
fn loinc_read_str(data: &'static [u8], pos: &mut usize) -> &'static str {
    let l = u16::from_le_bytes(data[*pos..*pos + 2].try_into().unwrap()) as usize;
    *pos += 2;
    let s = std::str::from_utf8(&data[*pos..*pos + l]).unwrap();
    *pos += l;
    s
}

#[allow(clippy::unwrap_used)]
fn loinc_parse_components(
    data: &'static [u8],
) -> HashMap<&'static str, crate::loinc::LoincComponents> {
    let mut pos = 0usize;
    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let mut map = HashMap::with_capacity(count);
    for _ in 0..count {
        let kl = data[pos] as usize;
        pos += 1;
        let key = std::str::from_utf8(&data[pos..pos + kl]).unwrap();
        pos += kl;
        let component = loinc_read_str(data, &mut pos);
        let property  = loinc_read_str(data, &mut pos);
        let timing    = loinc_read_str(data, &mut pos);
        let system    = loinc_read_str(data, &mut pos);
        let scale     = loinc_read_str(data, &mut pos);
        let has_method = data[pos];
        pos += 1;
        let method = if has_method == 1 {
            Some(loinc_read_str(data, &mut pos))
        } else {
            None
        };
        let _ = map.insert(key, crate::loinc::LoincComponents { component, property, timing, system, scale, method });
    }
    map
}

/// LOINC code descriptions keyed by LOINC_NUM.
pub static LOINC_DESCRIPTIONS: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| loinc_parse_descriptions(LOINC_DESCRIPTIONS_BIN));

/// LOINC component details (component/property/timing/system/scale/method) keyed by LOINC_NUM.
pub static LOINC_COMPONENTS: Lazy<HashMap<&'static str, crate::loinc::LoincComponents>> =
    Lazy::new(|| loinc_parse_components(LOINC_COMPONENTS_BIN));

#[allow(clippy::unwrap_used)]
fn loinc_read_code(data: &'static [u8], pos: &mut usize) -> &'static str {
    let l = data[*pos] as usize;
    *pos += 1;
    let s = std::str::from_utf8(&data[*pos..*pos + l]).unwrap();
    *pos += l;
    s
}

#[allow(clippy::unwrap_used)]
fn loinc_parse_parents(data: &'static [u8]) -> HashMap<&'static str, Option<&'static str>> {
    let mut pos = 0usize;
    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let mut map = HashMap::with_capacity(count);
    for _ in 0..count {
        let key = loinc_read_code(data, &mut pos);
        let has_parent = data[pos];
        pos += 1;
        let parent = if has_parent == 1 {
            Some(loinc_read_code(data, &mut pos))
        } else {
            None
        };
        let _ = map.insert(key, parent);
    }
    map
}

#[allow(clippy::unwrap_used)]
fn loinc_parse_children(data: &'static [u8]) -> HashMap<&'static str, Vec<&'static str>> {
    let mut pos = 0usize;
    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let mut map = HashMap::with_capacity(count);
    for _ in 0..count {
        let key = loinc_read_code(data, &mut pos);
        let num_children = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2;
        let mut kids = Vec::with_capacity(num_children);
        for _ in 0..num_children {
            kids.push(loinc_read_code(data, &mut pos));
        }
        let _ = map.insert(key, kids);
    }
    map
}

/// LOINC parent mapping keyed by LOINC code (term or Part number).
pub static LOINC_PARENTS: Lazy<HashMap<&'static str, Option<&'static str>>> =
    Lazy::new(|| loinc_parse_parents(LOINC_PARENTS_BIN));

/// LOINC children mapping keyed by LOINC code (term or Part number).
pub static LOINC_CHILDREN: Lazy<HashMap<&'static str, Vec<&'static str>>> =
    Lazy::new(|| loinc_parse_children(LOINC_CHILDREN_BIN));

/// LOINC Part/group display names (hierarchy branch nodes not present in
/// `LOINC_DESCRIPTIONS`, which only covers terms).
pub static LOINC_PART_NAMES: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| loinc_parse_descriptions(LOINC_PART_NAMES_BIN));
"#;
    std::fs::write(&out_path, rs_code).expect("Failed to write loinc_data.rs");
    eprintln!(
        "Generated LOINC data at {} with {} codes, {} parent edges, {} part names",
        out_path.display(),
        descriptions.len(),
        parents.len(),
        part_names.len()
    );
}

/// Generate empty LOINC maps when data is not available.
#[cfg(feature = "loinc")]
fn generate_empty_loinc_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("loinc_data.rs");

    let empty_code = r"// Generated by build.rs - empty LOINC maps (data file not found)
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static LOINC_DESCRIPTIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(HashMap::new);
pub static LOINC_COMPONENTS: Lazy<HashMap<&'static str, crate::loinc::LoincComponents>> = Lazy::new(HashMap::new);
pub static LOINC_PARENTS: Lazy<HashMap<&'static str, Option<&'static str>>> = Lazy::new(HashMap::new);
pub static LOINC_CHILDREN: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(HashMap::new);
pub static LOINC_PART_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(HashMap::new);
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty LOINC maps: {e}");
    }
    eprintln!("Generated empty LOINC maps at {}", out_path.display());
}

fn generate_icd9cm_data() {
    // Generate ICD-9-CM data
    let icd9_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/icd9cm/sample_codes.csv");

    if Path::new(icd9_data_path).exists() {
        match parse_icd9_csv(icd9_data_path) {
            Ok((codes, parents, children)) => {
                eprintln!("Successfully parsed ICD-9-CM data, generating maps...");
                generate_icd9_maps(&codes, &parents, &children);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse ICD-9-CM CSV: {e}");
                eprintln!("Using empty ICD-9-CM maps.");
                generate_empty_icd9_maps();
            }
        }
    } else {
        eprintln!("Warning: ICD-9-CM data file not found at {icd9_data_path}");
        eprintln!("Using empty ICD-9-CM maps.");
        generate_empty_icd9_maps();
    }
}

fn generate_atc_data() {
    // Generate ATC data
    let atc_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/atc/sample_codes.csv");

    if Path::new(atc_data_path).exists() {
        match parse_atc_csv(atc_data_path) {
            Ok((codes, parents, children, ddd)) => {
                eprintln!("Successfully parsed ATC data, generating maps...");
                generate_atc_maps(&codes, &parents, &children, &ddd);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse ATC CSV: {e}");
                eprintln!("Using empty ATC maps.");
                generate_empty_atc_maps();
            }
        }
    } else {
        eprintln!("Warning: ATC data file not found at {atc_data_path}");
        eprintln!("Using empty ATC maps.");
        generate_empty_atc_maps();
    }
}

fn generate_ndc_data() {
    // Generate NDC data
    let ndc_data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/ndc/sample_codes.csv");

    if Path::new(ndc_data_path).exists() {
        match parse_ndc_csv(ndc_data_path) {
            Ok((codes, labelers, products, packages)) => {
                eprintln!("Successfully parsed NDC data, generating maps...");
                generate_ndc_maps(&codes, &labelers, &products, &packages);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse NDC CSV: {e}");
                eprintln!("Using empty NDC maps.");
                generate_empty_ndc_maps();
            }
        }
    } else {
        eprintln!("Warning: NDC data file not found at {ndc_data_path}");
        eprintln!("Using empty NDC maps.");
        generate_empty_ndc_maps();
    }
}

fn generate_empty_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("icd10cm_data.rs");

    let empty_code = r"
// Generated by build.rs - empty maps
/// ICD-10-CM code descriptions (empty - data not loaded)
pub static ICD10_CM_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {};
/// ICD-10-CM parent code mapping (empty - data not loaded)
pub static ICD10_CM_PARENTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {};
/// ICD-10-CM child code mapping (empty - data not loaded)
pub static ICD10_CM_CHILDREN: phf::Map<&'static str, &'static [&'static str]> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty maps: {e}");
    }
    eprintln!("Generated empty ICD-10-CM maps at {}", out_path.display());
}

fn parse_cms_xml(
    path: &str,
) -> Result<
    (
        Vec<(String, String)>,
        Vec<(String, String)>,
        HashMap<String, Vec<String>>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut reader = Reader::from_file(path)?;
    let mut buf = Vec::new();
    let mut codes = Vec::new();
    let mut parents = Vec::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();

    let mut current_code: Option<String> = None;
    let mut current_desc: Option<String> = None;
    let mut in_name = false;
    let mut in_desc = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => match e.name().as_ref() {
                b"name" => in_name = true,
                b"desc" => in_desc = true,
                _ => {}
            },
            Event::Text(e) => {
                let text = quick_xml::escape::unescape(&e.decode()?)?.to_string();
                if in_name {
                    current_code = Some(text.to_uppercase());
                } else if in_desc {
                    current_desc = Some(text);
                }
            }
            Event::End(e) => {
                match e.name().as_ref() {
                    b"name" => in_name = false,
                    b"desc" => in_desc = false,
                    b"diag" | b"procedure" => {
                        // End of a code entry
                        if let (Some(code), Some(desc)) = (current_code.take(), current_desc.take())
                            && !code.is_empty()
                            && !desc.is_empty()
                        {
                            // Extract parent code by removing the last character after removing dots
                            // This handles codes like "I10.9" -> "I10" and "A01.0" -> "A01"
                            let normalized = code.replace('.', "");
                            if normalized.len() > 3 {
                                let parent_normalized =
                                    normalized[..normalized.len() - 1].to_string();
                                parents.push((code.clone(), parent_normalized.clone()));
                                children
                                    .entry(parent_normalized)
                                    .or_insert_with(Vec::new)
                                    .push(code.clone());
                            } else {
                                // Top-level codes have no parent
                                parents.push((code.clone(), String::new()));
                            }
                            codes.push((code, desc));
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok((codes, parents, children))
}

fn generate_phf_maps(
    codes: &[(String, String)],
    parents: &[(String, String)],
    children: &HashMap<String, Vec<String>>,
) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("icd10cm_data.rs");

    eprintln!("Parsed {} ICD-10-CM codes", codes.len());
    eprintln!("Found {} parent-child relationships", children.len());

    // Generate Rust code
    let mut output = String::new();
    output.push_str("// Generated by build.rs from CMS ICD-10-CM data\n\n");

    // Generate descriptions map - store descriptions and create Code structs on demand
    output.push_str("/// ICD-10-CM code descriptions.\n");
    output.push_str("/// Generated from CMS April 1, 2026 code tables.\n");
    output.push_str(
        "pub static ICD10_CM_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for (code, desc) in codes {
        let escaped_desc = desc
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        let _ = writeln!(output, "    \"{code}\" => \"{escaped_desc}\",");
    }
    output.push_str("};\n\n");

    // Generate parents map
    output.push_str("/// ICD-10-CM parent code mapping for hierarchy traversal.\n");
    output.push_str("/// Maps normalized codes to their immediate parent codes.\n");
    output.push_str(
        "pub static ICD10_CM_PARENTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {\n",
    );
    for (code, parent) in parents {
        if parent.is_empty() {
            let _ = writeln!(output, "    \"{code}\" => None,");
        } else {
            let _ = writeln!(output, "    \"{code}\" => Some(\"{parent}\"),");
        }
    }
    output.push_str("};\n\n");

    // Generate children map
    output.push_str("/// ICD-10-CM child code mapping for hierarchy traversal.\n");
    output.push_str("/// Maps normalized codes to their immediate child codes.\n");
    output.push_str("pub static ICD10_CM_CHILDREN: phf::Map<&'static str, &'static [&'static str]> = phf_map! {\n");
    for (parent, child_list) in children {
        // Sort and deduplicate child list for deterministic builds
        let mut sorted_children = child_list.clone();
        sorted_children.sort();
        sorted_children.dedup();

        let children_str = sorted_children
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(output, "    \"{parent}\" => &[{children_str}],");
    }
    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write ICD-10-CM data: {e}");
    }
    eprintln!("Generated ICD-10-CM data at {}", out_path.display());
}

// CCSR data structures and parsing

#[derive(Debug, Clone)]
struct CcsrCodeMapping {
    icd10_code: String,
    categories: Vec<ParsedCcsrCategory>,
}

#[derive(Debug, Clone)]
struct ParsedCcsrCategory {
    code: String,
    description: String,
    is_default_ip: bool,
    is_default_ed: bool,
    is_default_op: bool,
}

fn parse_ccsr_csv(path: &str) -> Result<Vec<CcsrCodeMapping>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut mappings: HashMap<String, CcsrCodeMapping> = HashMap::new();

    for result in reader.records() {
        let record = result?;

        // Parse ICD-10-CM code (column 0)
        let icd10_code = record
            .get(0)
            .ok_or("Missing ICD-10-CM code")?
            .trim()
            .to_uppercase();

        if icd10_code.is_empty() {
            continue;
        }

        // Parse default categories for different contexts
        let default_inpatient = record.get(2).unwrap_or("").trim();
        let default_ed = record.get(4).unwrap_or("").trim();
        let default_outpatient = record.get(6).unwrap_or("").trim();

        // Parse all CCSR categories (up to 6)
        let mut categories = Vec::new();
        for i in 0..6 {
            let cat_col = 8 + i * 2; // Categories start at column 8
            let desc_col = cat_col + 1;

            if let Some(cat_code) = record.get(cat_col) {
                let cat_code = cat_code.trim();
                if !cat_code.is_empty() && cat_code != "' '" {
                    let description = record.get(desc_col).unwrap_or("").trim().to_string();

                    let is_default_inpatient = default_inpatient == cat_code;
                    let is_default_ed = default_ed == cat_code;
                    let is_default_outpatient = default_outpatient == cat_code;

                    categories.push(ParsedCcsrCategory {
                        code: cat_code.replace('\'', ""),
                        description,
                        is_default_ip: is_default_inpatient,
                        is_default_ed,
                        is_default_op: is_default_outpatient,
                    });
                }
            }
        }

        if !categories.is_empty() {
            let _ = mappings.insert(
                icd10_code.clone(),
                CcsrCodeMapping {
                    icd10_code,
                    categories,
                },
            );
        }
    }

    let result: Vec<CcsrCodeMapping> = mappings.into_values().collect();
    eprintln!("Parsed {} ICD-10-CM to CCSR mappings", result.len());
    Ok(result)
}

fn generate_empty_ccsr_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ccsr_data.rs");

    let empty_code = r"
// Generated by build.rs - empty CCSR maps
use phf::phf_map;

/// All CCSR mappings (empty - data not loaded)
pub static CCSR_MAPPINGS: &[CcsrMapping] = &[];

/// ICD-10-CM to CCSR mappings (empty - data not loaded)
pub static ICD10CM_TO_CCSR_MAPPINGS: phf::Map<&'static str, &'static [usize]> = phf_map! {};
/// CCSR to ICD-10-CM reverse mappings (empty - data not loaded)
pub static CCSR_TO_ICD10CM_MAPPINGS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty CCSR maps: {e}");
    }
    eprintln!("Generated empty CCSR maps at {}", out_path.display());
}

fn generate_ccsr_maps(mappings: &[CcsrCodeMapping]) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ccsr_data.rs");

    let mut output = String::new();
    output.push_str("// Generated by build.rs from AHRQ CCSR data\n\n");
    output.push_str("use phf::phf_map;\n\n");

    // Generate all CCSR mappings as a single large static array
    output.push_str("static CCSR_MAPPINGS: &[crate::ccsr::CcsrMapping] = &[\n");

    for mapping in mappings {
        for cat in &mapping.categories {
            let _ = writeln!(
                output,
                "    crate::ccsr::CcsrMapping {{\n        category_code: \"{}\",\n        category_description: \"{}\",\n        is_default_ip: {},\n        is_default_ed: {},\n        is_default_op: {},\n    }},",
                cat.code.replace('"', "\\\""),
                cat.description.replace('"', "\\\""),
                cat.is_default_ip,
                cat.is_default_ed,
                cat.is_default_op
            );
        }
    }

    output.push_str("];\n\n");

    // Generate index map for ICD-10-CM -> CCSR
    output.push_str("/// ICD-10-CM to CCSR mappings\n");
    output.push_str("/// Maps normalized ICD-10-CM codes to indices in `CCSR_MAPPINGS` array\n");
    output.push_str("pub static ICD10CM_TO_CCSR_MAPPINGS: phf::Map<&'static str, &'static [usize]> = phf_map! {\n");

    let mut current_index = 0;
    for mapping in mappings {
        let count = mapping.categories.len();
        if count > 0 {
            let indices: Vec<String> = (current_index..current_index + count)
                .map(|i| i.to_string())
                .collect();
            let _ = writeln!(
                output,
                "    \"{}\" => &[{}],",
                mapping.icd10_code,
                indices.join(", ")
            );
            current_index += count;
        }
    }

    output.push_str("};\n\n");

    // Generate reverse mapping (CCSR -> ICD-10-CM)
    let mut reverse_mappings: HashMap<String, Vec<String>> = HashMap::new();
    for mapping in mappings {
        for cat in &mapping.categories {
            reverse_mappings
                .entry(cat.code.clone())
                .or_insert_with(Vec::new)
                .push(mapping.icd10_code.clone());
        }
    }

    // Sort and deduplicate ICD-10-CM codes for each CCSR category
    for icd10_codes in reverse_mappings.values_mut() {
        icd10_codes.sort();
        icd10_codes.dedup();
    }

    output.push_str("/// CCSR to ICD-10-CM reverse mappings\n");
    output.push_str("/// Maps CCSR categories to their ICD-10-CM codes\n");
    output.push_str("pub static CCSR_TO_ICD10CM_MAPPINGS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {\n");

    for (ccsr_code, icd10_codes) in &reverse_mappings {
        let icd10_str = icd10_codes
            .iter()
            .map(|code| format!("\"{code}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(output, "    \"{ccsr_code}\" => &[{icd10_str}],");
    }

    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write CCSR data: {e}");
    }
    eprintln!(
        "Generated CCSR data at {} with {} mappings",
        out_path.display(),
        mappings.len()
    );
}

// ICD-9-CM data structures and parsing

fn parse_icd9_csv(
    path: &str,
) -> Result<
    (
        Vec<(String, String)>,
        Vec<(String, Option<String>)>,
        HashMap<String, Vec<String>>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut codes = Vec::new();
    let mut parents = Vec::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();

    for result in reader.records() {
        let record = result?;

        // Parse code (column 0)
        let code = record
            .get(0)
            .ok_or("Missing ICD-9-CM code")?
            .trim()
            .to_uppercase();

        if code.is_empty() {
            continue;
        }

        // Parse description (column 1)
        let description = record
            .get(1)
            .ok_or("Missing description")?
            .trim()
            .to_string();

        // Parse parent (column 2) - may be empty for top-level codes
        let parent = record.get(2).unwrap_or("").trim().to_string();

        // Store the code and description
        codes.push((code.clone(), description));

        // Store parent relationship
        if parent.is_empty() {
            parents.push((code.clone(), None));
        } else {
            parents.push((code.clone(), Some(parent.clone())));
            // Add to parent's children list
            children.entry(parent).or_insert_with(Vec::new).push(code);
        }
    }

    Ok((codes, parents, children))
}

fn generate_empty_icd9_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("icd9cm_data.rs");

    let empty_code = r"
// Generated by build.rs - empty ICD-9-CM maps
/// ICD-9-CM code descriptions (empty - data not loaded)
pub static ICD9_CM_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {};
/// ICD-9-CM parent code mapping (empty - data not loaded)
pub static ICD9_CM_PARENTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {};
/// ICD-9-CM child code mapping (empty - data not loaded)
pub static ICD9_CM_CHILDREN: phf::Map<&'static str, &'static [&'static str]> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty ICD-9-CM maps: {e}");
    }
    eprintln!("Generated empty ICD-9-CM maps at {}", out_path.display());
}

fn generate_icd9_maps(
    codes: &[(String, String)],
    parents: &[(String, Option<String>)],
    children: &HashMap<String, Vec<String>>,
) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("icd9cm_data.rs");

    eprintln!("Parsed {} ICD-9-CM codes", codes.len());
    eprintln!("Found {} parent-child relationships", children.len());

    // Generate Rust code
    let mut output = String::new();
    output.push_str("// Generated by build.rs from ICD-9-CM data\n\n");

    // Generate descriptions map
    output.push_str("/// ICD-9-CM code descriptions.\n");
    output.push_str("/// Generated from ICD-9-CM sample data.\n");
    output.push_str(
        "pub static ICD9_CM_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for (code, desc) in codes {
        let escaped_desc = desc
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        let _ = writeln!(output, "    \"{code}\" => \"{escaped_desc}\",");
    }
    output.push_str("};\n\n");

    // Generate parents map
    output.push_str("/// ICD-9-CM parent code mapping for hierarchy traversal.\n");
    output.push_str("/// Maps normalized codes to their immediate parent codes.\n");
    output.push_str(
        "pub static ICD9_CM_PARENTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {\n",
    );
    for (code, parent) in parents {
        match parent {
            None => {
                let _ = writeln!(output, "    \"{code}\" => None,");
            }
            Some(p) => {
                let _ = writeln!(output, "    \"{code}\" => Some(\"{p}\"),");
            }
        }
    }
    output.push_str("};\n\n");

    // Generate children map
    output.push_str("/// ICD-9-CM child code mapping for hierarchy traversal.\n");
    output.push_str("/// Maps normalized codes to their immediate child codes.\n");
    output.push_str("pub static ICD9_CM_CHILDREN: phf::Map<&'static str, &'static [&'static str]> = phf_map! {\n");
    for (parent, child_list) in children {
        // Sort and deduplicate child list for deterministic builds
        let mut sorted_children = child_list.clone();
        sorted_children.sort();
        sorted_children.dedup();

        let children_str = sorted_children
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(output, "    \"{parent}\" => &[{children_str}],");
    }
    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write ICD-9-CM data: {e}");
    }
    eprintln!("Generated ICD-9-CM data at {}", out_path.display());
}

// ATC data structures and parsing

fn parse_atc_csv(
    path: &str,
) -> Result<
    (
        Vec<(String, String)>,
        Vec<(String, Option<String>)>,
        HashMap<String, Vec<String>>,
        Vec<(String, String)>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut codes = Vec::new();
    let mut parents = Vec::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    let mut ddd = Vec::new();

    for result in reader.records() {
        let record = result?;

        // Parse code (column 0)
        let code = record
            .get(0)
            .ok_or("Missing ATC code")?
            .trim()
            .to_uppercase();

        if code.is_empty() {
            continue;
        }

        // Validate ATC code format before processing
        if !is_valid_atc_format(&code) {
            eprintln!("Warning: Skipping invalid ATC code format: {code}");
            continue;
        }

        // Parse description (column 1)
        let description = record
            .get(1)
            .ok_or("Missing description")?
            .trim()
            .to_string();

        // Parse parent (column 2) - may be empty for top-level codes
        let parent = record.get(2).unwrap_or("").trim().to_string();

        // Parse DDD (column 3) - optional
        let ddd_value = record.get(3).unwrap_or("").trim().to_string();

        // Store the code and description
        codes.push((code.clone(), description));

        // Store DDD if present
        if !ddd_value.is_empty() {
            ddd.push((code.clone(), ddd_value));
        }

        // Store parent relationship
        if parent.is_empty() {
            parents.push((code, None));
        } else {
            parents.push((code.clone(), Some(parent.clone())));
            // Add to parent's children list
            children.entry(parent).or_insert_with(Vec::new).push(code);
        }
    }

    Ok((codes, parents, children, ddd))
}

/// Validate ATC code format (same logic as in atc/mod.rs but without needing the struct)
fn is_valid_atc_format(code: &str) -> bool {
    let bytes = code.as_bytes();

    if bytes.is_empty() || bytes.len() > 7 {
        return false;
    }

    // First character must be an uppercase letter
    if !bytes[0].is_ascii_uppercase() {
        return false;
    }

    // Check based on length
    match bytes.len() {
        1 => true, // Just the anatomical group (e.g., "C")
        3 => {
            // Anatomical + 2 digits (e.g., "C10")
            bytes[1].is_ascii_digit() && bytes[2].is_ascii_digit()
        }
        4 => {
            // + 1 letter (e.g., "C10A")
            bytes[1].is_ascii_digit() && bytes[2].is_ascii_digit() && bytes[3].is_ascii_uppercase()
        }
        5 => {
            // + 1 letter (e.g., "C10AA")
            bytes[1].is_ascii_digit()
                && bytes[2].is_ascii_digit()
                && bytes[3].is_ascii_uppercase()
                && bytes[4].is_ascii_uppercase()
        }
        7 => {
            // + 2 digits (e.g., "C10AA01")
            bytes[1].is_ascii_digit()
                && bytes[2].is_ascii_digit()
                && bytes[3].is_ascii_uppercase()
                && bytes[4].is_ascii_uppercase()
                && bytes[5].is_ascii_digit()
                && bytes[6].is_ascii_digit()
        }
        _ => false, // Invalid length (2, 6)
    }
}

fn generate_empty_atc_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("atc_data.rs");

    let empty_code = r"
// Generated by build.rs - empty ATC maps
/// ATC code descriptions (empty - data not loaded)
pub static ATC_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {};
/// ATC parent code mapping (empty - data not loaded)
pub static ATC_PARENTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {};
/// ATC child code mapping (empty - data not loaded)
pub static ATC_CHILDREN: phf::Map<&'static str, &'static [&'static str]> = phf_map! {};
/// ATC DDD values (empty - data not loaded)
pub static ATC_DDD_VALUES: phf::Map<&'static str, &'static str> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty ATC maps: {e}");
    }
    eprintln!("Generated empty ATC maps at {}", out_path.display());
}

fn generate_atc_maps(
    codes: &[(String, String)],
    parents: &[(String, Option<String>)],
    children: &HashMap<String, Vec<String>>,
    ddd: &[(String, String)],
) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("atc_data.rs");

    eprintln!("Parsed {} ATC codes", codes.len());
    eprintln!("Found {} parent-child relationships", children.len());
    eprintln!("Found {} DDD values", ddd.len());

    // Generate Rust code
    let mut output = String::new();
    output.push_str("// Generated by build.rs from ATC data\n\n");

    // Generate descriptions map
    output.push_str("/// ATC code descriptions.\n");
    output.push_str("/// Generated from ATC sample data.\n");
    output.push_str(
        "pub static ATC_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for (code, desc) in codes {
        let escaped_desc = desc
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        let _ = writeln!(output, "    \"{code}\" => \"{escaped_desc}\",");
    }
    output.push_str("};\n\n");

    // Generate parents map
    output.push_str("/// ATC parent code mapping for hierarchy traversal.\n");
    output.push_str("/// Maps normalized codes to their immediate parent codes.\n");
    output.push_str(
        "pub static ATC_PARENTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {\n",
    );
    for (code, parent) in parents {
        match parent {
            None => {
                let _ = writeln!(output, "    \"{code}\" => None,");
            }
            Some(p) => {
                let _ = writeln!(output, "    \"{code}\" => Some(\"{p}\"),");
            }
        }
    }
    output.push_str("};\n\n");

    // Generate children map
    output.push_str("/// ATC child code mapping for hierarchy traversal.\n");
    output.push_str("/// Maps normalized codes to their immediate child codes.\n");
    output.push_str(
        "pub static ATC_CHILDREN: phf::Map<&'static str, &'static [&'static str]> = phf_map! {\n",
    );
    for (parent, child_list) in children {
        // Sort and deduplicate child list for deterministic builds
        let mut sorted_children = child_list.clone();
        sorted_children.sort();
        sorted_children.dedup();

        let children_str = sorted_children
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(output, "    \"{parent}\" => &[{children_str}],");
    }
    output.push_str("};\n\n");

    // Generate DDD values map
    output.push_str("/// ATC DDD (Defined Daily Dose) values.\n");
    output.push_str("/// Maps ATC codes to their DDD values.\n");
    output
        .push_str("pub static ATC_DDD_VALUES: phf::Map<&'static str, &'static str> = phf_map! {\n");
    for (code, ddd_value) in ddd {
        let _ = writeln!(output, "    \"{code}\" => \"{ddd_value}\",");
    }
    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write ATC data: {e}");
    }
    eprintln!("Generated ATC data at {}", out_path.display());
}
/// Parse NDC CSV data.
fn parse_ndc_csv(
    path: &str,
) -> Result<
    (
        Vec<(String, String)>,
        Vec<(String, Option<String>)>,
        Vec<(String, Option<String>)>,
        Vec<(String, Option<String>)>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut codes = Vec::new();
    let mut labelers = Vec::new();
    let mut products = Vec::new();
    let mut packages = Vec::new();

    for result in reader.records() {
        let record = result?;

        // Parse NDC code (column 0)
        let ndc_code = record
            .get(0)
            .ok_or("Missing NDC code")?
            .trim()
            .to_uppercase();

        if ndc_code.is_empty() {
            continue;
        }

        // Validate NDC code format before processing
        if !is_valid_ndc_format(&ndc_code) {
            eprintln!("Warning: Skipping invalid NDC code format: {ndc_code}");
            continue;
        }

        // Parse description (column 1)
        let description = record
            .get(1)
            .ok_or("Missing description")?
            .trim()
            .to_string();

        // Parse labeler (column 2)
        let labeler = record.get(2).unwrap_or("").trim().to_string();

        // Parse product (column 3)
        let product = record.get(3).unwrap_or("").trim().to_string();

        // Parse package (column 4)
        let package = record.get(4).unwrap_or("").trim().to_string();

        // Store the code and description
        codes.push((ndc_code.clone(), description));

        // Store labeler relationship
        if labeler.is_empty() {
            labelers.push((ndc_code.clone(), None));
        } else {
            labelers.push((ndc_code.clone(), Some(labeler)));
        }

        // Store product relationship
        if product.is_empty() {
            products.push((ndc_code.clone(), None));
        } else {
            products.push((ndc_code.clone(), Some(product)));
        }

        // Store package relationship
        if package.is_empty() {
            packages.push((ndc_code, None));
        } else {
            packages.push((ndc_code, Some(package)));
        }
    }

    Ok((codes, labelers, products, packages))
}

/// Validate NDC code format.
fn is_valid_ndc_format(code: &str) -> bool {
    let parts: Vec<&str> = code.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    // Check each part contains only digits
    parts
        .iter()
        .all(|part| part.chars().all(|c| c.is_ascii_digit()))
}

/// Generate empty NDC maps.
fn generate_empty_ndc_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ndc_data.rs");

    let empty_code = r"
// Generated by build.rs - empty maps
/// NDC code descriptions (empty - data not loaded)
pub static NDC_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {};
/// NDC labeler relationships (empty - data not loaded)
pub static NDC_LABELERS: phf::Map<&'static str, Option<&'static str>> = phf_map! {};
/// NDC product relationships (empty - data not loaded)
pub static NDC_PRODUCTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {};
/// NDC package relationships (empty - data not loaded)
pub static NDC_PACKAGES: phf::Map<&'static str, Option<&'static str>> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty NDC data: {e}");
    }
    eprintln!("Generated empty NDC data at {}", out_path.display());
}

/// Generate NDC PHF maps.
fn generate_ndc_maps(
    codes: &[(String, String)],
    labelers: &[(String, Option<String>)],
    products: &[(String, Option<String>)],
    packages: &[(String, Option<String>)],
) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ndc_data.rs");

    let mut output = String::new();
    output.push_str("// Generated by build.rs - do not edit\n");
    output.push_str("use phf::phf_map;\n\n");

    // Generate descriptions map
    output.push_str("/// NDC code descriptions.\n");
    output.push_str("/// Maps NDC codes to their drug descriptions.\n");
    output.push_str(
        "pub static NDC_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for (code, description) in codes {
        let _ = writeln!(output, "    \"{code}\" => \"{description}\",");
    }
    output.push_str("};\n\n");

    // Generate labelers map
    output.push_str("/// NDC labeler relationships.\n");
    output.push_str("/// Maps NDC codes to their labeler codes.\n");
    output.push_str(
        "pub static NDC_LABELERS: phf::Map<&'static str, Option<&'static str>> = phf_map! {\n",
    );
    for (code, labeler) in labelers {
        match labeler {
            Some(labeler) => {
                let _ = writeln!(output, "    \"{code}\" => Some(\"{labeler}\"),");
            }
            None => {
                let _ = writeln!(output, "    \"{code}\" => None,");
            }
        }
    }
    output.push_str("};\n\n");

    // Generate products map
    output.push_str("/// NDC product relationships.\n");
    output.push_str("/// Maps NDC codes to their product codes.\n");
    output.push_str(
        "pub static NDC_PRODUCTS: phf::Map<&'static str, Option<&'static str>> = phf_map! {\n",
    );
    for (code, product) in products {
        match product {
            Some(product) => {
                let _ = writeln!(output, "    \"{code}\" => Some(\"{product}\"),");
            }
            None => {
                let _ = writeln!(output, "    \"{code}\" => None,");
            }
        }
    }
    output.push_str("};\n\n");

    // Generate packages map
    output.push_str("/// NDC package relationships.\n");
    output.push_str("/// Maps NDC codes to their package codes.\n");
    output.push_str(
        "pub static NDC_PACKAGES: phf::Map<&'static str, Option<&'static str>> = phf_map! {\n",
    );
    for (code, package) in packages {
        match package {
            Some(package) => {
                let _ = writeln!(output, "    \"{code}\" => Some(\"{package}\"),");
            }
            None => {
                let _ = writeln!(output, "    \"{code}\" => None,");
            }
        }
    }
    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write NDC data: {e}");
    }
    eprintln!("Generated NDC data at {}", out_path.display());
}

// CCS data structures and parsing

#[derive(Debug, Clone)]
struct CcsMapping {
    code: String,
    ccs_code: String,
}

fn parse_ccs_csv(path: &str) -> Result<Vec<CcsMapping>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut mappings = Vec::new();

    for result in reader.records() {
        let record = result?;

        // Parse code (column 0)
        let code = record.get(0).ok_or("Missing code")?.trim().to_uppercase();

        if code.is_empty() {
            continue;
        }

        // Parse CCS code (column 1)
        let ccs_code = record.get(1).ok_or("Missing CCS code")?.trim().to_string();

        if ccs_code.is_empty() {
            continue;
        }

        mappings.push(CcsMapping { code, ccs_code });
    }

    eprintln!("Parsed {} CCS mappings", mappings.len());
    Ok(mappings)
}

fn generate_empty_ccs_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ccs_data.rs");

    let empty_code = r"
// Generated by build.rs - empty CCS maps
/// CCS category descriptions (empty - data not loaded)
pub static CCS_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {};

/// ICD-10-CM to CCS mappings (empty - data not loaded)
pub static ICD10CM_TO_CCS_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {};

/// ICD-9-CM to CCS mappings (empty - data not loaded)
pub static ICD9CM_TO_CCS_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty CCS maps: {e}");
    }
    eprintln!("Generated empty CCS maps at {}", out_path.display());
}

fn generate_ccs_maps(icd10cm_mappings: &[CcsMapping], icd9cm_mappings: &[CcsMapping]) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ccs_data.rs");

    let mut output = String::new();
    output.push_str("// Generated by build.rs from CCS data\n\n");

    // Collect all unique CCS codes from both mappings
    let mut all_ccs_codes: std::collections::HashSet<&String> = std::collections::HashSet::new();
    for mapping in icd10cm_mappings {
        let _ = all_ccs_codes.insert(&mapping.ccs_code);
    }
    for mapping in icd9cm_mappings {
        let _ = all_ccs_codes.insert(&mapping.ccs_code);
    }

    // Generate CCS descriptions (using placeholder descriptions based on category numbers)
    output.push_str("/// CCS category descriptions.\n");
    output.push_str("/// Generated from CCS mapping data.\n");
    output.push_str(
        "pub static CCS_DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );

    let mut ccs_codes: Vec<_> = all_ccs_codes.iter().collect();
    ccs_codes.sort();

    for ccs_code in ccs_codes {
        let description = format!("CCS Category {ccs_code}");
        let _ = writeln!(output, "    \"{ccs_code}\" => \"{description}\",");
    }
    output.push_str("};\n\n");

    // Generate ICD-10-CM to CCS mappings
    output.push_str("/// ICD-10-CM to CCS mappings.\n");
    output.push_str("/// Maps ICD-10-CM codes to CCS categories.\n");
    output.push_str(
        "pub static ICD10CM_TO_CCS_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for mapping in icd10cm_mappings {
        let _ = writeln!(
            output,
            "    \"{}\" => \"{}\",",
            mapping.code, mapping.ccs_code
        );
    }
    output.push_str("};\n\n");

    // Generate ICD-9-CM to CCS mappings
    output.push_str("/// ICD-9-CM to CCS mappings.\n");
    output.push_str("/// Maps ICD-9-CM codes to CCS categories.\n");
    output.push_str(
        "pub static ICD9CM_TO_CCS_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for mapping in icd9cm_mappings {
        let _ = writeln!(
            output,
            "    \"{}\" => \"{}\",",
            mapping.code, mapping.ccs_code
        );
    }
    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write CCS data: {e}");
    }
    eprintln!(
        "Generated CCS data at {} with {} ICD-10-CM and {} ICD-9-CM mappings",
        out_path.display(),
        icd10cm_mappings.len(),
        icd9cm_mappings.len()
    );
}

fn generate_ndc_to_atc_data() {
    // Generate NDC to ATC mapping data
    let ndc_to_atc_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ndc/ndc_to_atc_mappings.csv"
    );

    if Path::new(ndc_to_atc_path).exists() {
        match parse_ndc_to_atc_csv(ndc_to_atc_path) {
            Ok(mappings) => {
                eprintln!("Successfully parsed NDC to ATC data, generating mappings...");
                generate_ndc_to_atc_maps(&mappings);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse NDC to ATC CSV: {e}");
                eprintln!("Using empty NDC to ATC mappings.");
                generate_empty_ndc_to_atc_maps();
            }
        }
    } else {
        eprintln!("Warning: NDC to ATC data file not found at {ndc_to_atc_path}");
        eprintln!("Using empty NDC to ATC mappings.");
        generate_empty_ndc_to_atc_maps();
    }
}

#[derive(Debug, Clone)]
struct NdcToAtcMapping {
    ndc_code: String,
    atc_code: String,
}

fn parse_ndc_to_atc_csv(path: &str) -> Result<Vec<NdcToAtcMapping>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut mappings = Vec::new();

    for result in reader.records() {
        let record = result?;

        // Parse NDC code (column 0)
        let ndc_code = record
            .get(0)
            .ok_or("Missing NDC code")?
            .trim()
            .replace('-', "")
            .to_uppercase();

        if ndc_code.is_empty() {
            continue;
        }

        // Parse ATC code (column 1)
        let atc_code = record.get(1).ok_or("Missing ATC code")?.trim().to_string();

        if atc_code.is_empty() {
            continue;
        }

        mappings.push(NdcToAtcMapping { ndc_code, atc_code });
    }

    eprintln!("Parsed {} NDC to ATC mappings", mappings.len());
    Ok(mappings)
}

fn generate_empty_ndc_to_atc_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ndc_to_atc_data.rs");

    let empty_code = r"
// Generated by build.rs - empty NDC to ATC maps
/// NDC to ATC mappings (empty - data not loaded)
pub static NDC_TO_ATC_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty NDC to ATC maps: {e}");
    }
    eprintln!("Generated empty NDC to ATC maps at {}", out_path.display());
}

fn generate_ndc_to_atc_maps(mappings: &[NdcToAtcMapping]) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ndc_to_atc_data.rs");

    let mut output = String::new();
    output.push_str("// Generated by build.rs from NDC to ATC data\n\n");

    // Generate NDC to ATC mappings
    output.push_str("/// NDC to ATC mappings.\n");
    output.push_str("/// Maps NDC codes to ATC codes.\n");
    output.push_str(
        "pub static NDC_TO_ATC_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for mapping in mappings {
        let _ = writeln!(
            output,
            "    \"{}\" => \"{}\",",
            mapping.ndc_code, mapping.atc_code
        );
    }
    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write NDC to ATC data: {e}");
    }
    eprintln!(
        "Generated NDC to ATC data at {} with {} mappings",
        out_path.display(),
        mappings.len()
    );
}

fn generate_ndc_to_rxnorm_data() {
    // Generate NDC to RxNorm mapping data
    let ndc_to_rxnorm_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ndc/ndc_to_rxnorm_mappings.csv"
    );

    if Path::new(ndc_to_rxnorm_path).exists() {
        match parse_ndc_to_rxnorm_csv(ndc_to_rxnorm_path) {
            Ok(mappings) => {
                eprintln!("Successfully parsed NDC to RxNorm data, generating mappings...");
                generate_ndc_to_rxnorm_maps(&mappings);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse NDC to RxNorm CSV: {e}");
                eprintln!("Using empty NDC to RxNorm mappings.");
                generate_empty_ndc_to_rxnorm_maps();
            }
        }
    } else {
        eprintln!("Warning: NDC to RxNorm data file not found at {ndc_to_rxnorm_path}");
        eprintln!("Using empty NDC to RxNorm mappings.");
        generate_empty_ndc_to_rxnorm_maps();
    }
}

#[derive(Debug, Clone)]
struct NdcToRxNormMapping {
    ndc_code: String,
    rxnorm_code: String,
}

fn parse_ndc_to_rxnorm_csv(
    path: &str,
) -> Result<Vec<NdcToRxNormMapping>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut mappings = Vec::new();

    for result in reader.records() {
        let record = result?;

        // Parse NDC code (column 0)
        let ndc_code = record
            .get(0)
            .ok_or("Missing NDC code")?
            .trim()
            .replace('-', "")
            .to_uppercase();

        if ndc_code.is_empty() {
            continue;
        }

        // Parse RxNorm code (column 1)
        let rxnorm_code = record
            .get(1)
            .ok_or("Missing RxNorm code")?
            .trim()
            .to_string();

        if rxnorm_code.is_empty() {
            continue;
        }

        mappings.push(NdcToRxNormMapping {
            ndc_code,
            rxnorm_code,
        });
    }

    eprintln!("Parsed {} NDC to RxNorm mappings", mappings.len());
    Ok(mappings)
}

fn generate_empty_ndc_to_rxnorm_maps() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ndc_to_rxnorm_data.rs");

    let empty_code = r"
// Generated by build.rs - empty NDC to RxNorm maps
/// NDC to RxNorm mappings (empty - data not loaded)
pub static NDC_TO_RXNORM_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {};
";

    if let Err(e) = std::fs::write(&out_path, empty_code) {
        panic!("Failed to write empty NDC to RxNorm maps: {e}");
    }
    eprintln!(
        "Generated empty NDC to RxNorm maps at {}",
        out_path.display()
    );
}

fn generate_ndc_to_rxnorm_maps(mappings: &[NdcToRxNormMapping]) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let out_path = std::path::Path::new(&out_dir).join("ndc_to_rxnorm_data.rs");

    let mut output = String::new();
    output.push_str("// Generated by build.rs from NDC to RxNorm data\n\n");

    // Generate NDC to RxNorm mappings
    output.push_str("/// NDC to `RxNorm` mappings.\n");
    output.push_str("/// Maps NDC codes to `RxNorm` codes.\n");
    output.push_str(
        "pub static NDC_TO_RXNORM_MAPPINGS: phf::Map<&'static str, &'static str> = phf_map! {\n",
    );
    for mapping in mappings {
        let _ = writeln!(
            output,
            "    \"{}\" => \"{}\",",
            mapping.ndc_code, mapping.rxnorm_code
        );
    }
    output.push_str("};\n");

    if let Err(e) = std::fs::write(&out_path, output) {
        panic!("Failed to write NDC to RxNorm data: {e}");
    }
    eprintln!(
        "Generated NDC to RxNorm data at {} with {} mappings",
        out_path.display(),
        mappings.len()
    );
}
