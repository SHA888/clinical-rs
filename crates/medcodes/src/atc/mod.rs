//! ATC (Anatomical Therapeutic Chemical Classification System)
//!
//! This module provides ATC code lookup, validation, and hierarchy traversal.
//! ATC is maintained by the WHO Collaborating Centre for Drug Statistics Methodology.
//!
//! # ATC Hierarchy Levels
//!
//! 1. **Anatomical main group** (1st level): 1 letter (A-Z)
//!    - Example: "C" = Cardiovascular system
//! 2. **Therapeutic subgroup** (2nd level): 2 digits
//!    - Example: "C10" = Lipid modifying agents
//! 3. **Pharmacological subgroup** (3rd level): 1 letter
//!    - Example: "C10A" = Lipid modifying agents, plain
//! 4. **Chemical subgroup** (4th level): 1 letter
//!    - Example: "C10AA" = HMG `CoA` reductase inhibitors (statins)
//! 5. **Chemical substance** (5th level): 2 digits
//!    - Example: "C10AA01" = Simvastatin
//!
//! # DDD (Defined Daily Dose)
//!
//! DDD is the assumed average maintenance dose per day for a drug used for its main indication
//! in adults. DDD values are optional metadata associated with ATC codes.

use crate::{Code, CodeSystem, MedCodeError, System};
use phf::phf_map;

/// ATC code system implementation with 5-level hierarchy.
#[derive(Debug)]
pub struct Atc {
    /// Map of normalized codes to descriptions
    descriptions: &'static phf::Map<&'static str, &'static str>,
    /// Map of codes to their parent codes
    parents: &'static phf::Map<&'static str, Option<&'static str>>,
    /// Map of codes to their children codes
    children: &'static phf::Map<&'static str, &'static [&'static str]>,
    /// Map of codes to DDD values (optional)
    ddd_values: &'static phf::Map<&'static str, &'static str>,
}

impl Default for Atc {
    fn default() -> Self {
        Self::new()
    }
}

impl Atc {
    /// Create a new `Atc` instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::{atc::Atc, CodeSystem};
    ///
    /// let atc = Atc::new();
    /// assert!(atc.is_valid("C10AA01")); // Simvastatin
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptions: &ATC_DESCRIPTIONS,
            parents: &ATC_PARENTS,
            children: &ATC_CHILDREN,
            ddd_values: &ATC_DDD_VALUES,
        }
    }

    /// Normalize an ATC code.
    ///
    /// This function:
    /// - Removes all whitespace
    /// - Converts to uppercase
    /// - Validates format (1 letter + 0-6 alphanumeric characters)
    ///
    /// Examples:
    /// - "C10AA01" -> "C10AA01"
    /// - " c10aa01 " -> "C10AA01"
    /// - "c10aa01" -> "C10AA01"
    #[must_use]
    pub fn normalize(&self, code: &str) -> String {
        code.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase()
    }

    /// Get the DDD (Defined Daily Dose) for a code if available.
    ///
    /// DDD is the assumed average maintenance dose per day for a drug
    /// used for its main indication in adults.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::atc::Atc;
    ///
    /// let atc = Atc::new();
    /// if let Some(ddd) = atc.get_defined_daily_dose("C10AA01") {
    ///     println!("DDD for Simvastatin: {}", ddd);
    /// }
    /// ```
    #[must_use]
    pub fn get_defined_daily_dose(&self, code: &str) -> Option<&str> {
        let normalized = self.normalize(code);
        self.ddd_values.get(normalized.as_str()).copied()
    }

    /// Get the DDD (Defined Daily Dose) for a code if available.
    ///
    /// This is an alias for `get_defined_daily_dose` for convenience.
    #[must_use]
    pub fn ddd(&self, code: &str) -> Option<&str> {
        self.get_defined_daily_dose(code)
    }

    /// Get the hierarchy level of an ATC code.
    ///
    /// # Hierarchy Levels
    ///
    /// - Level 1: Anatomical main group (1 character)
    /// - Level 2: Therapeutic subgroup (3 characters)
    /// - Level 3: Pharmacological subgroup (4 characters)
    /// - Level 4: Chemical subgroup (5 characters)
    /// - Level 5: Chemical substance (7 characters)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::atc::{Atc, AtcLevel};
    ///
    /// let atc = Atc::new();
    /// assert_eq!(atc.level("C"), Some(AtcLevel::Anatomical));
    /// assert_eq!(atc.level("C10"), Some(AtcLevel::Therapeutic));
    /// assert_eq!(atc.level("C10AA01"), Some(AtcLevel::ChemicalSubstance));
    /// ```
    #[must_use]
    pub fn level(&self, code: &str) -> Option<AtcLevel> {
        let normalized = self.normalize(code);

        match normalized.len() {
            1 => Some(AtcLevel::Anatomical),
            3 => Some(AtcLevel::Therapeutic),
            4 => Some(AtcLevel::Pharmacological),
            5 => Some(AtcLevel::ChemicalSubgroup),
            7 => Some(AtcLevel::ChemicalSubstance),
            _ => None,
        }
    }

    /// Check if a code is a valid ATC code format.
    ///
    /// Valid ATC codes follow these patterns:
    /// - Level 1: 1 uppercase letter (A-Z)
    /// - Level 2: 1 uppercase letter + 2 digits
    /// - Level 3: Level 2 + 1 uppercase letter
    /// - Level 4: Level 3 + 1 uppercase letter
    /// - Level 5: Level 4 + 2 digits
    fn is_valid_format(&self, code: &str) -> bool {
        let normalized = self.normalize(code);
        let bytes = normalized.as_bytes();

        if bytes.is_empty() || bytes.len() > 7 {
            return false;
        }

        // First character must be an uppercase letter
        if !bytes[0].is_ascii_uppercase() {
            return false;
        }

        // Check based on length
        match bytes.len() {
            1 => true, // Just the anatomical group (e.g., "C")
            3 => {
                // Anatomical + 2 digits (e.g., "C10")
                bytes[1].is_ascii_digit() && bytes[2].is_ascii_digit()
            }
            4 => {
                // + 1 letter (e.g., "C10A")
                bytes[1].is_ascii_digit()
                    && bytes[2].is_ascii_digit()
                    && bytes[3].is_ascii_uppercase()
            }
            5 => {
                // + 1 letter (e.g., "C10AA")
                bytes[1].is_ascii_digit()
                    && bytes[2].is_ascii_digit()
                    && bytes[3].is_ascii_uppercase()
                    && bytes[4].is_ascii_uppercase()
            }
            7 => {
                // + 2 digits (e.g., "C10AA01")
                bytes[1].is_ascii_digit()
                    && bytes[2].is_ascii_digit()
                    && bytes[3].is_ascii_uppercase()
                    && bytes[4].is_ascii_uppercase()
                    && bytes[5].is_ascii_digit()
                    && bytes[6].is_ascii_digit()
            }
            _ => false, // Invalid length (2, 6)
        }
    }
}

