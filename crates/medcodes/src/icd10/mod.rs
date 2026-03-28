//! ICD-10-CM (International Classification of Diseases, 10th Revision, Clinical Modification)
//!
//! This module provides ICD-10-CM code lookup, validation, and hierarchy traversal.

use crate::{Code, CodeSystem, MedCodeError, System};
use phf::phf_map;

/// ICD-10-CM code system implementation.
#[derive(Debug)]
pub struct Icd10Cm {
    /// Map of normalized codes to descriptions
    descriptions: &'static phf::Map<&'static str, &'static str>,
    /// Map of codes to their parent codes
    parents: &'static phf::Map<&'static str, Option<&'static str>>,
    /// Map of codes to their children codes
    children: &'static phf::Map<&'static str, &'static [&'static str]>,
}

impl Icd10Cm {
    /// Create a new `Icd10Cm` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptions: &ICD10_CM_DESCRIPTIONS,
            parents: &ICD10_CM_PARENTS,
            children: &ICD10_CM_CHILDREN,
        }
    }

    /// Normalize an ICD-10-CM code.
    #[must_use]
    pub fn normalize_code(&self, code: &str) -> String {
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

impl Default for Icd10Cm {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeSystem for Icd10Cm {
    /// Look up an ICD-10-CM code.
    fn lookup(&self, code: &str) -> Result<Code, MedCodeError> {
        let normalized = self.normalize_code(code);
        self.descriptions.get(normalized.as_str()).map_or_else(
            || Err(MedCodeError::not_found(code, System::Icd10Cm)),
            |description| {
                Ok(Code {
                    system: System::Icd10Cm,
                    code: normalized,
                    description: description.to_string(),
                })
            },
        )
    }

    /// Check if a code is valid in ICD-10-CM.
    fn is_valid(&self, code: &str) -> bool {
        let normalized = self.normalize_code(code);
        self.descriptions.contains_key(normalized.as_str())
    }

    /// Normalize a code (remove formatting, uppercase, etc.).
    fn normalize(&self, code: &str) -> String {
        // Delegate to the inherent method with a different name
        self.normalize_code(code)
    }

    /// Get all ancestors of a code in the ICD-10-CM hierarchy.
    fn ancestors(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        // Validate the code exists first
        if !self.descriptions.contains_key(normalized.as_str()) {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        }
        let mut ancestors = Vec::new();
        let mut current: String = normalized;

        while let Some(Some(parent)) = self.parents.get(current.as_str()) {
            if let Some(description) = self.descriptions.get(parent) {
                ancestors.push(Code {
                    system: System::Icd10Cm,
                    code: parent.to_string(),
                    description: description.to_string(),
                });
                current = parent.to_string();
            } else {
                break;
            }
        }

        Ok(ancestors)
    }

    /// Get all descendants of a code in the ICD-10-CM hierarchy.
    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        // Validate the code exists first
        if !self.descriptions.contains_key(normalized.as_str()) {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        }
        let mut descendants = Vec::new();
        let mut to_visit = vec![normalized];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = to_visit.pop() {
            // Skip if already visited to prevent cycles
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(children) = self.children.get(current.as_str()) {
                for child in *children {
                    if let Some(description) = self.descriptions.get(child) {
                        descendants.push(Code {
                            system: System::Icd10Cm,
                            code: child.to_string(),
                            description: description.to_string(),
                        });
                        to_visit.push(child.to_string());
                    }
                }
            }
        }

        Ok(descendants)
    }

    /// Get the immediate parent of a code.
    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        // Validate the code exists first
        if !self.descriptions.contains_key(normalized.as_str()) {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        }

        self.parents
            .get(normalized.as_str())
            .map_or(Ok(None), |parent_opt| {
                parent_opt.as_ref().map_or(Ok(None), |parent| {
                    self.descriptions
                        .get(parent)
                        .map_or(Ok(None), |description| {
                            Ok(Some(Code {
                                system: System::Icd10Cm,
                                code: parent.to_string(),
                                description: description.to_string(),
                            }))
                        })
                })
            })
    }

    /// Get all immediate children of a code.
    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        // Validate the code exists first
        if !self.descriptions.contains_key(normalized.as_str()) {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        }

        self.children.get(normalized.as_str()).map_or_else(
            || Ok(Vec::new()),
            |children| {
                let mut result = Vec::new();
                for child in *children {
                    if let Some(description) = self.descriptions.get(child) {
                        result.push(Code {
                            system: System::Icd10Cm,
                            code: child.to_string(),
                            description: description.to_string(),
                        });
                    }
                }
                Ok(result)
            },
        )
    }
}

// Static data generated by build.rs from CMS ICD-10-CM code tables
include!(concat!(env!("OUT_DIR"), "/icd10cm_data.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_normalize() {
        let icd10 = Icd10Cm::new();
        assert_eq!(icd10.normalize_code("I10.9"), "I109");
        assert_eq!(icd10.normalize_code("i10.9"), "I109");
        assert_eq!(icd10.normalize_code("A01"), "A01");
        assert_eq!(icd10.normalize_code("  I10.9  "), "I109"); // Test whitespace trimming
        assert_eq!(icd10.normalize_code("\tI10.9\n"), "I109"); // Test tab/newline trimming
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_is_valid() {
        let icd10 = Icd10Cm::new();
        // These will be false until we populate the data
        assert!(!icd10.is_valid("I10.9"));
        assert!(!icd10.is_valid("INVALID"));
        assert!(!icd10.is_valid("  I10.9  ")); // Test with whitespace
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parent_error_handling() {
        let icd10 = Icd10Cm::new();
        // Test unknown code returns error
        let result = icd10.parent("INVALID");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MedCodeError::NotFound { code, system }
            if code == "INVALID" && system == System::Icd10Cm));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_children_error_handling() {
        let icd10 = Icd10Cm::new();
        // Test unknown code returns error
        let result = icd10.children("INVALID");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MedCodeError::NotFound { code, system }
            if code == "INVALID" && system == System::Icd10Cm));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_ancestors_error_handling() {
        let icd10 = Icd10Cm::new();
        // Test unknown code returns error
        let result = icd10.ancestors("INVALID");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MedCodeError::NotFound { code, system }
            if code == "INVALID" && system == System::Icd10Cm));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_descendants_error_handling() {
        let icd10 = Icd10Cm::new();
        // Test unknown code returns error
        let result = icd10.descendants("INVALID");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MedCodeError::NotFound { code, system }
            if code == "INVALID" && system == System::Icd10Cm));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_lookup_error_handling() {
        let icd10 = Icd10Cm::new();
        // Test unknown code returns error
        let result = icd10.lookup("INVALID");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MedCodeError::NotFound { code, system }
            if code == "INVALID" && system == System::Icd10Cm));
    }
}
