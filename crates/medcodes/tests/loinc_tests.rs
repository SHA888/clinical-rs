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

    // The immediate parent of a LOINC term is always a LOINC Part number
    // (from the ComponentHierarchyBySystem multiaxial hierarchy), not another
    // term: 2160-0 (Creatinine SerPl-mCnc) groups under Part LP385359-7.
    let parent = loinc.parent("2160-0").unwrap().unwrap();
    assert_eq!(parent.code, "LP385359-7");
    assert!(parent.description.contains("Creatinine"));

    // The Part's children include 2160-0 alongside sibling creatinine panel
    // members that share the same Component/System/Class grouping.
    let children = loinc.children("LP385359-7").unwrap();
    assert!(children.iter().any(|c| c.code == "2160-0"));

    // Terms are leaves: a term never has children of its own.
    assert!(loinc.children("2160-0").unwrap().is_empty());
}

#[test]
fn test_loinc_ancestors_descendants() {
    let loinc = Loinc::new();

    // Walking ancestors from a term climbs through Part-level Component,
    // System, and Class groupings up to the hierarchy root.
    let ancestors = loinc.ancestors("2160-0").unwrap();
    assert!(!ancestors.is_empty());
    assert_eq!(ancestors.first().unwrap().code, "LP385359-7");
    assert_eq!(ancestors.last().unwrap().code, "LP432695-7");

    // Terms are leaves: a term never has descendants of its own.
    assert!(loinc.descendants("2160-0").unwrap().is_empty());

    // Descendants of the Part are the sibling creatinine panel members.
    let descendants = loinc.descendants("LP385359-7").unwrap();
    assert!(descendants.iter().any(|c| c.code == "2160-0"));
}

#[test]
fn test_loinc_hierarchy_not_found() {
    let loinc = Loinc::new();
    for method_result in [
        loinc.parent("9999-9").map(|_| ()),
        loinc.children("9999-9").map(|_| ()),
        loinc.ancestors("9999-9").map(|_| ()),
        loinc.descendants("9999-9").map(|_| ()),
    ] {
        match method_result.unwrap_err() {
            MedCodeError::NotFound { system, code } => {
                assert_eq!(system, System::Loinc);
                assert_eq!(code, "9999-9");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
