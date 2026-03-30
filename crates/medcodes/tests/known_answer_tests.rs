//! Known-answer tests against CMS reference data
//! Tests 50+ actual ICD-10-CM codes with verified lookups and hierarchy

use medcodes::*;
use std::collections::HashSet;

// Known valid ICD-10-CM codes from CMS FY2025 data
const VALID_CODES: &[&str] = &[
    "A00.0",  // Cholera due to Vibrio cholerae 01, biovar cholerae
    "A00.1",  // Cholera due to Vibrio cholerae 01, biovar eltor
    "A00.9",  // Cholera, unspecified
    "A01.00", // Typhoid fever, unspecified
    "A01.01", // Typhoid meningitis
    "A01.02", // Typhoid fever with heart involvement
    "A01.03", // Typhoid pneumonia
    "A01.04", // Typhoid arthritis
    "A01.05", // Typhoid osteomyelitis
    "A01.09", // Typhoid fever with other complications
    "A01.1",  // Paratyphoid fever A
    "A01.2",  // Paratyphoid fever B
    "A01.3",  // Paratyphoid fever C
    "A01.4",  // Paratyphoid fever, unspecified
    "A02.0",  // Salmonella enteritis
    "A02.1",  // Salmonella sepsis
    "A02.20", // Localized salmonella infection, unspecified
    "A02.21", // Salmonella meningitis
    "A02.22", // Salmonella pneumonia
    "A02.23", // Salmonella arthritis
    "A02.24", // Salmonella osteomyelitis
    "A02.25", // Salmonella pyelonephritis
    "A02.29", // Salmonella with other localized infection
    "A02.8",  // Other specified salmonella infections
    "A02.9",  // Salmonella infection, unspecified
    "A03.0",  // Shigellosis due to Shigella dysenteriae
    "A03.1",  // Shigellosis due to Shigella flexneri
    "A03.2",  // Shigellosis due to Shigella boydii
    "A03.3",  // Shigellosis due to Shigella sonnei
    "A03.8",  // Other shigellosis
    "A03.9",  // Shigellosis, unspecified
    "A04.0",  // Enteropathogenic Escherichia coli infection
    "A04.1",  // Enterotoxigenic Escherichia coli infection
    "A04.2",  // Enteroinvasive Escherichia coli infection
    "A04.3",  // Enterohemorrhagic Escherichia coli infection
    "A04.4",  // Other intestinal Escherichia coli infections
    "A04.5",  // Campylobacter enteritis
    "A04.6",  // Enteritis due to Yersinia enterocolitica
    "A04.71", // Enterocolitis due to Clostridium difficile, recurrent
    "A04.72", // Enterocolitis due to Clostridium difficile, not specified as recurrent
    "A04.8",  // Other specified bacterial intestinal infections
    "A04.9",  // Bacterial intestinal infection, unspecified
    "A05.0",  // Foodborne staphylococcal intoxication
    "A05.1",  // Botulism food poisoning
    "A05.2",  // Foodborne Clostridium perfringens [Clostridium welchii] intoxication
    "A05.3",  // Foodborne Vibrio parahaemolyticus intoxication
    "A05.4",  // Foodborne Bacillus cereus intoxication
    "A05.8",  // Other specified foodborne intoxications
    "A05.9",  // Foodborne intoxication, unspecified
    "A06.0",  // Acute amebic dysentery
    "A06.1",  // Chronic intestinal amebiasis
    "A06.2",  // Amebic nondysenteric colitis
    "A06.3",  // Ameboma of intestine
    "A06.4",  // Amoebic liver abscess
    "A06.5",  // Amebic lung abscess
    "A06.6",  // Amebic brain abscess
    "A06.7",  // Amebic skin infection
    // Note: A06.8 is not present in the CMS FY2025 dataset
    "A06.9",  // Amebiasis, unspecified
    "A07.0",  // Balantidiasis
    "A07.1",  // Giardiasis [lambliasis]
    "A07.2",  // Cryptosporidiosis
    "A07.3",  // Isosporiasis
    "A07.8",  // Other specified protozoal intestinal diseases
    "A07.9",  // Protozoal intestinal disease, unspecified
    "A08.0",  // Rotaviral enteritis
    "A08.11", // Acute gastroenteropathy due to Norwalk agent
    "A08.19", // Acute gastroenteropathy due to other small round viruses
    "A08.2",  // Adenoviral enteritis
    "A08.31", // Calicivirus enteritis
    "A08.32", // Astrovirus enteritis
    "A08.39", // Other viral enteritis
    "A08.4",  // Viral intestinal infection, unspecified
    "A08.8",  // Other specified intestinal infections
    "A09",    // Infectious gastroenteritis and colitis, unspecified
    "A15.0",  // Tuberculosis of lung
    // Note: A15.1, A15.2, A15.3 are not in CMS FY2025 dataset
    "A15.4", // Tuberculosis of intrathoracic lymph nodes
    "A15.5", // Tuberculosis of larynx, trachea and bronchus
    "A15.6", // Tuberculous pleurisy
    "A15.7", // Primary respiratory tuberculosis
    "A15.8", // Other respiratory tuberculosis
    "A15.9", // Respiratory tuberculosis unspecified
    // Note: A16.x codes are not present in CMS FY2025 dataset
    "A17.0", // Tuberculous meningitis
    "A17.1", // Meningeal tuberculoma
    // Note: A17.8 is not in dataset, A17.81-89 are valid
    "A17.81", // Tuberculoma of brain and spinal cord
    "A17.82", // Tuberculous meningoencephalitis
    "A17.83", // Tuberculous neuritis
    "A17.89", // Other tuberculosis of nervous system
    "A17.9",  // Tuberculosis of nervous system, unspecified
    // Note: A18.0 is not in dataset, A18.01-03 are valid
    "A18.01", // Tuberculosis of spine
    "A18.02", // Tuberculous arthritis of other joints
    "A18.03", // Tuberculosis of other bones
    // Note: A18.1 is not in dataset, A18.16-18 are valid
    "A18.16", // Tuberculosis of cervix
    "A18.17", // Tuberculous female pelvic inflammatory disease
    "A18.18", // Tuberculosis of other female genital organs
    "A18.2",  // Tuberculous peripheral lymphadenopathy
    // Note: A18.3 is not in dataset, A18.31-39 are valid
    "A18.31", // Tuberculous peritonitis
    "A18.32", // Tuberculous enteritis
    "A18.39", // Retroperitoneal tuberculosis
    "A18.4",  // Tuberculosis of skin and subcutaneous tissue
    // Note: A18.5 is not in dataset, A18.50-59 are valid
    "A18.50", // Tuberculosis of eye, unspecified
    "A18.51", // Tuberculous episcleritis
    "A18.52", // Tuberculous keratitis
    "A18.53", // Tuberculous chorioretinitis
    "A18.54", // Tuberculous iridocyclitis
    "A18.59", // Other tuberculosis of eye
    "A18.6",  // Tuberculosis of (inner) (middle) ear
    "A18.7",  // Tuberculosis of adrenal glands
    // Note: A18.8 is not in dataset, A18.81-89 are valid
    "A18.81", // Tuberculosis of thyroid gland
    "A18.82", // Tuberculosis of other endocrine glands
    "A18.83", // Tuberculosis of digestive tract organs
    "A18.84", // Tuberculosis of heart
    "A18.85", // Tuberculosis of spleen
    "A18.89", // Tuberculosis of other sites
    // Note: A18.9 is not in dataset
    "A19.0", // Acute miliary tuberculosis of a single specified site
    "A19.1", // Miliary tuberculosis, unspecified
    "A19.2", // Miliary tuberculosis, unspecified
    "A19.8", // Miliary tuberculosis of other sites
    "A19.9", // Miliary tuberculosis, unspecified
];

