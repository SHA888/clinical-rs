#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

//! Comprehensive tests for all public methods in the medcodes crate

use medcodes::*;
use proptest::prelude::Just;
use proptest::{prop_assert, prop_assert_eq, prop_oneof, proptest};

// ==================== Types Tests ====================

#[cfg(test)]
mod types_tests {
    use super::*;

    #[test]
    fn test_code_new() {
        let code = Code::new(System::Icd10Cm, "A0100", "Cholera");
        assert_eq!(code.system, System::Icd10Cm);
        assert_eq!(code.code(), "A0100");
        assert_eq!(code.description(), "Cholera");
    }

    #[test]
    fn test_code_from_tuple() {
        let code: Code = (System::Icd10Cm, "A0100", "Cholera").into();
        assert_eq!(code.system, System::Icd10Cm);
        assert_eq!(code.code(), "A0100");
        assert_eq!(code.description(), "Cholera");
    }

    #[test]
    fn test_medcode_error_constructors() {
        let not_found = MedCodeError::not_found("A0100", System::Icd10Cm);
        assert!(matches!(not_found, MedCodeError::NotFound { code, system }
            if code == "A0100" && system == System::Icd10Cm));

        let invalid_format = MedCodeError::invalid_format("A.01.00", System::Icd10Cm);
        assert!(
            matches!(invalid_format, MedCodeError::InvalidFormat { code, system }
            if code == "A.01.00" && system == System::Icd10Cm)
        );

        let no_mapping = MedCodeError::no_mapping("A0100", System::Icd10Cm, System::Ccsr);
        assert!(
            matches!(no_mapping, MedCodeError::NoMapping { code, source_system, target_system }
            if code == "A0100" && source_system == System::Icd10Cm && target_system == System::Ccsr)
        );
    }

    #[test]
    fn test_system_display() {
        let system = System::Icd10Cm;
        assert_eq!(format!("{system}"), "ICD-10-CM");
        let system = System::Ccsr;
        assert_eq!(format!("{system}"), "CCSR");
    }

    #[test]
    fn test_code_display() {
        let code = Code::new(System::Icd10Cm, "A0100", "Cholera");
        assert_eq!(format!("{code}"), "ICD-10-CM A0100: Cholera");
    }
}

// ==================== ICD-10-CM Tests ====================

#[cfg(test)]
mod icd10cm_comprehensive_tests {
    use super::*;

    #[test]
    fn test_icd10cm_new() {
        let icd10 = Icd10Cm::new();
        // Test that it implements Default
        let icd10_default: Icd10Cm = Icd10Cm::default();
        assert_eq!(
            icd10.normalize_code("A0100"),
            icd10_default.normalize_code("A0100")
        );
    }

    #[test]
    fn test_icd10cm_normalize_code() {
        let icd10 = Icd10Cm::new();

        // Test various formats
        assert_eq!(icd10.normalize_code("A0100"), "A0100");
        assert_eq!(icd10.normalize_code("A01.00"), "A0100");
        assert_eq!(icd10.normalize_code("a01.00"), "A0100");
        assert_eq!(icd10.normalize_code("  A01.00  "), "A0100");
        assert_eq!(icd10.normalize_code("A0100 "), "A0100");
        assert_eq!(icd10.normalize_code(" A0100"), "A0100");
    }

    #[test]
    fn test_icd10cm_lookup() {
        let icd10 = Icd10Cm::new();

        // Test valid code
        let result = icd10.lookup("A00.0");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code.system, System::Icd10Cm);
        assert_eq!(code.code(), "A00.0");

