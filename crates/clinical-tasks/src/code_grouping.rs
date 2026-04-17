//! Code grouping for feature extraction using `medcodes`.
//!
//! This module provides ICD code grouping functionality using CCSR (Clinical Classifications
//! Software Refined) categories. When the `medcodes` feature is enabled, diagnosis codes
//! can be grouped into clinically meaningful categories for improved feature engineering.
//!
//! # Example
//!
//! ```rust,ignore
//! use clinical_tasks::code_grouping::CodeGrouper;
//!
//! let grouper = CodeGrouper::new();
//! let category = grouper.group_icd10("I21.0"); // Acute myocardial infarction
//! assert_eq!(category, Some("Circulatory".to_string()));
//! ```

use std::collections::HashMap;

/// Grouper for ICD codes into clinical categories.
#[derive(Debug, Clone, Default)]
pub struct CodeGrouper {
    // Reserved for future detailed code-to-category mappings
    _private: (),
}

impl CodeGrouper {
    /// Create a new code grouper with built-in mappings.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Group an ICD-10-CM code into a clinical category.
    ///
    /// Uses prefix matching to determine the body system category.
    /// Returns `None` if the code cannot be categorized.
    #[must_use]
    pub fn group_icd10(&self, code: &str) -> Option<String> {
        let code_upper = code.to_uppercase();
        let prefix = code_upper.chars().next()?;

        // First letter determines body system (simplified mapping)
        let category = match prefix {
            'A' | 'B' => "Infectious",
            'C' | 'D' => {
                // D codes: D0-D4 = neoplasms, D5-D9 = blood/immune
                if code_upper.len() >= 2 {
                    let second = code_upper.chars().nth(1)?;
                    if second == '0'
                        || second == '1'
                        || second == '2'
                        || second == '3'
                        || second == '4'
                    {
                        "Neoplasms"
                    } else {
                        "Blood"
                    }
                } else {
                    "Neoplasms"
                }
            }
            'E' => "Endocrine",
            'F' => "Mental",
            'G' => "Nervous",
            'H' => {
                // H0-H5 = eye, H6-H9 = ear
                if code_upper.len() >= 2 {
                    let second = code_upper.chars().nth(1)?;
                    if second == '0'
                        || second == '1'
                        || second == '2'
                        || second == '3'
                        || second == '4'
                        || second == '5'
                    {
                        "Eye"
                    } else {
                        "Ear"
                    }
                } else {
                    "Eye"
                }
            }
            'I' => "Circulatory",
            'J' => "Respiratory",
            'K' => "Digestive",
            'L' => "Skin",
            'M' => "Musculoskeletal",
            'N' => "Genitourinary",
            'O' => "Pregnancy",
            'P' => "Perinatal",
            'Q' => "Congenital",
            'R' => "Symptoms",
            'S' | 'T' => "Injury",
            'V' | 'W' | 'X' | 'Y' => "External",
            'Z' => "Factors",
            _ => return None,
        };

        Some(category.to_string())
    }

    /// Group an ICD-9-CM code into a clinical category.
    ///
    /// Uses numeric range matching to determine the body system category.
    /// Returns `None` if the code cannot be categorized.
    #[must_use]
    pub fn group_icd9(&self, code: &str) -> Option<String> {
        // Parse numeric portion
        let numeric_part: String = code.chars().take_while(char::is_ascii_digit).collect();
        let category_num: u32 = numeric_part.parse().ok()?;

        let category = match category_num {
            0..=139 => "Infectious",
            140..=239 => "Neoplasms",
            240..=279 => "Endocrine",
            280..=289 => "Blood",
            290..=319 => "Mental",
            320..=389 => "Nervous",
            390..=459 => "Circulatory",
            460..=519 => "Respiratory",
            520..=579 => "Digestive",
            580..=629 => "Genitourinary",
            630..=679 => "Pregnancy",
            680..=709 => "Skin",
            710..=739 => "Musculoskeletal",
            740..=759 => "Congenital",
            760..=779 => "Perinatal",
            780..=799 => "Symptoms",
            800..=999 => "Injury",
            _ => return None,
        };

        Some(category.to_string())
    }

    /// Get all available category names.
    #[must_use]
    pub fn categories(&self) -> Vec<String> {
        vec![
            "Circulatory".to_string(),
            "Respiratory".to_string(),
            "Digestive".to_string(),
            "Nervous".to_string(),
            "Musculoskeletal".to_string(),
            "Genitourinary".to_string(),
            "Skin".to_string(),
            "Endocrine".to_string(),
            "Blood".to_string(),
            "Infectious".to_string(),
            "Mental".to_string(),
            "Eye".to_string(),
            "Ear".to_string(),
            "Pregnancy".to_string(),
            "Perinatal".to_string(),
            "Congenital".to_string(),
            "Neoplasms".to_string(),
            "Symptoms".to_string(),
            "Injury".to_string(),
            "External".to_string(),
        ]
    }

    /// Count codes by category from a list of ICD codes.
    ///
    /// Returns a map of category name to count.
    #[must_use]
    pub fn count_by_category(&self, codes: &[String], version: IcdVersion) -> HashMap<String, u32> {
        let mut counts: HashMap<String, u32> = HashMap::new();

        for code in codes {
            let category = match version {
                IcdVersion::Icd9 => self.group_icd9(code),
                IcdVersion::Icd10 => self.group_icd10(code),
            };

            if let Some(cat) = category {
                *counts.entry(cat).or_insert(0) += 1;
            }
        }

        counts
    }
}

/// ICD version for code grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcdVersion {
    /// ICD-9-CM (used in MIMIC-III)
    Icd9,
    /// ICD-10-CM (used in MIMIC-IV)
    Icd10,
}

/// Feature extractor with code grouping support.
///
/// This trait extends the base feature extraction with ICD code grouping
/// when the `medcodes` feature is enabled.
pub trait GroupedFeatureExtractor {
    /// Extract grouped diagnosis counts by body system.
    ///
    /// Returns a map of category name to count.
    fn extract_grouped_diagnoses(
        &self,
        diagnosis_codes: &[String],
        version: IcdVersion,
    ) -> HashMap<String, f64>;
}

impl GroupedFeatureExtractor for CodeGrouper {
    fn extract_grouped_diagnoses(
        &self,
        diagnosis_codes: &[String],
        version: IcdVersion,
    ) -> HashMap<String, f64> {
        let counts = self.count_by_category(diagnosis_codes, version);
        counts.into_iter().map(|(k, v)| (k, f64::from(v))).collect()
    }
}
