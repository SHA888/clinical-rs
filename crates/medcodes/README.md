# medcodes

Medical code ontologies, hierarchy traversal, and cross-system mapping.

## Features

- `icd10cm` - ICD-10-CM code definitions and hierarchy traversal (default)
- `ccsr` - Clinical Classifications Software Refined cross-mapping
- `serde` - Serialization support for all code types

## Usage

### ICD-10-CM Code Lookup and Hierarchy

```text
use medcodes::{icd10::Icd10Cm, CodeSystem};

let icd10 = Icd10Cm::new();

// Look up a code
let code = icd10.lookup("A00.0").unwrap();
println!("{}: {}", code.code(), code.description());

// Check if a code is valid
assert!(icd10.is_valid("A00.0"));
assert!(!icd10.is_valid("INVALID"));

// Normalize a code (remove dots and whitespace, uppercase)
assert_eq!(icd10.normalize(" a 00 .0 "), "A000");

// Traverse the hierarchy
let ancestors = icd10.ancestors("A00.0").unwrap();
for ancestor in ancestors {
    println!("Parent: {} - {}", ancestor.code(), ancestor.description());
}

let parent = icd10.parent("A00.0").unwrap();
if let Some(parent) = parent {
    println!("Immediate parent: {}", parent.code());
}

let children = icd10.children("A00").unwrap();
for child in children {
    println!("Child: {} - {}", child.code(), child.description());
}
```

### Cross-System Mapping (ICD-10-CM ↔ CCSR)

```text
use medcodes::ccsr::{Icd10CmToCcsr, CcsrToIcd10Cm, CcsrContext};

// ICD-10-CM to CCSR
let forward_mapper = Icd10CmToCcsr::new();
let categories = forward_mapper.get_categories("A00.0").unwrap();
for category in categories {
    println!("CCSR: {} - {}", category.code, category.description);
}

// Get default category for a specific context
let default_category = forward_mapper.get_default_category(
    "A00.0",
    CcsrContext::Inpatient
).unwrap();

// CCSR to ICD-10-CM
let reverse_mapper = CcsrToIcd10Cm::new();
let icd10_codes = reverse_mapper.get_icd10_codes("DIG001").unwrap();
for code in icd10_codes {
    println!("ICD-10-CM: {}", code);
}
```

### Using the `CrossMap` Trait

```text
use medcodes::{CodeSystem, System, CrossMap};
use medcodes::ccsr::Icd10CmToCcsr;

let mapper = Icd10CmToCcsr::new();

// Map between systems
let mapped_codes = mapper.map("A00.0", System::Ccsr).unwrap();
```

## Data Sources

- **ICD-10-CM**: CMS FY2025 release (October 1, 2024)
- **CCSR**: AHRQ v2024.1 and v2026.1 mapping files

## License

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
