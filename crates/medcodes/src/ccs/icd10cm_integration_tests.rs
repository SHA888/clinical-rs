#[cfg(test)]
mod icd10cm_ccs_integration_tests {
    use crate::{CrossMap, Icd10CmToCcs, System};

    #[test]
    #[allow(clippy::panic)]
    fn test_icd10cm_to_ccs_mapping() {
        let mapper = Icd10CmToCcs::new();

        // Test with sample codes from our CSV
        let test_cases = [
            ("A00.0", Some("1.0")),
            ("A00.1", Some("1.0")),
            ("A01.0", Some("1.0")),
            ("A01.1", Some("1.0")),
            ("Z99.99", None), // Should not exist
        ];

        for (icd10_code, expected_ccs) in test_cases {
            match mapper.map(icd10_code, System::Ccs) {
                Ok(mapped_codes) => {
                    if let Some(expected) = expected_ccs {
                        assert!(
                            !mapped_codes.is_empty(),
                            "Expected mapping for {icd10_code}"
                        );
                        assert_eq!(
                            mapped_codes[0].code, expected,
                            "Expected {icd10_code} to map to {expected}"
                        );
                        println!(
                            "✓ ICD-10-CM {icd10_code} maps to CCS {}",
                            mapped_codes[0].code
                        );
                    } else {
                        panic!("Unexpected mapping for {icd10_code}: {mapped_codes:?}");
                    }
                }
                Err(_) => {
                    if expected_ccs.is_none() {
                        println!("✓ ICD-10-CM {icd10_code} correctly failed to map");
                    } else if let Some(expected) = expected_ccs {
                        panic!(
                            "Failed to map {icd10_code} but expected mapping to {expected}"
                        );
                    }
                }
            }
        }
    }
}
