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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::{icd10::Icd10Cm, CodeSystem};
    ///
    /// let icd10 = Icd10Cm::new();
    /// assert!(icd10.is_valid("A00.0"));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptions: &ICD10_CM_DESCRIPTIONS,
            parents: &ICD10_CM_PARENTS,
            children: &ICD10_CM_CHILDREN,
        }
    }

    /// Normalize an ICD-10-CM code.
    ///
    /// This function:
    /// - Removes all whitespace (leading, trailing, and internal)
    /// - Removes all dots
    /// - Converts to uppercase
    ///
    /// Examples:
    /// - "A00.0" -> "A000"
    /// - " a 00 .0 " -> "A000"
    /// - "a00.0" -> "A000"
    #[must_use]
    pub fn normalize_code(&self, code: &str) -> String {
        let mut result = String::with_capacity(code.len());
        for ch in code.chars() {
            if !ch.is_ascii_whitespace() && ch != '.' {
                result.push(ch.to_ascii_uppercase());
            }
        }
        result
    }

    /// Find the dotted form of a normalized code for description lookup.
    ///
    /// The description map stores codes with dots (e.g., "A00.0"), while
    /// hierarchy operations use normalized codes (e.g., "A000"). This helper
    /// maps from normalized back to dotted form.
    #[must_use]
    fn dotted_form_for_normalized(&self, normalized: &str) -> Option<&'static str> {
        // Use iterator find for more idiomatic code
        // TODO: Consider generating a reverse map at build time for O(1) lookup
        self.descriptions
            .keys()
            .find(|&&dotted_key| self.normalize_code(dotted_key) == normalized)
            .copied()
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

        // Try normalized form first
        if let Some(description) = self.descriptions.get(normalized.as_str()) {
            return Ok(Code {
                system: System::Icd10Cm,
                code: normalized,
                description: description.to_string(),
            });
        }

        // Try to find a dotted form that matches the normalized input
        if let Some(dotted_form) = self.dotted_form_for_normalized(&normalized)
            && let Some(description) = self.descriptions.get(dotted_form)
        {
            return Ok(Code {
                system: System::Icd10Cm,
                code: dotted_form.to_string(),
                description: description.to_string(),
            });
        }

        Err(MedCodeError::not_found(code, System::Icd10Cm))
    }

    /// Check if a code is valid in ICD-10-CM.
    fn is_valid(&self, code: &str) -> bool {
        let normalized = self.normalize_code(code);

        // Check normalized form first
        if self.descriptions.contains_key(normalized.as_str()) {
            return true;
        }

        // Check if there's a dotted form that matches the normalized input
        self.dotted_form_for_normalized(&normalized).is_some()
    }

    /// Normalize a code (remove formatting, uppercase, etc.).
    ///
    /// This trait method delegates to `normalize_code` and has the same behavior:
    /// - Removes all whitespace (leading, trailing, and internal)
    /// - Removes all dots
    /// - Converts to uppercase
    fn normalize(&self, code: &str) -> String {
        // Delegate to the inherent method with a different name
        self.normalize_code(code)
    }

    /// Get all ancestors of a code in the ICD-10-CM hierarchy.
    fn ancestors(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        let trimmed = code.trim().to_uppercase();

        // Validate the code exists first (check both formats)
        let found_key = if self.descriptions.contains_key(normalized.as_str()) {
            &normalized
        } else if self.descriptions.contains_key(trimmed.as_str()) {
            &trimmed
        } else {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        };

        // For hierarchy lookups, always use normalized (non-dotted) format
        let hierarchy_key = self.normalize_code(found_key);

        let mut ancestors = Vec::new();
        let mut current: String = hierarchy_key;

        while let Some(Some(parent)) = self.parents.get(current.as_str()) {
            // Find the dotted form of the parent to get its description
            if let Some(dotted_parent) = self.dotted_form_for_normalized(parent) {
                if let Some(description) = self.descriptions.get(dotted_parent) {
                    ancestors.push(Code {
                        system: System::Icd10Cm,
                        code: dotted_parent.to_string(),
                        description: description.to_string(),
                    });
                    current = parent.to_string();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(ancestors)
    }

    /// Get all descendants of a code in the ICD-10-CM hierarchy.
    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        let trimmed = code.trim().to_uppercase();

        // Validate the code exists first (check both formats)
        let found_key = if self.descriptions.contains_key(normalized.as_str()) {
            &normalized
        } else if self.descriptions.contains_key(trimmed.as_str()) {
            &trimmed
        } else {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        };

        // For hierarchy lookups, always use normalized (non-dotted) format
        let hierarchy_key = self.normalize_code(found_key);

        let mut descendants = Vec::new();
        let mut to_visit = vec![hierarchy_key];
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
                        // Normalize child for hierarchy lookup
                        let child_normalized = self.normalize_code(child);
                        to_visit.push(child_normalized);
                    }
                }
            }
        }

        Ok(descendants)
    }

    /// Get the immediate parent of a code.
    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        let trimmed = code.trim().to_uppercase();

        // Validate the code exists first (check both formats)
        let found_key = if self.descriptions.contains_key(normalized.as_str()) {
            &normalized
        } else if self.descriptions.contains_key(trimmed.as_str()) {
            &trimmed
        } else {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        };

        // For hierarchy lookups, always use normalized (non-dotted) format
        let hierarchy_key = self.normalize_code(found_key);

        if let Some(Some(parent)) = self.parents.get(hierarchy_key.as_str())
            && let Some(dotted_parent) = self.dotted_form_for_normalized(parent)
            && let Some(description) = self.descriptions.get(dotted_parent)
        {
            return Ok(Some(Code {
                system: System::Icd10Cm,
                code: dotted_parent.to_string(),
                description: description.to_string(),
            }));
        }

        Ok(None)
    }

    /// Get all immediate children of a code.
    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize_code(code);
        let trimmed = code.trim().to_uppercase();

        // Validate the code exists first (check both formats)
        let found_key = if self.descriptions.contains_key(normalized.as_str()) {
            &normalized
        } else if self.descriptions.contains_key(trimmed.as_str()) {
            &trimmed
        } else {
            return Err(MedCodeError::not_found(code, System::Icd10Cm));
        };

        // For hierarchy lookups, always use normalized (non-dotted) format
        let hierarchy_key = self.normalize_code(found_key);

        self.children.get(hierarchy_key.as_str()).map_or_else(
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
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_normalize() {
        let icd10 = Icd10Cm::new();
        assert_eq!(icd10.normalize_code("I10.9"), "I109");
        assert_eq!(icd10.normalize_code("i10.9"), "I109");
        assert_eq!(icd10.normalize_code("A01"), "A01");
        assert_eq!(icd10.normalize_code("  I10.9  "), "I109"); // Test whitespace trimming
        assert_eq!(icd10.normalize_code("\tI10.9\n"), "I109"); // Test tab/newline trimming
    }

    #[test]
    fn test_is_valid() {
        let icd10 = Icd10Cm::new();
        // These will be false until we populate the data
        assert!(!icd10.is_valid("I10.9"));
        assert!(!icd10.is_valid("INVALID"));
        assert!(!icd10.is_valid("  I10.9  ")); // Test with whitespace
    }

    #[test]
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
