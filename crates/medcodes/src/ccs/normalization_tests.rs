#[cfg(test)]
mod ccs_normalization_tests {
    use crate::{CrossMap, Icd9CmToCcs, Icd10CmToCcs, System};

    #[test]
    fn test_icd10cm_code_normalization() {
        let mapper = Icd10CmToCcs::new();

        // Test case normalization
        let test_cases = [
            ("a00.0", Some("1.0")), // lowercase -> uppercase
            ("A00.0", Some("1.0")), // already uppercase
            ("A00.9", Some("1.0")), // another valid code
        ];

        for (input_code, expected_ccs) in test_cases {
            match mapper.map(input_code, System::Ccs) {
                Ok(mapped_codes) => {
                    if let Some(expected) = expected_ccs {
                        assert!(
                            !mapped_codes.is_empty(),
                            "Expected mapping for {}",
                            input_code
                        );
                        assert_eq!(
                            mapped_codes[0].code, expected,
                            "Expected {} to map to {}",
                            input_code, expected
                        );
                        println!(
                            "✓ ICD-10-CM {} (normalized) maps to CCS {}",
                            input_code, mapped_codes[0].code
                        );
                    } else {
                        panic!("Unexpected mapping for {}: {:?}", input_code, mapped_codes);
                    }
                }
                Err(_e) => {
                    if expected_ccs.is_none() {
                        println!("✓ ICD-10-CM {} correctly failed to map", input_code);
                    } else if let Some(expected) = expected_ccs {
                        panic!(
                            "Failed to map {} but expected mapping to {}",
                            input_code, expected
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_icd9cm_code_normalization() {
        let mapper = Icd9CmToCcs::new();

        // Test case normalization
        let test_cases = [
            ("001", Some("1.0")),   // already uppercase
            ("001.0", Some("1.0")), // already uppercase with dot
            ("001.1", Some("1.0")), // another valid code
        ];

        for (input_code, expected_ccs) in test_cases {
            match mapper.map(input_code, System::Ccs) {
                Ok(mapped_codes) => {
                    if let Some(expected) = expected_ccs {
                        assert!(
                            !mapped_codes.is_empty(),
                            "Expected mapping for {}",
                            input_code
                        );
                        assert_eq!(
                            mapped_codes[0].code, expected,
                            "Expected {} to map to {}",
                            input_code, expected
                        );
                        println!(
                            "✓ ICD-9-CM {} (normalized) maps to CCS {}",
                            input_code, mapped_codes[0].code
                        );
                    } else {
                        panic!("Unexpected mapping for {}: {:?}", input_code, mapped_codes);
                    }
                }
                Err(_e) => {
                    if expected_ccs.is_none() {
                        println!("✓ ICD-9-CM {} correctly failed to map", input_code);
                    } else if let Some(expected) = expected_ccs {
                        panic!(
                            "Failed to map {} but expected mapping to {}",
                            input_code, expected
                        );
                    }
                }
            }
        }
    }
}
