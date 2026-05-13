//! LOINC terminology support.
//!
//! This module provides lookup and component extraction for LOINC codes.
//! It is gated behind the `loinc` feature flag.

use crate::CodeSystem;
use crate::types::{Code, MedCodeError, System};
use phf::phf_map;

/// Component details for a LOINC code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoincComponents {
    /// LOINC component axis (what is measured).
    pub component: &'static str,
    /// LOINC property axis (kind of quantity/observation).
    pub property: &'static str,
    /// LOINC timing axis.
    pub timing: &'static str,
    /// LOINC system axis (specimen/system measured).
    pub system: &'static str,
    /// LOINC scale axis.
    pub scale: &'static str,
    /// Optional LOINC method axis.
    pub method: Option<&'static str>,
}

// Placeholder maps – real generation will replace these.
static DESCRIPTIONS: phf::Map<&'static str, &'static str> = phf_map! {
    // Example entry – replace with generated data.
    "2160-0" => "Creatinine [Mass/volume] in Serum or Plasma",
};

static COMPONENTS: phf::Map<&'static str, LoincComponents> = phf_map! {
    "2160-0" => LoincComponents {
        component: "Creatinine",
        property: "Mass",
        timing: "Pt",
        system: "Ser/Plas",
        scale: "Qn",
        method: None,
    },
};

/// LOINC code system implementation.
pub struct Loinc;

impl Default for Loinc {
    fn default() -> Self {
        Self::new()
    }
}

impl Loinc {
    /// Create a new instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Retrieve component details for a given LOINC code.
    ///
    /// # Errors
    ///
    /// Returns [`MedCodeError::NotFound`] when the code does not exist in the loaded map.
    pub fn components(&self, code: &str) -> Result<LoincComponents, MedCodeError> {
        let norm = self.normalize(code);
        COMPONENTS
            .get(norm.as_str())
            .cloned()
            .ok_or_else(|| MedCodeError::NotFound {
                system: System::Loinc,
                code: code.to_string(),
            })
    }
}

impl CodeSystem for Loinc {
    fn lookup(&self, code: &str) -> Result<Code, MedCodeError> {
        let norm = self.normalize(code);
        DESCRIPTIONS
            .get(norm.as_str())
            .map(|desc| Code {
                system: System::Loinc,
                code: norm.clone(),
                description: desc.to_string(),
            })
            .ok_or_else(|| MedCodeError::NotFound {
                system: System::Loinc,
                code: code.to_string(),
            })
    }

    fn ancestors(&self, _code: &str) -> Result<Vec<Code>, MedCodeError> {
        // LOINC does not have a hierarchical ancestry in this implementation.
        Ok(vec![])
    }

    fn descendants(&self, _code: &str) -> Result<Vec<Code>, MedCodeError> {
        // No hierarchy.
        Ok(vec![])
    }

    fn is_valid(&self, code: &str) -> bool {
        let norm = self.normalize(code);
        DESCRIPTIONS.contains_key(norm.as_str())
    }

    fn normalize(&self, code: &str) -> String {
        // LOINC codes are stored with hyphen; keep as-is for lookup.
        code.trim().to_string()
    }

    fn parent(&self, _code: &str) -> Result<Option<Code>, MedCodeError> {
        Ok(None)
    }

    fn children(&self, _code: &str) -> Result<Vec<Code>, MedCodeError> {
        Ok(vec![])
    }
}