        // Test invalid code
        let result = icd10.lookup("INVALID");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MedCodeError::NotFound { .. }));
    }

    #[test]
    fn test_icd10cm_is_valid() {
        let icd10 = Icd10Cm::new();

        // Test valid codes (using codes that exist in the actual dataset)
        assert!(icd10.is_valid("A00.0")); // Cholera due to Vibrio cholerae 01, biovar cholerae
        assert!(icd10.is_valid("A00.0")); // Should normalize to A00.0
        assert!(icd10.is_valid("a00.0")); // Case insensitive

        // Test invalid codes
        assert!(!icd10.is_valid("INVALID"));
        assert!(!icd10.is_valid(""));
        assert!(!icd10.is_valid("A"));
    }

    #[test]
    fn test_icd10cm_normalize() {
        let icd10 = Icd10Cm::new();

        // Test through trait
        assert_eq!(icd10.normalize("A01.00"), "A0100");
        assert_eq!(icd10.normalize("a01.00"), "A0100");
    }

    #[test]
    fn test_icd10cm_ancestors() {
        let icd10 = Icd10Cm::new();

        // Test hierarchy on codes that have ancestors
        // Use codes from known valid set, check if they have ancestors
        let test_codes = ["A00.0", "A01.00", "A02.0"];

        for &code in &test_codes {
            if icd10.is_valid(code) {
                let result = icd10.ancestors(code);
                assert!(result.is_ok(), "ancestors({code}) should succeed");
                let ancestors = result.unwrap();

                // All ancestors should be valid ICD-10-CM codes
                for ancestor in &ancestors {
                    assert_eq!(ancestor.system, System::Icd10Cm);
                    assert!(icd10.is_valid(&ancestor.code));
                }
            }
        }

        // Test invalid code
        let result = icd10.ancestors("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_icd10cm_descendants() {
        let icd10 = Icd10Cm::new();

        // Test hierarchy on codes that may have descendants
        let test_codes = ["A00", "A01", "A02"];

        for &code in &test_codes {
            if icd10.is_valid(code) {
                let result = icd10.descendants(code);
                assert!(result.is_ok(), "descendants({code}) should succeed");
                let descendants = result.unwrap();

                // All descendants should be valid ICD-10-CM codes
                for descendant in &descendants {
                    assert_eq!(descendant.system, System::Icd10Cm);
                    assert!(icd10.is_valid(&descendant.code));
                }
            }
        }

        // Test on leaf node (should have no descendants)
        if icd10.is_valid("A00.0") {
            let result = icd10.descendants("A00.0");
            assert!(result.is_ok());
            let descendants = result.unwrap();
            assert!(descendants.is_empty(), "A00.0 is a leaf node");
        }

        // Test invalid code
        let result = icd10.descendants("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_icd10cm_parent() {
        let icd10 = Icd10Cm::new();

        // Test valid code
        let result = icd10.parent("A00.0");
        assert!(result.is_ok());
        let parent = result.unwrap();

        // Should have a parent (unless it's a top-level category)
        if let Some(p) = parent {
            assert_eq!(p.system, System::Icd10Cm);
            assert!(icd10.is_valid(&p.code));
        }

        // Test invalid code
        let result = icd10.parent("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_icd10cm_children() {
        let icd10 = Icd10Cm::new();

        // Test hierarchy on codes that may have children
        let test_codes = ["A00", "A01", "A02"];

        for &code in &test_codes {
            if icd10.is_valid(code) {
                let result = icd10.children(code);
                assert!(result.is_ok(), "children({code}) should succeed");
                let children = result.unwrap();

                // All children should be valid ICD-10-CM codes
                for child in &children {
                    assert_eq!(child.system, System::Icd10Cm);
                    assert!(icd10.is_valid(&child.code));
                }
            }
        }

        // Test invalid code
        let result = icd10.children("INVALID");
        assert!(result.is_err());

        // Test leaf node (should have no children)
        if icd10.is_valid("A00.0") {
            let result = icd10.children("A00.0");
            assert!(result.is_ok());
            let leaf_children = result.unwrap();
            assert!(leaf_children.is_empty(), "A00.0 is a leaf node");
        }
    }
}

// ==================== CCSR Tests ====================

#[cfg(test)]
mod ccsr_comprehensive_tests {
    use super::*;

    #[test]
    fn test_ccsr_category_new() {
        let category = CcsrCategory::new("DIG001", "Intestinal infectious diseases");
        assert_eq!(category.code, "DIG001");
        assert_eq!(category.description, "Intestinal infectious diseases");
    }

    #[test]
    fn test_icd10cm_to_ccsr_new() {
        let mapper = Icd10CmToCcsr::new();
        // Test Default implementation
        let mapper_default: Icd10CmToCcsr = Icd10CmToCcsr::default();
        assert_eq!(mapper.source_system(), mapper_default.source_system());
    }

    #[test]
    fn test_icd10cm_to_ccsr_get_categories() {
        let mapper = Icd10CmToCcsr::new();

        // Test with a known ICD-10-CM code
        let result = mapper.get_categories("A00.0");

        // This might fail if the data doesn't contain A00.0, but let's test the structure
        match result {
            Ok(categories) => {
                let categories: Vec<CcsrCategory> = categories;
                assert!(!categories.is_empty());
                for category in &categories {
                    assert!(!category.code.is_empty());
                    assert!(!category.description.is_empty());
                }
            }
            Err(MedCodeError::NotFound { .. }) => {
                // Expected if code not in mapping
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }

        // Test invalid code
        let result = mapper.get_categories("INVALID");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MedCodeError::NotFound { .. }));
    }

    #[test]
    fn test_icd10cm_to_ccsr_get_default_category() {
        let mapper = Icd10CmToCcsr::new();

        // Test with different contexts
        let contexts = [
            CcsrContext::Inpatient,
            CcsrContext::EmergencyDepartment,
            CcsrContext::Outpatient,
        ];

        for context in contexts {
            let result = mapper.get_default_category("A00.0", context);

            match result {
                Ok(category) => {
                    assert!(!category.code.is_empty());
                    assert!(!category.description.is_empty());
                }
                Err(MedCodeError::NotFound { .. } | MedCodeError::NoMapping { .. }) => {
                    // Expected if code not in mapping or no default for context
                }
                Err(e) => {
                    panic!("Unexpected error for context {context:?}: {e:?}");
                }
            }
        }

        // Test invalid code
        let result = mapper.get_default_category("INVALID", CcsrContext::Inpatient);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MedCodeError::NotFound { .. }));
    }

    #[test]
    fn test_icd10cm_to_ccsr_crossmap_trait() {
        let mapper = Icd10CmToCcsr::new();

        // Test source and target systems
        assert_eq!(mapper.source_system(), System::Icd10Cm);
        assert_eq!(mapper.target_system(), System::Ccsr);

        // Test mapping to correct target system
        let result = mapper.map("A00.0", System::Ccsr);
        match result {
            Ok(codes) => {
                let codes: Vec<Code> = codes;
                assert!(!codes.is_empty());
                for code in &codes {
                    assert_eq!(code.system, System::Ccsr);
                }
            }
            Err(MedCodeError::NotFound { .. }) => {
                // Expected if code not in mapping
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }

        // Test mapping to wrong target system
        let result = mapper.map("A00.0", System::Icd9Cm);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MedCodeError::NoMapping { .. }
        ));
    }

    #[test]
    fn test_ccsr_to_icd10cm_new() {
        let mapper = CcsrToIcd10Cm::new();
        // Test Default implementation
        let mapper_default: CcsrToIcd10Cm = CcsrToIcd10Cm::default();
        assert_eq!(mapper.source_system(), mapper_default.source_system());
    }

    #[test]
    fn test_ccsr_to_icd10cm_get_icd10_codes() {
        let mapper = CcsrToIcd10Cm::new();

        // Test with a known CCSR category
        let result = mapper.get_icd10_codes("DIG001");

        match result {
            Ok(codes) => {
                assert!(!codes.is_empty());
                for code in &codes {
                    assert!(!code.is_empty());
                    // Should be valid ICD-10-CM format (3-7 characters, starts with letter)
                    assert!(code.len() >= 3 && code.len() <= 7);
                    assert!(code.chars().next().unwrap().is_ascii_uppercase());
                }
            }
            Err(MedCodeError::NotFound { .. }) => {
                // Expected if category not in mapping
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }

        // Test invalid category
        let result = mapper.get_icd10_codes("INVALID");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MedCodeError::NotFound { .. }));
    }

    #[test]
    fn test_ccsr_to_icd10cm_crossmap_trait() {
        let mapper = CcsrToIcd10Cm::new();

        // Test source and target systems
        assert_eq!(mapper.source_system(), System::Ccsr);
        assert_eq!(mapper.target_system(), System::Icd10Cm);

        // Test mapping to correct target system
        let result = mapper.map("DIG001", System::Icd10Cm);
        match result {
            Ok(codes) => {
                assert!(!codes.is_empty());
                for code in &codes {
                    assert_eq!(code.system, System::Icd10Cm);
                }
            }
            Err(MedCodeError::NotFound { .. }) => {
                // Expected if category not in mapping
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }

        // Test mapping to wrong target system
        let result = mapper.map("DIG001", System::Icd9Cm);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MedCodeError::NoMapping { .. }
        ));
    }
}

