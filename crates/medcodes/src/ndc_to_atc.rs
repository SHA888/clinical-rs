//! NDC to ATC cross-mapping module

use crate::{Code, CrossMap, MedCodeError, System};
use phf::phf_map;

/// NDC to ATC cross-map implementation
#[derive(Debug)]
pub struct NdcToAtc {
    /// Maps NDC codes to ATC codes
    mappings: &'static phf::Map<&'static str, &'static str>,
}

impl Default for NdcToAtc {
    fn default() -> Self {
        Self::new()
    }
}

impl NdcToAtc {
    /// Create a new NDC to ATC cross-map
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: &NDC_TO_ATC_MAPPINGS,
        }
    }
}

impl CrossMap for NdcToAtc {
    /// Map an NDC code to ATC codes
    ///
    /// # Errors
    ///
    /// Returns an error if the NDC code cannot be mapped to ATC
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, MedCodeError> {
        if target_system != System::Atc {
            return Err(MedCodeError::no_mapping(code, System::Ndc, target_system));
        }

        let normalized_code = code.replace('-', "").to_uppercase();

        self.mappings.get(&normalized_code).map_or_else(
            || Err(MedCodeError::no_mapping(code, System::Ndc, System::Atc)),
            |atc_code| {
                Ok(vec![Code {
                    system: System::Atc,
                    code: atc_code.to_string(),
                    description: "ATC Code".to_string(), // TODO: Look up ATC description from ATC data
                }])
            },
        )
    }

    fn source_system(&self) -> System {
        System::Ndc
    }

    fn target_system(&self) -> System {
        System::Atc
    }
}

// Include generated data from build.rs
include!(concat!(env!("OUT_DIR"), "/ndc_to_atc_data.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndc_to_atc_new() {
        let mapper = NdcToAtc::new();
        // Test with a sample mapping
        assert!(!mapper.mappings.is_empty());
    }

    #[test]
    fn test_ndc_to_atc_mapping() {
        let mapper = NdcToAtc::new();

        // Test with a known mapping
        match mapper.map("12345-6789-01", System::Atc) {
            Ok(mapped_codes) => {
                assert!(!mapped_codes.is_empty());
                assert_eq!(mapped_codes[0].system, System::Atc);
                println!("✓ NDC 12345-6789-01 maps to ATC {}", mapped_codes[0].code);
            }
            Err(e) => {
                panic!("Failed to map NDC to ATC: {}", e);
            }
        }

        // Test with unknown NDC
        match mapper.map("9999-9999-99", System::Atc) {
            Ok(_) => {
                panic!("Unexpectedly mapped unknown NDC code");
            }
            Err(_) => {
                println!("✓ Unknown NDC correctly failed to map");
            }
        }
    }
}
