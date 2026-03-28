//! Snapshot tests for hierarchy traversal using insta
//! Captures the exact output of hierarchy operations for known codes

use insta::{assert_debug_snapshot, assert_snapshot};
use medcodes::*;

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn test_icd10cm_hierarchy_traversal_snapshots() {
        let icd10 = Icd10Cm::new();

        // Test ancestors for common codes
        let test_codes = ["I109", "I10", "E119", "J45909", "M545"];

        for &code in &test_codes {
            if icd10.is_valid(code) {
                let ancestors = icd10.ancestors(code).unwrap_or_default();
                assert_debug_snapshot!(format!("ancestors_{}", code), &ancestors);

                let descendants = icd10.descendants(code).unwrap_or_default();
                assert_debug_snapshot!(format!("descendants_{}", code), &descendants);

                let parent = icd10.parent(code).unwrap_or_default();
                assert_debug_snapshot!(format!("parent_{}", code), &parent);

                let children = icd10.children(code).unwrap_or_default();
                assert_debug_snapshot!(format!("children_{}", code), &children);
            }
        }
    }

    #[test]
    fn test_ccsr_mapping_snapshots() {
        let forward_mapper = Icd10CmToCcsr::new();
        let reverse_mapper = CcsrToIcd10Cm::new();

        // Test CCSR mappings for common codes
        let test_codes = ["I109", "I10", "E119"];

        for &code in &test_codes {
            // Forward mapping
            if let Ok(categories) = forward_mapper.get_categories(code) {
                assert_debug_snapshot!(format!("ccsr_categories_{}", code), &categories);
            }

            // Context-sensitive mapping
            let contexts = [
                CcsrContext::Inpatient,
                CcsrContext::EmergencyDepartment,
                CcsrContext::Outpatient,
            ];

            for &context in &contexts {
                if let Ok(default_category) = forward_mapper.get_default_category(code, context) {
                    assert_debug_snapshot!(
                        format!("ccsr_default_{}_{:?}", code, context),
                        &default_category
                    );
                }
            }
        }

        // Test reverse mapping for common CCSR categories
        let test_ccsr_codes = ["CIRC001", "ENDO001", "RESP001"];

        for &ccsr_code in &test_ccsr_codes {
            if let Ok(icd10_codes) = reverse_mapper.get_icd10_codes(ccsr_code) {
                assert_debug_snapshot!(format!("icd10_from_ccsr_{}", ccsr_code), &icd10_codes);
            }
        }
    }

    #[test]
    fn test_code_lookup_snapshots() {
        let icd10 = Icd10Cm::new();

        // Test detailed lookup information for common codes
        let test_codes = ["I109", "I10", "E119", "J45909", "M545"];

        for &code in &test_codes {
            if let Ok(lookup_result) = icd10.lookup(code) {
                assert_debug_snapshot!(format!("lookup_{}", code), &lookup_result);
            }
        }
    }

    #[test]
    fn test_crossmap_trait_snapshots() {
        let forward_mapper = Icd10CmToCcsr::new();
        let reverse_mapper = CcsrToIcd10Cm::new();

        // Test CrossMap trait implementations
        let test_codes = ["I109", "I10"];

        for &code in &test_codes {
            // Forward mapping through CrossMap trait
            if let Ok(mapped_codes) = forward_mapper.map(code, System::Ccsr) {
                assert_debug_snapshot!(format!("crossmap_forward_{}", code), &mapped_codes);
            }

            // Reverse mapping through CrossMap trait
            if let Ok(categories) = forward_mapper.get_categories(code) {
                for category in categories {
                    if let Ok(mapped_codes) = reverse_mapper.map(&category.code, System::Icd10Cm) {
                        assert_debug_snapshot!(
                            format!("crossmap_reverse_{}_{}", code, category.code),
                            &mapped_codes
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_error_case_snapshots() {
        let icd10 = Icd10Cm::new();
        let forward_mapper = Icd10CmToCcsr::new();
        let reverse_mapper = CcsrToIcd10Cm::new();

        let invalid_codes = ["INVALID", "XYZ", "", "A"];

        for &code in &invalid_codes {
            // ICD-10-CM errors
            let lookup_error = icd10.lookup(code).unwrap_err();
            assert_debug_snapshot!(format!("lookup_error_{}", code), &lookup_error);

            let ancestors_error = icd10.ancestors(code).unwrap_err();
            assert_debug_snapshot!(format!("ancestors_error_{}", code), &ancestors_error);

            // Cross-mapping errors
            let forward_error = forward_mapper.get_categories(code).unwrap_err();
            assert_debug_snapshot!(format!("forward_error_{}", code), &forward_error);

            let reverse_error = reverse_mapper.get_icd10_codes(code).unwrap_err();
            assert_debug_snapshot!(format!("reverse_error_{}", code), &reverse_error);
        }
    }

    #[test]
    fn test_system_display_snapshots() {
        // Test system display formats
        let systems = [System::Icd10Cm, System::Ccsr];

        for &system in &systems {
            let display_str = format!("{}", system);
            assert_snapshot!(format!("system_display_{:?}", system), &display_str);
        }
    }

    #[test]
    fn test_medcode_error_snapshots() {
        // Test different error types
        let not_found = MedCodeError::not_found("I109", System::Icd10Cm);
        assert_debug_snapshot!("error_not_found", &not_found);

        let invalid_format = MedCodeError::invalid_format("I.10.9", System::Icd10Cm);
        assert_debug_snapshot!("error_invalid_format", &invalid_format);

        let no_mapping = MedCodeError::no_mapping("I109", System::Icd10Cm, System::Ccsr);
        assert_debug_snapshot!("error_no_mapping", &no_mapping);

        let hierarchy = MedCodeError::hierarchy("Cycle detected in hierarchy");
        assert_debug_snapshot!("error_hierarchy", &hierarchy);

        let data = MedCodeError::data("Failed to parse data");
        assert_debug_snapshot!("error_data", &data);
    }
}
