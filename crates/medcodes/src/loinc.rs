//! LOINC terminology support.
//!
//! This module provides lookup and component extraction for LOINC codes.
//! It is gated behind the `loinc` feature flag.

use crate::CodeSystem;
use crate::types::{Code, MedCodeError, System};

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

// Include the generated data from build.rs
include!(concat!(env!("OUT_DIR"), "/loinc_data.rs"));

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
        LOINC_COMPONENTS
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
        LOINC_DESCRIPTIONS
            .get(norm.as_str())
            .map(|desc: &&str| Code {
                system: System::Loinc,
                code: norm.clone(),
                description: desc.to_string(),
            })
            .ok_or_else(|| MedCodeError::NotFound {
                system: System::Loinc,
                code: code.to_string(),
            })
    }

    fn ancestors(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let norm = self.normalize(code);
        let mut ancestors = Vec::new();
        let mut current = norm.as_str();

        while let Some(Some(parent)) = LOINC_PARENTS.get(current) {
            if let Some(desc) = LOINC_DESCRIPTIONS.get(parent) {
                ancestors.push(Code {
                    system: System::Loinc,
                    code: parent.to_string(),
                    description: (*desc).to_string(),
                });
                current = parent;
            } else {
                break;
            }
        }

        Ok(ancestors)
    }

    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let norm = self.normalize(code);
        let mut descendants = Vec::new();
        let mut stack = vec![norm.as_str()];

        while let Some(current) = stack.pop() {
            if let Some(children) = LOINC_CHILDREN.get(current) {
                for &child in children.iter() {
                    if let Some(desc) = LOINC_DESCRIPTIONS.get(child) {
                        descendants.push(Code {
                            system: System::Loinc,
                            code: child.to_string(),
                            description: (*desc).to_string(),
                        });
                        stack.push(child);
                    }
                }
            }
        }

        Ok(descendants)
    }

    fn is_valid(&self, code: &str) -> bool {
        let norm = self.normalize(code);
        LOINC_DESCRIPTIONS.contains_key(norm.as_str())
    }

    fn normalize(&self, code: &str) -> String {
        // LOINC codes are stored with hyphen; keep as-is for lookup.
        code.trim().to_string()
    }

    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError> {
        let norm = self.normalize(code);
        if let Some(Some(parent)) = LOINC_PARENTS.get(norm.as_str()) {
            if let Some(desc) = LOINC_DESCRIPTIONS.get(parent) {
                Ok(Some(Code {
                    system: System::Loinc,
                    code: parent.to_string(),
                    description: (*desc).to_string(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let norm = self.normalize(code);
        if let Some(children) = LOINC_CHILDREN.get(norm.as_str()) {
            let mut result = Vec::new();
            for &child in children.iter() {
                if let Some(desc) = LOINC_DESCRIPTIONS.get(child) {
                    result.push(Code {
                        system: System::Loinc,
                        code: child.to_string(),
                        description: (*desc).to_string(),
                    });
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
}