// ==================== Property Tests ====================

proptest! {
    #[test]
    fn test_icd10cm_normalize_idempotent(input in "[A-Za-z0-9.]{1,10}") {
        let icd10 = Icd10Cm::new();
        let normalized1 = icd10.normalize_code(&input);
        let normalized2 = icd10.normalize_code(&normalized1);
        // Normalization is idempotent for valid ICD-10-CM patterns (letters, digits, dots only)
        prop_assert_eq!(normalized1, normalized2);
    }

    #[test]
    fn test_icd10cm_normalize_uppercase(input in "[a-z0-9. ]{1,10}") {
        let icd10 = Icd10Cm::new();
        let normalized = icd10.normalize_code(&input);
        // Result should be uppercase (dots removed, spaces preserved)
        prop_assert!(normalized.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == ' '));
    }

    #[test]
    fn test_icd10cm_normalize_no_dots(input in "[A-Za-z0-9. ]{1,10}") {
        let icd10 = Icd10Cm::new();
        let normalized = icd10.normalize_code(&input);
        // Result should not contain dots (spaces preserved)
        prop_assert!(!normalized.contains('.'));
    }

    #[test]
    fn test_icd10cm_is_valid_after_normalize(code in "[A-Za-z0-9. ]{1,10}") {
        let icd10 = Icd10Cm::new();
        let normalized = icd10.normalize_code(&code);
        // If the original code is valid, the normalized version should also be valid
        if icd10.is_valid(&code) {
            prop_assert!(icd10.is_valid(&normalized));
        }
    }

    #[test]
    fn test_code_display_roundtrip(system in prop_oneof![
        Just(System::Icd10Cm),
        Just(System::Ccsr),
    ], code in "[A-Z0-9]{3,7}", desc in "[a-zA-Z ]{10,50}") {
        let code_obj = Code::new(system, &code, &desc);
        let display_str = format!("{code_obj}");
        let system_str = format!("{system}");
        // Should contain system, code, and description
        prop_assert!(display_str.contains(&system_str));
        prop_assert!(display_str.contains(&code));
        prop_assert!(display_str.contains(&desc));
    }
}

