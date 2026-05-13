//! Tests for LOINC support.
//! Run with `cargo test --features loinc`
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use medcodes::types::{MedCodeError, System};
use medcodes::{CodeSystem, Loinc};

#[test]
fn test_loinc_lookup_success() {
    let loinc = Loinc::new();
    let code = loinc.lookup("2160-0").unwrap();
    assert_eq!(code.system, System::Loinc);
    assert_eq!(code.code, "2160-0");
    assert_eq!(
        code.description,
        "Creatinine [Mass/volume] in Serum or Plasma"
    );
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
    assert_eq!(comp.property, "Mass");
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
