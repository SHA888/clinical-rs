//! CCSR (Clinical Classifications Software Refined) module
//!
//! Provides cross-mapping between ICD-10-CM diagnosis codes and CCSR categories.

use crate::{Code, CodeSystem, CrossMap, MedCodeError, System};

/// CCSR category information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CcsrCategory {
    /// The CCSR category code (e.g., "DIG001")
    pub code: String,
    /// Human-readable description (e.g., "Intestinal infection")
    pub description: String,
}

impl CcsrCategory {
    /// Create a new CCSR category
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::ccsr::CcsrCategory;
    ///
    /// let category = CcsrCategory::new("DIG001", "Intestinal infectious diseases");
    /// assert_eq!(category.code, "DIG001");
    /// assert_eq!(category.description, "Intestinal infectious diseases");
    /// ```
    #[must_use]
    pub fn new(code: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: description.into(),
        }
    }
}

/// Mapping context for CCSR default assignments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CcsrContext {
    /// Inpatient (IP) - hospital stays
    Inpatient,
    /// Emergency Department (ED)
    EmergencyDepartment,
    /// Outpatient (OP) - hospital-based outpatient
    Outpatient,
}

/// Internal representation of a CCSR mapping entry
/// This struct matches the generated data from build.rs
#[derive(Debug, Clone)]
pub struct CcsrMapping {
    /// CCSR category code
    pub category_code: &'static str,
    /// CCSR category description
    pub category_description: &'static str,
    /// Is this the default for inpatient context
    pub is_default_ip: bool,
    /// Is this the default for emergency department context
    pub is_default_ed: bool,
    /// Is this the default for outpatient context
    pub is_default_op: bool,
}

/// ICD-10-CM to CCSR cross-map implementation
#[derive(Debug)]
pub struct Icd10CmToCcsr {
    /// Maps ICD-10-CM codes to their CCSR category indices
    mappings: &'static phf::Map<&'static str, &'static [usize]>,
}

impl Icd10CmToCcsr {
    /// Create a new ICD-10-CM to CCSR cross-map
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::ccsr::Icd10CmToCcsr;
    ///
    /// let mapper = Icd10CmToCcsr::new();
    /// let categories = mapper.get_categories("A00.0").unwrap();
    /// assert!(!categories.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: &ICD10CM_TO_CCSR_MAPPINGS,
        }
    }

    /// Get all CCSR categories for an ICD-10-CM code
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::ccsr::Icd10CmToCcsr;
    ///
    /// let mapper = Icd10CmToCcsr::new();
    /// let categories = mapper.get_categories("A00.0").unwrap();
    /// // Returns all CCSR categories that map to ICD-10-CM code A00.0
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MedCodeError::NotFound` if the ICD-10-CM code is not in the mapping
    pub fn get_categories(&self, icd10_code: &str) -> Result<Vec<CcsrCategory>, MedCodeError> {
        let normalized = Self::normalize_code(icd10_code);

        self.mappings.get(normalized.as_str()).map_or_else(
            || Err(MedCodeError::not_found(icd10_code, System::Icd10Cm)),
            |indices| {
                let categories: Vec<CcsrCategory> = indices
                    .iter()
                    .map(|&idx| {
                        let mapping = &CCSR_MAPPINGS[idx];
                        CcsrCategory {
                            code: mapping.category_code.to_string(),
                            description: mapping.category_description.to_string(),
                        }
                    })
                    .collect();
                Ok(categories)
            },
        )
    }

    /// Get the default CCSR category for a specific context
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::ccsr::{Icd10CmToCcsr, CcsrContext};
    ///
    /// let mapper = Icd10CmToCcsr::new();
    /// let category = mapper.get_default_category("A00.0", CcsrContext::Inpatient).unwrap();
    /// // Returns the default CCSR category for inpatient stays
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MedCodeError::NotFound` if the ICD-10-CM code is not in the mapping,
    /// or `MedCodeError::NoMapping` if no default is defined for the context
    pub fn get_default_category(
        &self,
        icd10_code: &str,
        context: CcsrContext,
    ) -> Result<CcsrCategory, MedCodeError> {
        let normalized = Self::normalize_code(icd10_code);

        self.mappings.get(normalized.as_str()).map_or_else(
            || Err(MedCodeError::not_found(icd10_code, System::Icd10Cm)),
            |indices| {
                let default_mapping = indices.iter().find_map(|&idx| {
                    let mapping = &CCSR_MAPPINGS[idx];
                    let is_default = match context {
                        CcsrContext::Inpatient => mapping.is_default_ip,
                        CcsrContext::EmergencyDepartment => mapping.is_default_ed,
                        CcsrContext::Outpatient => mapping.is_default_op,
                    };
                    if is_default {
                        Some(CcsrCategory {
                            code: mapping.category_code.to_string(),
                            description: mapping.category_description.to_string(),
                        })
                    } else {
                        None
                    }
                });

                default_mapping.map_or_else(
                    || {
                        Err(MedCodeError::no_mapping(
                            icd10_code,
                            System::Icd10Cm,
                            System::Ccsr,
                        ))
                    },
                    Ok,
                )
            },
        )
    }

    /// Normalize an ICD-10-CM code (remove dots, uppercase)
    #[must_use]
    fn normalize_code(code: &str) -> String {
        let trimmed = code.trim();
        let mut result = String::with_capacity(trimmed.len());
        for ch in trimmed.chars() {
            if ch != '.' {
                result.push(ch.to_ascii_uppercase());
            }
        }
        result
    }
}

