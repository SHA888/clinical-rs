//! Tests for LOINC support.
//! Run with `cargo test --features loinc`
#![cfg(feature = "loinc")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use medcodes::types::{MedCodeError, System};
use medcodes::{CodeSystem, Loinc};

#[test]
fn test_loinc_lookup_success() {
    let loinc = Loinc::new();
    // Test with a common LOINC code that should exist in the dataset
    let code = loinc.lookup("2160-0").unwrap();
    assert_eq!(code.system, System::Loinc);
    assert_eq!(code.code, "2160-0");
    // Description may vary slightly depending on LOINC version, but should contain "Creatinine"
    assert!(code.description.contains("Creatinine"));
}

#[test]
fn test_loinc_lookup_not_found() {
    let loinc = Loinc::new();
    let err = loinc.lookup("9999-9").unwrap_err();
    match err {
        MedCodeError::NotFound { system, code } => {
            assert_eq!(system, System::Loinc);
            assert_eq!(code, "9999-9");
        }
        _ => panic!("Unexpected error type"),
    }
}

#[test]
fn test_loinc_components() {
    let loinc = Loinc::new();
    let comp = loinc.components("2160-0").unwrap();
    assert_eq!(comp.component, "Creatinine");
    assert_eq!(comp.property, "MCnc");
    assert_eq!(comp.timing, "Pt");
    assert_eq!(comp.system, "Ser/Plas");
    assert_eq!(comp.scale, "Qn");
    assert!(comp.method.is_none());
}

#[test]
fn test_loinc_is_valid() {
    let loinc = Loinc::new();
    assert!(loinc.is_valid("2160-0"));
    assert!(!loinc.is_valid("9999-9"));
}

#[test]
fn test_loinc_known_answers() {
    let loinc = Loinc::new();

    // Creatinine – Mass/volume in serum or plasma
    let comp = loinc.components("2160-0").unwrap();
    assert_eq!(comp.component, "Creatinine");
    assert_eq!(comp.property, "MCnc");
    assert_eq!(comp.system, "Ser/Plas");
    assert!(
        loinc
            .lookup("2160-0")
            .unwrap()
            .description
            .contains("Creatinine")
    );

    // Glucose – Mass/volume in serum or plasma
    let comp = loinc.components("2345-7").unwrap();
    assert_eq!(comp.component, "Glucose");
    assert_eq!(comp.property, "MCnc");
    assert_eq!(comp.system, "Ser/Plas");
    assert!(
        loinc
            .lookup("2345-7")
            .unwrap()
            .description
            .contains("Glucose")
    );

    // Troponin I – cardiac, Mass/volume in serum or plasma
    let comp = loinc.components("10839-9").unwrap();
    assert_eq!(comp.component, "Troponin I.cardiac");
    assert_eq!(comp.property, "MCnc");
    assert_eq!(comp.system, "Ser/Plas");
    assert!(
        loinc
            .lookup("10839-9")
            .unwrap()
            .description
            .contains("Troponin")
    );

    // Troponin T – cardiac, Mass/volume in serum or plasma
    let comp = loinc.components("6598-7").unwrap();
    assert_eq!(comp.component, "Troponin T.cardiac");
    assert_eq!(comp.property, "MCnc");
    assert_eq!(comp.system, "Ser/Plas");

    // Potassium – Moles/volume in serum or plasma
    let comp = loinc.components("2823-3").unwrap();
    assert_eq!(comp.component, "Potassium");
    assert_eq!(comp.property, "SCnc");
    assert_eq!(comp.system, "Ser/Plas");
}

#[test]
fn test_loinc_normalize() {
    let loinc = Loinc::new();

    // Leading/trailing whitespace is stripped
    assert!(loinc.is_valid("  2160-0  "));
    let code = loinc.lookup("  2160-0  ").unwrap();
    assert_eq!(code.code, "2160-0");

    let comp = loinc.components("\t2160-0\n").unwrap();
    assert_eq!(comp.component, "Creatinine");
}

#[test]
fn test_loinc_hierarchy() {
    let loinc = Loinc::new();
    // LOINC hierarchy via ComponentHierarchyBySystem is not yet loaded.
    // The hyphen in "2160-0" is a check digit, not a parent separator —
    // "2160" is not a valid LOINC code. Real hierarchy requires
    // AccessoryFiles/ComponentHierarchyBySystem/ComponentHierarchyBySystem.csv.
    assert!(loinc.parent("2160-0").unwrap().is_none());
    assert!(loinc.children("2160-0").unwrap().is_empty());
}

#[test]
fn test_loinc_ancestors_descendants() {
    let loinc = Loinc::new();
    // Hierarchy is empty until ComponentHierarchyBySystem is loaded.
    assert!(loinc.ancestors("2160-0").unwrap().is_empty());
    assert!(loinc.descendants("2160-0").unwrap().is_empty());
}