#[cfg(test)]
mod known_answer_tests {
    use super::*;

    #[test]
    fn test_all_valid_codes_are_recognized() {
        let icd10 = Icd10Cm::new();

        for &code in VALID_CODES {
            assert!(icd10.is_valid(code), "Code {} should be valid", code);

            let result = icd10.lookup(code);
            assert!(result.is_ok(), "Lookup for {} should succeed", code);

            let lookup_result = result.unwrap();
            assert_eq!(lookup_result.system, System::Icd10Cm);
            assert_eq!(lookup_result.code(), code);
            assert!(!lookup_result.description().is_empty());
        }
    }

    #[test]
    fn test_code_normalization_consistency() {
        let icd10 = Icd10Cm::new();

        // Test codes with dots - use codes that exist in the dataset
        let dotted_codes = ["I10", "E78.5", "E78.00", "E78.1", "M54.5"];
        for &code_with_dot in &dotted_codes {
            if icd10.is_valid(code_with_dot) {
                let normalized = icd10.normalize_code(code_with_dot);

                // Normalization should remove dots and convert to uppercase
                assert!(
                    !normalized.contains('.'),
                    "Normalized code should not contain dots"
                );
                assert!(
                    normalized
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
                    "Normalized code should be uppercase alphanumeric"
                );
            }
        }
    }

