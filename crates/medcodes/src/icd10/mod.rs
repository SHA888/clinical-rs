//! ICD-10-CM (International Classification of Diseases, 10th Revision, Clinical Modification)
//!
//! This module provides ICD-10-CM code lookup, validation, and hierarchy traversal.

use crate::{Code, CodeSystem, MedCodeError, System};
use phf::phf_map;

/// ICD-10-CM code system implementation.
#[derive(Debug)]
pub struct Icd10Cm {
    /// Map of normalized codes to Code information
    codes: &'static phf::Map<&'static str, Code>,
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
            codes: &ICD10_CM_CODES,
            parents: &ICD10_CM_PARENTS,
            children: &ICD10_CM_CHILDREN,
        }
    }

    /// Normalize an ICD-10-CM code (remove dots, uppercase).
    ///
    /// This is an inherent method that provides the actual implementation.
    /// The trait method calls this to avoid infinite recursion.
    #[must_use]
    pub fn normalize(&self, code: &str) -> String {
        let mut result = String::with_capacity(code.len());
        for ch in code.chars() {
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
        let normalized = self.normalize(code);
        self.codes
            .get(&normalized)
            .cloned()
            .ok_or_else(|| MedCodeError::not_found(code, System::Icd10Cm))
    }

    /// Check if a code is valid in ICD-10-CM.
    fn is_valid(&self, code: &str) -> bool {
        let normalized = self.normalize(code);
        self.codes.contains_key(&normalized)
    }

    /// Normalize an ICD-10-CM code.
    fn normalize(&self, code: &str) -> String {
        // Call the struct's normalize method to avoid infinite recursion
        // We need to call the inherent method, not the trait method
        Self::normalize(self, code)
    }

    /// Get all ancestors of a code in the ICD-10-CM hierarchy.
    fn ancestors(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);
        // Validate the code exists first
        if !self.codes.contains_key(&normalized) {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        }
        let mut ancestors = Vec::new();
        let mut current: String = normalized;

        while let Some(Some(parent)) = self.parents.get(current.as_str()) {
            if let Some(parent_code) = self.codes.get(parent) {
                ancestors.push(parent_code.clone());
                current = parent.to_string();
            } else {
                break;
            }
        }

        Ok(ancestors)
    }

    /// Get all descendants of a code in the ICD-10-CM hierarchy.
    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);
        // Validate the code exists first
        if !self.codes.contains_key(&normalized) {
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
            if let Some(children) = self.children.get(&current) {
                for child in *children {
                    if let Some(child_code) = self.codes.get(child) {
                        descendants.push(child_code.clone());
                        to_visit.push(child.to_string());
                    }
                }
            }
        }

        Ok(descendants)
    }

    /// Get the immediate parent of a code.
    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError> {
        let normalized = self.normalize(code);
        if let Some(Some(parent)) = self.parents.get(&normalized) {
            self.codes
                .get(parent)
                .map_or(Ok(None), |parent_code| Ok(Some(parent_code.clone())))
        } else {
            Ok(None)
        }
    }

    /// Get all immediate children of a code.
    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);
        self.children.get(&normalized).map_or_else(
            || Ok(Vec::new()),
            |children| {
                let mut result = Vec::new();
                for child in *children {
                    if let Some(child_code) = self.codes.get(child) {
                        result.push(child_code.clone());
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
    fn test_normalize() {
        let icd10 = Icd10Cm::new();
        assert_eq!(icd10.normalize("I10.9"), "I109");
        assert_eq!(icd10.normalize("i10.9"), "I109");
        assert_eq!(icd10.normalize("A01"), "A01");
    }

    #[test]
    fn test_is_valid() {
        let icd10 = Icd10Cm::new();
        // These will be false until we populate the data
        assert!(!icd10.is_valid("I10.9"));
        assert!(!icd10.is_valid("INVALID"));
    }
}
