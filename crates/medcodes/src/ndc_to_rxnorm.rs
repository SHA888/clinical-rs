//! NDC to `RxNorm` cross-mapping module

use crate::{Code, CrossMap, MedCodeError, System};
use phf::phf_map;

/// NDC to `RxNorm` cross-map implementation
#[derive(Debug)]
pub struct NdcToRxNorm {
    /// Maps NDC codes to `RxNorm` codes
    mappings: &'static phf::Map<&'static str, &'static str>,
}

impl Default for NdcToRxNorm {
    fn default() -> Self {
        Self::new()
    }
}

impl NdcToRxNorm {
    /// Create a new NDC to `RxNorm` cross-map
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: &NDC_TO_RXNORM_MAPPINGS,
        }
    }
}

impl CrossMap for NdcToRxNorm {
    /// Map an NDC code to `RxNorm` codes
    ///
    /// # Errors
    ///
    /// Returns an error if the NDC code cannot be mapped to `RxNorm`
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, MedCodeError> {
        if target_system != System::RxNorm {
            return Err(MedCodeError::no_mapping(code, System::Ndc, target_system));
        }

        let normalized_code = code.replace('-', "").to_uppercase();

        self.mappings.get(&normalized_code).map_or_else(
            || Err(MedCodeError::no_mapping(code, System::Ndc, System::RxNorm)),
            |rxnorm_code| {
                Ok(vec![Code {
                    system: System::RxNorm,
                    code: rxnorm_code.to_string(),
                    description: "RxNorm Concept".to_string(), // TODO: Look up RxNorm description from RxNorm data
                }])
            },
        )
    }

    fn source_system(&self) -> System {
        System::Ndc
    }

    fn target_system(&self) -> System {
        System::RxNorm
    }
}

// Include generated data from build.rs
include!(concat!(env!("OUT_DIR"), "/ndc_to_rxnorm_data.rs"));

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use super::*;

    #[test]
    fn test_ndc_to_rxnorm_new() {
        let mapper = NdcToRxNorm::new();
        // Test with a sample mapping
        assert!(!mapper.mappings.is_empty());
    }

    #[test]
    fn test_ndc_to_rxnorm_mapping() {
        let mapper = NdcToRxNorm::new();

        // Test with a known mapping
        match mapper.map("12345-6789-01", System::RxNorm) {
            Ok(mapped_codes) => {
                assert!(!mapped_codes.is_empty());
                assert_eq!(mapped_codes[0].system, System::RxNorm);
                println!(
                    "✓ NDC 12345-6789-01 maps to RxNorm {}",
                    mapped_codes[0].code
                );
            }
            Err(e) => {
                panic!("Failed to map NDC to RxNorm: {e}");
            }
        }

        // Test with unknown NDC
        match mapper.map("9999-9999-99", System::RxNorm) {
            Ok(_) => {
                panic!("Unexpectedly mapped unknown NDC code");
            }
            Err(_) => {
                println!("✓ Unknown NDC correctly failed to map");
            }
        }
    }
}