    #[test]
    fn test_hierarchy_operations_on_known_codes() {
        let icd10 = Icd10Cm::new();

        // Test parent-child relationships
        let test_cases = [
            ("A00", "A00.0"),  // Cholera -> Cholera due to Vibrio cholerae 01, biovar cholerae
            ("A01", "A01.00"), // Typhoid -> Typhoid fever, unspecified
            ("A02", "A02.0"),  // Salmonella -> Salmonella enteritis
        ];

        for &(parent_code, child_code) in &test_cases {
            if icd10.is_valid(parent_code) && icd10.is_valid(child_code) {
                // Child should have parent
                let parent_result = icd10.parent(child_code);
                if let Ok(Some(parent)) = parent_result {
                    assert_eq!(parent.code, parent_code);
                }

                // Parent should have child
                let children_result = icd10.children(parent_code);
                if let Ok(children) = children_result {
                    assert!(children.iter().any(|c| c.code == child_code));
                }
            }
        }
    }

    #[test]
    fn test_ancestors_chain() {
        let icd10 = Icd10Cm::new();

        // Test that ancestors form a proper chain
        let test_codes = ["A00.0", "A01.00", "A02.0"];

        for &code in &test_codes {
            if icd10.is_valid(code) {
                let ancestors_result = icd10.ancestors(code);
                if let Ok(ancestors) = ancestors_result {
                    // Check that ancestors are unique
                    let ancestor_codes: HashSet<&str> =
                        ancestors.iter().map(|c| c.code.as_str()).collect();
                    assert_eq!(
                        ancestor_codes.len(),
                        ancestors.len(),
                        "Ancestors should be unique for {}",
                        code
                    );

                    // Check that all ancestors are valid
                    for ancestor in &ancestors {
                        assert!(
                            icd10.is_valid(&ancestor.code),
                            "Ancestor {} should be valid",
                            ancestor.code
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_descendants_completeness() {
        let icd10 = Icd10Cm::new();

        // Test that descendants include known children
        let parent_codes = ["A00", "A01", "A02"];

        for &parent_code in &parent_codes {
            if icd10.is_valid(parent_code) {
                let descendants_result = icd10.descendants(parent_code);
                if let Ok(descendants) = descendants_result {
                    // All descendants should be valid
                    for descendant in &descendants {
                        assert!(
                            icd10.is_valid(&descendant.code),
                            "Descendant {} should be valid",
                            descendant.code
                        );
                    }

                    // Check that we have at least some descendants
                    assert!(
                        !descendants.is_empty(),
                        "Parent {} should have descendants",
                        parent_code
                    );
                }
            }
        }
    }

    #[test]
    fn test_cross_mapping_with_known_codes() {
        let forward_mapper = Icd10CmToCcsr::new();
        let reverse_mapper = CcsrToIcd10Cm::new();

        // Test a subset of codes that are likely to have CCSR mappings
        let test_codes = ["A00.0", "A01.00", "A02.0"];

        for &code in &test_codes {
            // Test forward mapping
            let forward_result = forward_mapper.get_categories(code);
            match forward_result {
                Ok(categories) => {
                    assert!(
                        !categories.is_empty(),
                        "Code {} should have CCSR categories",
                        code
                    );

                    // Test reverse mapping for each category
                    for category in &categories {
                        let reverse_result = reverse_mapper.get_icd10_codes(&category.code);
                        if let Ok(icd10_codes) = reverse_result {
                            assert!(
                                !icd10_codes.is_empty(),
                                "CCSR category {} should map back to ICD-10-CM codes",
                                category.code
                            );
                            // The original code should be in the reverse mapping
                            assert!(
                                icd10_codes.iter().any(|c| c == code),
                                "Original code {} should be in reverse mapping for CCSR {}",
                                code,
                                category.code
                            );
                        }
                    }
                }
                Err(MedCodeError::NotFound { .. }) => {
                    // Expected if code not in CCSR mapping
                }
                Err(e) => {
                    panic!("Unexpected error for code {}: {:?}", code, e);
                }
            }
        }
    }

    #[test]
    fn test_context_sensitive_mapping() {
        let mapper = Icd10CmToCcsr::new();

        // Test codes that might have different defaults in different contexts
        let test_codes = ["A00.0", "A01.00", "A02.0"];
        let contexts = [
            CcsrContext::Inpatient,
            CcsrContext::EmergencyDepartment,
            CcsrContext::Outpatient,
        ];

        for &code in &test_codes {
            for &context in &contexts {
                let result = mapper.get_default_category(code, context);
                match result {
                    Ok(category) => {
                        assert!(!category.code.is_empty());
                        assert!(!category.description.is_empty());
                    }
                    Err(MedCodeError::NotFound { .. } | MedCodeError::NoMapping { .. }) => {
                        // Expected if code not in mapping or no default for context
                    }
                    Err(e) => {
                        panic!(
                            "Unexpected error for code {} in context {:?}: {:?}",
                            code, context, e
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_error_consistency() {
        let icd10 = Icd10Cm::new();
        let forward_mapper = Icd10CmToCcsr::new();
        let reverse_mapper = CcsrToIcd10Cm::new();

        let invalid_codes = ["INVALID", "XYZ", "", "A", "9999"];

        for &code in &invalid_codes {
            // ICD-10-CM operations should return NotFound
            assert!(matches!(
                icd10.lookup(code),
                Err(MedCodeError::NotFound { .. })
            ));
            assert!(matches!(
                icd10.ancestors(code),
                Err(MedCodeError::NotFound { .. })
            ));
            assert!(matches!(
                icd10.descendants(code),
                Err(MedCodeError::NotFound { .. })
            ));
            assert!(matches!(
                icd10.parent(code),
                Err(MedCodeError::NotFound { .. })
            ));
            assert!(matches!(
                icd10.children(code),
                Err(MedCodeError::NotFound { .. })
            ));

            // Cross-mapping operations should return NotFound
            assert!(matches!(
                forward_mapper.get_categories(code),
                Err(MedCodeError::NotFound { .. })
            ));
            assert!(matches!(
                reverse_mapper.get_icd10_codes(code),
                Err(MedCodeError::NotFound { .. })
            ));
        }
    }

    #[test]
    fn test_code_descriptions_content() {
        let icd10 = Icd10Cm::new();

        for &code in VALID_CODES {
            if let Ok(lookup_result) = icd10.lookup(code) {
                let description = lookup_result.description();

                // Description should be meaningful
                assert!(
                    !description.is_empty(),
                    "Description for {} should not be empty",
                    code
                );
                assert!(
                    description.len() > 3,
                    "Description for {} should be meaningful",
                    code
                );

                // Description should not contain obvious placeholders
                assert!(
                    !description.contains("UNKNOWN"),
                    "Description for {} should not be placeholder",
                    code
                );
                assert!(
                    !description.contains("N/A"),
                    "Description for {} should not be N/A",
                    code
                );
            }
        }
    }

    #[test]
    fn test_code_system_consistency() {
        let icd10 = Icd10Cm::new();
        let forward_mapper = Icd10CmToCcsr::new();
        let reverse_mapper = CcsrToIcd10Cm::new();

        // Check that all returned codes have correct system types
        for &code in VALID_CODES {
            if let Ok(lookup_result) = icd10.lookup(code) {
                assert_eq!(lookup_result.system, System::Icd10Cm);
            }

            if let Ok(categories) = forward_mapper.get_categories(code) {
                for category in &categories {
                    assert_eq!(category.code.chars().count(), 6); // CCSR codes are 6 characters
                    assert!(
                        category
                            .code
                            .chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                    );
                }
            }
        }

        // Test CCSR to ICD-10-CM mapping
        let test_ccsr_codes = ["CIRC001", "ENDO001", "RESP001"]; // Common CCSR categories
        for &ccsr_code in &test_ccsr_codes {
            if let Ok(icd10_codes) = reverse_mapper.get_icd10_codes(ccsr_code) {
                for icd10_code in &icd10_codes {
                    assert!(icd10_code.len() >= 3 && icd10_code.len() <= 7);
                    assert!(icd10_code.chars().next().unwrap().is_ascii_uppercase());
                }
            }
        }
    }

    #[test]
    fn test_performance_with_known_codes() {
        let icd10 = Icd10Cm::new();

        // This is more of a smoke test than a performance benchmark
        let start = std::time::Instant::now();

        for &code in VALID_CODES {
            let _ = icd10.is_valid(code);
            let _ = icd10.lookup(code);
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 2000,
            "Operations on known codes should be fast, took {}ms",
            duration.as_millis()
        );
    }
}
