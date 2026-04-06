//! CCS (Clinical Classifications Software) module
//!
//! Provides CCS categories and cross-mapping from ICD diagnosis codes.

use crate::{Code, CrossMap, MedCodeError, System};
use phf::phf_map;

/// CCS category information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CcsCategory {
    /// The CCS category code (e.g., "1.0")
    pub code: String,
    /// Human-readable description (e.g., "Infectious and parasitic diseases")
    pub description: String,
}

impl CcsCategory {
    /// Create a new CCS category
    #[must_use]
    pub fn new(code: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: description.into(),
        }
    }
}

/// ICD-10-CM to CCS cross-map implementation
#[derive(Debug)]
pub struct Icd10CmToCcs {
    /// Maps ICD-10-CM codes to CCS categories
    mappings: &'static phf::Map<&'static str, &'static str>,
    /// CCS category descriptions
    descriptions: &'static phf::Map<&'static str, &'static str>,
}

impl Default for Icd10CmToCcs {
    fn default() -> Self {
        Self::new()
    }
}

impl Icd10CmToCcs {
    /// Create a new ICD-10-CM to CCS cross-map
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: &ICD10CM_TO_CCS_MAPPINGS,
            descriptions: &CCS_DESCRIPTIONS,
        }
    }

    /// Get the CCS category for an ICD-10-CM code
    ///
    /// # Errors
    ///
    /// Returns an error if the ICD-10-CM code cannot be mapped to a CCS category.
    pub fn get_category(&self, code: &str) -> Result<CcsCategory, MedCodeError> {
        // Normalize ICD-10-CM code: uppercase, preserve dots
        let normalized_code = code.to_uppercase();

        if let Some(&ccs_code) = self.mappings.get(&normalized_code) {
            if let Some(&description) = self.descriptions.get(ccs_code) {
                Ok(CcsCategory::new(ccs_code, description))
            } else {
                Err(MedCodeError::data(format!(
                    "CCS code {ccs_code} found but has no description"
                )))
            }
        } else {
            Err(MedCodeError::no_mapping(code, System::Icd10Cm, System::Ccs))
        }
    }
}

impl CrossMap for Icd10CmToCcs {
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, MedCodeError> {
        if target_system != System::Ccs {
            return Err(MedCodeError::no_mapping(
                code,
                System::Icd10Cm,
                target_system,
            ));
        }

        let category = self.get_category(code)?;
        let mapped_code = Code {
            system: System::Ccs,
            code: category.code,
            description: category.description,
        };

        Ok(vec![mapped_code])
    }

    fn source_system(&self) -> System {
        System::Icd10Cm
    }

    fn target_system(&self) -> System {
        System::Ccs
    }
}

/// ICD-9-CM to CCS cross-map implementation
#[derive(Debug)]
pub struct Icd9CmToCcs {
    /// Maps ICD-9-CM codes to CCS categories
    mappings: &'static phf::Map<&'static str, &'static str>,
    /// CCS category descriptions
    descriptions: &'static phf::Map<&'static str, &'static str>,
}

impl Default for Icd9CmToCcs {
    fn default() -> Self {
        Self::new()
    }
}

impl Icd9CmToCcs {
    /// Create a new ICD-9-CM to CCS cross-map
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: &ICD9CM_TO_CCS_MAPPINGS,
            descriptions: &CCS_DESCRIPTIONS,
        }
    }

    /// Get the CCS category for an ICD-9-CM code
    ///
    /// # Errors
    ///
    /// Returns an error if the ICD-9-CM code cannot be mapped to a CCS category.
    pub fn get_category(&self, code: &str) -> Result<CcsCategory, MedCodeError> {
        // Normalize ICD-9-CM code: uppercase, preserve dots
        let normalized_code = code.to_uppercase();

        if let Some(&ccs_code) = self.mappings.get(&normalized_code) {
            if let Some(&description) = self.descriptions.get(ccs_code) {
                Ok(CcsCategory::new(ccs_code, description))
            } else {
                Err(MedCodeError::data(format!(
                    "CCS code {ccs_code} found but has no description"
                )))
            }
        } else {
            Err(MedCodeError::no_mapping(code, System::Icd9Cm, System::Ccs))
        }
    }
}

impl CrossMap for Icd9CmToCcs {
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, MedCodeError> {
        if target_system != System::Ccs {
            return Err(MedCodeError::no_mapping(
                code,
                System::Icd9Cm,
                target_system,
            ));
        }

        let category = self.get_category(code)?;
        let mapped_code = Code {
            system: System::Ccs,
            code: category.code,
            description: category.description,
        };

        Ok(vec![mapped_code])
    }

    fn source_system(&self) -> System {
        System::Icd9Cm
    }

    fn target_system(&self) -> System {
        System::Ccs
    }
}

// Include generated data from build.rs
include!(concat!(env!("OUT_DIR"), "/ccs_data.rs"));

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod icd10cm_integration_tests;

#[cfg(test)]
mod normalization_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icd10cm_to_ccs_new() {
        let mapper = Icd10CmToCcs::new();
        // Test with a sample mapping
        assert!(!mapper.mappings.is_empty());
    }

    #[test]
    fn test_icd9cm_to_ccs_new() {
        let mapper = Icd9CmToCcs::new();
        // Test with a sample mapping
        assert!(!mapper.mappings.is_empty());
    }

    #[test]
    fn test_ccs_category_new() {
        let category = CcsCategory::new("1.0", "Test category");
        assert_eq!(category.code, "1.0");
        assert_eq!(category.description, "Test category");
    }
}