impl CodeSystem for Atc {
    fn lookup(&self, code: &str) -> Result<Code, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Atc));
        }

        if let Some(&description) = self.descriptions.get(normalized.as_str()) {
            Ok(Code::new(System::Atc, normalized, description))
        } else {
            Err(MedCodeError::not_found(code, System::Atc))
        }
    }

    fn is_valid(&self, code: &str) -> bool {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return false;
        }

        self.descriptions.contains_key(normalized.as_str())
    }

    fn normalize(&self, code: &str) -> String {
        // Call the inherent method, not the trait method to avoid recursion
        Self::normalize(self, code)
    }

    fn ancestors(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Atc));
        }

        let mut ancestors = Vec::new();
        let mut current = Some(normalized.as_str());

        while let Some(current_code) = current {
            if let Some(&Some(parent)) = self.parents.get(current_code) {
                if let Some(&description) = self.descriptions.get(parent) {
                    ancestors.push(Code::new(System::Atc, parent.to_string(), description));
                    current = Some(parent);
                } else {
                    return Err(MedCodeError::data(format!(
                        "Parent {} exists but has no description",
                        parent
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

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Atc));
        }

        let mut descendants = Vec::new();
        let mut to_visit = vec![normalized.as_str()];

        while let Some(current) = to_visit.pop() {
            if let Some(children) = self.children.get(current) {
                for &child in *children {
                    if let Some(&description) = self.descriptions.get(child) {
                        descendants.push(Code::new(System::Atc, child.to_string(), description));
                        to_visit.push(child);
                    } else {
                        return Err(MedCodeError::data(format!(
                            "Child {} exists but has no description",
                            child
                        )));
                    }
                }
            }
        }

        Ok(descendants)
    }

    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Atc));
        }

        if let Some(&Some(parent)) = self.parents.get(normalized.as_str()) {
            if let Some(&description) = self.descriptions.get(parent) {
                Ok(Some(Code::new(
                    System::Atc,
                    parent.to_string(),
                    description,
                )))
            } else {
                Err(MedCodeError::data(format!(
                    "Parent {} exists but has no description",
                    parent
                )))
            }
        } else {
            Ok(None)
        }
    }

    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Atc));
        }

        let mut result = Vec::new();

        if let Some(children) = self.children.get(normalized.as_str()) {
            for &child in *children {
                if let Some(&description) = self.descriptions.get(child) {
                    result.push(Code::new(System::Atc, child.to_string(), description));
                } else {
                    return Err(MedCodeError::data(format!(
                        "Child {} exists but has no description",
                        child
                    )));
                }
            }
        }

        Ok(result)
    }
}