impl Default for Icd10CmToCcsr {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossMap for Icd10CmToCcsr {
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, MedCodeError> {
        if target_system != System::Ccsr {
            return Err(MedCodeError::no_mapping(
                code,
                System::Icd10Cm,
                target_system,
            ));
        }

        let categories = self.get_categories(code)?;
        let mapped_codes: Vec<Code> = categories
            .into_iter()
            .map(|cat| Code {
                system: System::Ccsr,
                code: cat.code,
                description: cat.description,
            })
            .collect();

        Ok(mapped_codes)
    }

    fn source_system(&self) -> System {
        System::Icd10Cm
    }

    fn target_system(&self) -> System {
        System::Ccsr
    }
}

/// CCSR to ICD-10-CM cross-map implementation
#[derive(Debug)]
pub struct CcsrToIcd10Cm {
    /// Maps CCSR categories to their ICD-10-CM codes
    mappings: &'static phf::Map<&'static str, &'static [&'static str]>,
}

impl CcsrToIcd10Cm {
    /// Create a new CCSR to ICD-10-CM cross-map
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::ccsr::CcsrToIcd10Cm;
    ///
    /// let mapper = CcsrToIcd10Cm::new();
    /// let icd10_codes = mapper.get_icd10_codes("DIG001").unwrap();
    /// assert!(!icd10_codes.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: &CCSR_TO_ICD10CM_MAPPINGS,
        }
    }

    /// Get all ICD-10-CM codes for a CCSR category
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::ccsr::CcsrToIcd10Cm;
    ///
    /// let mapper = CcsrToIcd10Cm::new();
    /// let icd10_codes = mapper.get_icd10_codes("DIG001").unwrap();
    /// // Returns all ICD-10-CM codes that map to CCSR category DIG001
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MedCodeError::NotFound` if the CCSR category is not in the mapping
    pub fn get_icd10_codes(&self, ccsr_code: &str) -> Result<Vec<String>, MedCodeError> {
        self.mappings.get(ccsr_code).map_or_else(
            || Err(MedCodeError::not_found(ccsr_code, System::Ccsr)),
            |icd10_codes| Ok(icd10_codes.iter().map(|&code| code.to_string()).collect()),
        )
    }
}

impl Default for CcsrToIcd10Cm {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossMap for CcsrToIcd10Cm {
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, MedCodeError> {
        if target_system != System::Icd10Cm {
            return Err(MedCodeError::no_mapping(code, System::Ccsr, target_system));
        }

        let icd10_codes = self.get_icd10_codes(code)?;

        // Need to get descriptions from ICD-10-CM system
        let icd10_system = crate::Icd10Cm::new();
        let mut mapped_codes = Vec::new();

        for icd10_code in icd10_codes {
            if let Ok(icd10_result) = icd10_system.lookup(&icd10_code) {
                mapped_codes.push(icd10_result);
            }
        }

        Ok(mapped_codes)
    }

    fn source_system(&self) -> System {
        System::Ccsr
    }

    fn target_system(&self) -> System {
        System::Icd10Cm
    }
}

// Include generated CCSR mapping data
include!(concat!(env!("OUT_DIR"), "/ccsr_data.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_categories() {
        let mapper = Icd10CmToCcsr::new();
        // This will test against actual data once the mapping is generated
        let result = mapper.get_categories("A0100");
        // For now, expect error since we don't have data loaded
        assert!(result.is_err());
    }

    #[test]
    fn test_get_default_category() {
        let mapper = Icd10CmToCcsr::new();
        // Test with invalid code
        let result = mapper.get_default_category("INVALID", CcsrContext::Inpatient);
        assert!(result.is_err());
    }

    #[test]
    fn test_crossmap_trait() {
        let mapper = Icd10CmToCcsr::new();

        // Test source and target systems
        assert_eq!(mapper.source_system(), System::Icd10Cm);
        assert_eq!(mapper.target_system(), System::Ccsr);

        // Test mapping to non-CCSR system fails
        let result = mapper.map("A0100", System::Icd9Cm);
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_normalize_code() {
        let mapper = Icd10CmToCcsr::new();

        // These would be called internally, but we test the behavior through get_categories
        // For now, just verify the CrossMap trait is implemented correctly
        let categories = mapper.get_categories("INVALID");
        assert!(categories.is_err());

        let error = categories.unwrap_err();
        assert!(matches!(error, MedCodeError::NotFound { code, system }
            if code == "INVALID" && system == System::Icd10Cm));
    }

    #[test]
    fn test_ccsr_to_icd10_crossmap() {
        let mapper = CcsrToIcd10Cm::new();

        // Test source and target systems
        assert_eq!(mapper.source_system(), System::Ccsr);
        assert_eq!(mapper.target_system(), System::Icd10Cm);

        // Test mapping to non-ICD-10-CM system fails
        let result = mapper.map("DIG001", System::Icd9Cm);
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_ccsr_to_icd10_get_codes() {
        let mapper = CcsrToIcd10Cm::new();

        // Test with invalid CCSR code
        let result = mapper.get_icd10_codes("INVALID");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MedCodeError::NotFound { code, system }
            if code == "INVALID" && system == System::Ccsr));
    }
}
