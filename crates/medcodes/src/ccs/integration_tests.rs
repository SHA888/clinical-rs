#[cfg(test)]
mod ccs_integration_tests {
    use crate::{CrossMap, Icd9CmToCcs, System};

    #[test]
    fn test_icd9cm_to_ccs_mapping() {
        let mapper = Icd9CmToCcs::new();

        // Test with sample codes from our CSV
        let test_cases = [
            ("001", Some("1.0")),
            ("001.0", Some("1.0")),
            ("002", Some("1.0")),
            ("002.0", Some("1.0")),
            ("999.99", None), // Should not exist
        ];

        for (icd9_code, expected_ccs) in test_cases {
            match mapper.map(icd9_code, System::Ccs) {
                Ok(mapped_codes) => {
                    if let Some(expected) = expected_ccs {
                        assert!(
                            !mapped_codes.is_empty(),
                            "Expected mapping for {}",
                            icd9_code
                        );
                        assert_eq!(
                            mapped_codes[0].code, expected,
                            "Expected {} to map to {}",
                            icd9_code, expected
                        );
                        println!(
                            "✓ ICD-9-CM {} maps to CCS {}",
                            icd9_code, mapped_codes[0].code
                        );
                    } else {
                        panic!("Unexpected mapping for {}: {:?}", icd9_code, mapped_codes);
                    }
                }
                Err(_) => {
                    if expected_ccs.is_none() {
                        println!("✓ ICD-9-CM {} correctly failed to map", icd9_code);
                    } else if let Some(expected) = expected_ccs {
                        panic!(
                            "Failed to map {} but expected mapping to {}",
                            icd9_code, expected
                        );
                    }
                }
            }
        }
    }
}