/// ATC hierarchy levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtcLevel {
    /// Level 1: Anatomical main group (1 character)
    Anatomical,
    /// Level 2: Therapeutic subgroup (3 characters)
    Therapeutic,
    /// Level 3: Pharmacological subgroup (4 characters)
    Pharmacological,
    /// Level 4: Chemical subgroup (5 characters)
    ChemicalSubgroup,
    /// Level 5: Chemical substance (7 characters)
    ChemicalSubstance,
}

impl AtcLevel {
    /// Get the display name for this level.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Anatomical => "Anatomical main group",
            Self::Therapeutic => "Therapeutic subgroup",
            Self::Pharmacological => "Pharmacological subgroup",
            Self::ChemicalSubgroup => "Chemical subgroup",
            Self::ChemicalSubstance => "Chemical substance",
        }
    }

    /// Get the expected code length for this level.
    #[must_use]
    pub const fn code_length(&self) -> usize {
        match self {
            Self::Anatomical => 1,
            Self::Therapeutic => 3,
            Self::Pharmacological => 4,
            Self::ChemicalSubgroup => 5,
            Self::ChemicalSubstance => 7,
        }
    }
}

impl std::fmt::Display for AtcLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// Include generated data from build.rs
include!(concat!(env!("OUT_DIR"), "/atc_data.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atc_new() {
        let atc = Atc::new();
        assert!(atc.is_valid("C10AA01"));
    }

    #[test]
    fn test_normalize() {
        let atc = Atc::new();

        // Basic normalization
        assert_eq!(atc.normalize("C10AA01"), "C10AA01");
        assert_eq!(atc.normalize(" c10aa01 "), "C10AA01");
        assert_eq!(atc.normalize("c10aa01"), "C10AA01");
    }

    #[test]
    fn test_normalize_idempotent() {
        let atc = Atc::new();
        let code = "C10AA01";
        let normalized_once = atc.normalize(code);
        let normalized_twice = atc.normalize(&normalized_once);
        assert_eq!(normalized_once, normalized_twice);
    }

    #[test]
    fn test_level_detection() {
        let atc = Atc::new();

        assert_eq!(atc.level("C"), Some(AtcLevel::Anatomical));
        assert_eq!(atc.level("C10"), Some(AtcLevel::Therapeutic));
        assert_eq!(atc.level("C10A"), Some(AtcLevel::Pharmacological));
        assert_eq!(atc.level("C10AA"), Some(AtcLevel::ChemicalSubgroup));
        assert_eq!(atc.level("C10AA01"), Some(AtcLevel::ChemicalSubstance));

        // Invalid lengths
        assert_eq!(atc.level("C1"), None);
        assert_eq!(atc.level("C10AA0"), None);
        assert_eq!(atc.level("C10AA012"), None);
    }

    #[test]
    fn test_valid_format() {
        let atc = Atc::new();

        // Valid formats
        assert!(atc.is_valid_format("A"));
        assert!(atc.is_valid_format("C10"));
        assert!(atc.is_valid_format("C10A"));
        assert!(atc.is_valid_format("C10AA"));
        assert!(atc.is_valid_format("C10AA01"));

        // Invalid formats
        assert!(!atc.is_valid_format("1")); // Must start with letter
        assert!(!atc.is_valid_format("C1")); // Invalid length
        assert!(!atc.is_valid_format("C10AA0")); // Invalid length
        assert!(!atc.is_valid_format("")); // Empty
        assert!(!atc.is_valid_format("C10AA012")); // Too long
        assert!(!atc.is_valid_format("C10A01")); // Missing letter at pos 4
    }

    #[test]
    fn test_atc_level_display() {
        assert_eq!(AtcLevel::Anatomical.name(), "Anatomical main group");
        assert_eq!(AtcLevel::Therapeutic.name(), "Therapeutic subgroup");
        assert_eq!(AtcLevel::Pharmacological.name(), "Pharmacological subgroup");
        assert_eq!(AtcLevel::ChemicalSubgroup.name(), "Chemical subgroup");
        assert_eq!(AtcLevel::ChemicalSubstance.name(), "Chemical substance");
    }

    #[test]
    fn test_atc_level_code_length() {
        assert_eq!(AtcLevel::Anatomical.code_length(), 1);
        assert_eq!(AtcLevel::Therapeutic.code_length(), 3);
        assert_eq!(AtcLevel::Pharmacological.code_length(), 4);
        assert_eq!(AtcLevel::ChemicalSubgroup.code_length(), 5);
        assert_eq!(AtcLevel::ChemicalSubstance.code_length(), 7);
    }
}
