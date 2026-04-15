//! Optional medical code normalization using the `medcodes` crate.
//!
//! This module is only available when the `medcodes` feature is enabled.
//! It provides ICD code normalization during ETL parsing, and auto-detects
//! whether to use ICD-9-CM or ICD-10-CM based on the MIMIC version.

use crate::types::MimicVersion;
use medcodes::CodeSystem;

/// A code normalizer that delegates to the appropriate ICD code system.
pub enum CodeNormalizer {
    /// ICD-9-CM normalizer for MIMIC-III
    Icd9(medcodes::icd9::Icd9Cm),
    /// ICD-10-CM normalizer for MIMIC-IV
    Icd10(medcodes::icd10::Icd10Cm),
}

impl CodeNormalizer {
    /// Create a normalizer for the given MIMIC version.
    ///
    /// - `MimicVersion::MimicIII` → ICD-9-CM
    /// - `MimicVersion::MimicIV` → ICD-10-CM
    #[must_use]
    pub fn for_version(version: MimicVersion) -> Self {
        match version {
            MimicVersion::MimicIII => Self::Icd9(medcodes::icd9::Icd9Cm::new()),
            MimicVersion::MimicIV => Self::Icd10(medcodes::icd10::Icd10Cm::new()),
        }
    }

    /// Normalize an ICD code string (trim, uppercase, remove dots).
    #[must_use]
    pub fn normalize(&self, code: &str) -> String {
        match self {
            Self::Icd9(icd9) => icd9.normalize(code),
            Self::Icd10(icd10) => icd10.normalize(code),
        }
    }

    /// Check if the code is valid in the underlying code system.
    #[must_use]
    pub fn is_valid(&self, code: &str) -> bool {
        match self {
            Self::Icd9(icd9) => icd9.is_valid(code),
            Self::Icd10(icd10) => icd10.is_valid(code),
        }
    }

    /// Return a human-readable label for the active code system.
    #[must_use]
    pub const fn system_name(&self) -> &'static str {
        match self {
            Self::Icd9(_) => "ICD-9-CM",
            Self::Icd10(_) => "ICD-10-CM",
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_normalizer_for_mimic_iii() {
        let norm = CodeNormalizer::for_version(MimicVersion::MimicIII);
        assert_eq!(norm.system_name(), "ICD-9-CM");
        // ICD-9-CM normalize trims and uppercases
        let result = norm.normalize("  001.0  ");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_normalizer_for_mimic_iv() {
        let norm = CodeNormalizer::for_version(MimicVersion::MimicIV);
        assert_eq!(norm.system_name(), "ICD-10-CM");
        let result = norm.normalize("  a00.0  ");
        assert!(result.contains("A00"));
    }

    #[test]
    fn test_auto_detect_by_version() {
        let iii = CodeNormalizer::for_version(MimicVersion::MimicIII);
        let iv = CodeNormalizer::for_version(MimicVersion::MimicIV);
        assert_eq!(iii.system_name(), "ICD-9-CM");
        assert_eq!(iv.system_name(), "ICD-10-CM");
    }
}