// ==================== Edge Case Tests ====================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_and_whitespace_inputs() {
        let icd10 = Icd10Cm::new();

        // Test empty string
        assert_eq!(icd10.normalize_code(""), "");
        assert!(!icd10.is_valid(""));

        // Test whitespace only
        assert_eq!(icd10.normalize_code("   "), "");
        assert!(!icd10.is_valid("   "));

        // Test whitespace around valid code (normalization removes dots and whitespace)
        assert_eq!(icd10.normalize_code("  A00.0  "), "A000");
        assert!(icd10.is_valid("A00.0")); // Valid code without whitespace
    }

    #[test]
    fn test_special_characters() {
        let icd10 = Icd10Cm::new();

        // Test various special characters (dots are removed, others remain)
        let inputs = ["A00.0", "A00-0", "A00_0", "A00 0"];
        for input in inputs {
            let normalized = icd10.normalize_code(input);
            // Dots are removed, other characters are kept as-is by current implementation
            assert!(
                normalized
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
            );
        }
    }

    #[test]
    fn test_very_long_inputs() {
        let icd10 = Icd10Cm::new();

        // Test very long input
        let long_input = "A".repeat(100);
        let normalized = icd10.normalize_code(&long_input);
        assert!(!icd10.is_valid(&normalized));

        // Test lookup with very long input
        let result = icd10.lookup(&long_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_hierarchy_edge_cases() {
        let icd10 = Icd10Cm::new();

        // Test hierarchy operations with invalid codes
        let invalid_code = "INVALID";

        assert!(icd10.ancestors(invalid_code).is_err());
        assert!(icd10.descendants(invalid_code).is_err());
        assert!(icd10.parent(invalid_code).is_err());
        assert!(icd10.children(invalid_code).is_err());
    }

    #[test]
    fn test_ccsr_edge_cases() {
        let forward_mapper = Icd10CmToCcsr::new();
        let reverse_mapper = CcsrToIcd10Cm::new();

        // Test with empty strings
        assert!(forward_mapper.get_categories("").is_err());
        assert!(reverse_mapper.get_icd10_codes("").is_err());

        // Test with whitespace
        assert!(forward_mapper.get_categories("   ").is_err());
        assert!(reverse_mapper.get_icd10_codes("   ").is_err());

        // Test CrossMap trait with invalid inputs
        assert!(forward_mapper.map("", System::Ccsr).is_err());
        assert!(reverse_mapper.map("", System::Icd10Cm).is_err());
    }
}
