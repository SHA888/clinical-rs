//! ICD-9-CM (International Classification of Diseases, 9th Revision, Clinical Modification)
//!
//! This module provides ICD-9-CM code lookup, validation, and hierarchy traversal.
//! ICD-9-CM was frozen in October 2015 and is primarily used for historical data.

use crate::{Code, CodeSystem, MedCodeError, System};
use phf::phf_map;

/// ICD-9-CM code system implementation.
#[derive(Debug)]
pub struct Icd9Cm {
    /// Map of normalized codes to descriptions
    descriptions: &'static phf::Map<&'static str, &'static str>,
    /// Map of codes to their parent codes
    parents: &'static phf::Map<&'static str, Option<&'static str>>,
    /// Map of codes to their children codes
    children: &'static phf::Map<&'static str, &'static [&'static str]>,
}

impl Default for Icd9Cm {
    fn default() -> Self {
        Self::new()
    }
}

impl Icd9Cm {
    /// Create a new `Icd9Cm` instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::{icd9::Icd9Cm, CodeSystem};
    ///
    /// let icd9 = Icd9Cm::new();
    /// assert!(icd9.is_valid("001.0"));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptions: &ICD9_CM_DESCRIPTIONS,
            parents: &ICD9_CM_PARENTS,
            children: &ICD9_CM_CHILDREN,
        }
    }

    /// Normalize an ICD-9-CM code.
    ///
    /// This function:
    /// - Removes all whitespace (leading, trailing, and internal)
    /// - Preserves dots for 3-digit categories (e.g., "001" stays "001")
    /// - Preserves dots for subcategories (e.g., "001.0" stays "001.0")
    /// - Converts to uppercase
    ///
    /// Examples:
    /// - "001.0" -> "001.0"
    /// - " 001 .0 " -> "001.0"
    /// - "0010" -> "001.0" (adds dot for valid subcategories)
    /// - "001" -> "001" (no dot for 3-digit categories)
    #[must_use]
    pub fn normalize(&self, code: &str) -> String {
        let mut normalized = code
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase();

        // Add dot for 4-digit codes if it's a valid subcategory format
        if normalized.len() == 4
            && !normalized.contains('.')
            && let Some(three_digits) = normalized.get(0..3)
            && let Some(last_digit) = normalized.get(3..4)
        {
            normalized = format!("{three_digits}.{last_digit}");
        }

        normalized
    }
}

impl CodeSystem for Icd9Cm {
    fn lookup(&self, code: &str) -> Result<Code, MedCodeError> {
        let normalized = self.normalize(code);

        if let Some(&description) = self.descriptions.get(normalized.as_str()) {
            Ok(Code::new(System::Icd9Cm, normalized, description))
        } else {
            Err(MedCodeError::not_found(code, System::Icd9Cm))
        }
    }

    fn is_valid(&self, code: &str) -> bool {
        let normalized = self.normalize(code);
        self.descriptions.contains_key(normalized.as_str())
    }

    fn normalize(&self, code: &str) -> String {
        // Call the inherent method, not the trait method to avoid recursion
        Self::normalize(self, code)
    }

    fn ancestors(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);
        let mut ancestors = Vec::new();
        let mut current = Some(normalized.as_str());

        while let Some(current_code) = current {
            if let Some(&Some(parent)) = self.parents.get(current_code) {
                if let Some(&description) = self.descriptions.get(parent) {
                    ancestors.push(Code::new(System::Icd9Cm, parent.to_string(), description));
                    current = Some(parent);
                } else {
                    return Err(MedCodeError::data(format!(
                        "Parent {parent} exists but has no description"
                    )));
                }
            } else {
                break;
            }
        }

        Ok(ancestors)
    }

    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);
        let mut descendants = Vec::new();
        let mut to_visit = vec![normalized.as_str()];

        while let Some(current) = to_visit.pop() {
            if let Some(children) = self.children.get(current) {
                for &child in *children {
                    if let Some(&description) = self.descriptions.get(child) {
                        descendants.push(Code::new(System::Icd9Cm, child, description));
                        to_visit.push(child);
                    }
                }
            }
        }

        Ok(descendants)
    }

    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError> {
        let normalized = self.normalize(code);

        if let Some(&Some(parent)) = self.parents.get(normalized.as_str()) {
            if let Some(&description) = self.descriptions.get(parent) {
                Ok(Some(Code::new(
                    System::Icd9Cm,
                    parent.to_string(),
                    description,
                )))
            } else {
                Err(MedCodeError::data(format!(
                    "Parent {parent} exists but has no description"
                )))
            }
        } else {
            Ok(None)
        }
    }

    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);
        let mut result = Vec::new();

        if let Some(children) = self.children.get(normalized.as_str()) {
            for &child in *children {
                if let Some(&description) = self.descriptions.get(child) {
                    result.push(Code::new(System::Icd9Cm, child, description));
                }
            }
        }

        Ok(result)
    }
}

// Include generated data from build.rs
include!(concat!(env!("OUT_DIR"), "/icd9cm_data.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icd9cm_new() {
        let icd9 = Icd9Cm::new();
        // Test with a code from our sample data
        assert!(icd9.is_valid("001.0"));
    }

    #[test]
    fn test_normalize() {
        let icd9 = Icd9Cm::new();

        // Basic normalization
        assert_eq!(icd9.normalize("001.0"), "001.0");
        assert_eq!(icd9.normalize(" 001 .0 "), "001.0");
        assert_eq!(icd9.normalize("0010"), "001.0");
        assert_eq!(icd9.normalize("001"), "001");

        // Case conversion
        assert_eq!(icd9.normalize("cholera"), "CHOLERA");
    }

    #[test]
    fn test_normalize_idempotent() {
        let icd9 = Icd9Cm::new();
        let code = "001.0";
        let normalized_once = icd9.normalize(code);
        let normalized_twice = icd9.normalize(&normalized_once);
        assert_eq!(normalized_once, normalized_twice);
    }

    #[test]
    fn test_normalize_uppercase() {
        let icd9 = Icd9Cm::new();
        assert_eq!(icd9.normalize("cholera"), "CHOLERA");
        assert_eq!(icd9.normalize("Cholera"), "CHOLERA");
    }

    #[test]
    fn test_normalize_no_dots() {
        let icd9 = Icd9Cm::new();
        // 3-digit codes should not have dots added
        assert_eq!(icd9.normalize("001"), "001");
        assert_eq!(icd9.normalize("V01"), "V01");
        // 4-digit codes get dot added if they look like subcategories
        assert_eq!(icd9.normalize("E800"), "E80.0");
        assert_eq!(icd9.normalize("E80.0"), "E80.0");
        // Note: E800->E80.0 conversion assumes E800 is a subcategory of E80
    }
}
