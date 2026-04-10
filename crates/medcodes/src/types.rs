//! Core types for medical code systems.

use std::fmt::{self, Display, Formatter};

/// Supported medical code systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum System {
    /// International Classification of Diseases, 9th Revision, Clinical Modification
    Icd9Cm,
    /// International Classification of Diseases, 10th Revision, Clinical Modification
    Icd10Cm,
    /// International Classification of Diseases, 10th Revision, Procedure Coding System
    Icd10Pcs,
    /// Anatomical Therapeutic Chemical Classification System
    Atc,
    /// National Drug Code
    Ndc,
    /// Logical Observation Identifiers Names and Codes
    Loinc,
    /// Systematized Nomenclature of Medicine Clinical Terms
    SnoMed,
    /// `RxNorm`
    RxNorm,
    /// Clinical Classifications Software
    Ccs,
    /// Clinical Classifications Software Refined
    Ccsr,
    /// Current Procedural Terminology
    Cpt,
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Icd9Cm => write!(f, "ICD-9-CM"),
            Self::Icd10Cm => write!(f, "ICD-10-CM"),
            Self::Icd10Pcs => write!(f, "ICD-10-PCS"),
            Self::Atc => write!(f, "ATC"),
            Self::Ndc => write!(f, "NDC"),
            Self::Loinc => write!(f, "LOINC"),
            Self::SnoMed => write!(f, "SNOMED CT"),
            Self::RxNorm => write!(f, "RxNorm"),
            Self::Ccs => write!(f, "CCS"),
            Self::Ccsr => write!(f, "CCSR"),
            Self::Cpt => write!(f, "CPT"),
        }
    }
}

/// A medical code with its system and description.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Code {
    /// The code system this code belongs to
    pub system: System,
    /// The raw code value (e.g., "I10.9", "A01", "123456")
    pub code: String,
    /// Human-readable description
    pub description: String,
}

impl Code {
    /// Create a new medical code.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::{Code, System};
    ///
    /// let code = Code::new(
    ///     System::Icd10Cm,
    ///     "A00.0",
    ///     "Cholera due to Vibrio cholerae 01, biovar cholerae"
    /// );
    /// assert_eq!(code.system, System::Icd10Cm);
    /// ```
    #[must_use]
    pub fn new(system: System, code: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            system,
            code: code.into(),
            description: description.into(),
        }
    }

    /// Get the system this code belongs to.
    #[must_use]
    pub const fn system(&self) -> System {
        self.system
    }

    /// Get the raw code value.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.system, self.code, self.description)
    }
}

impl From<(System, &'static str, &'static str)> for Code {
    fn from((system, code, description): (System, &'static str, &'static str)) -> Self {
        Self {
            system,
            code: code.to_string(),
            description: description.to_string(),
        }
    }
}

/// Trait for code system implementations.
pub trait CodeSystem {
    /// Look up a code by its value.
    ///
    /// # Errors
    ///
    /// Returns an error if the code is not found in this system.
    fn lookup(&self, code: &str) -> Result<Code, MedCodeError>;

    /// Check if a code is valid in this system.
    fn is_valid(&self, code: &str) -> bool;

    /// Normalize a code (remove formatting, uppercase, etc.).
    #[must_use]
    fn normalize(&self, code: &str) -> String;

    /// Get all ancestors of a code in the hierarchy.
    ///
    /// # Errors
    ///
    /// Returns an error if the code is not found or hierarchy traversal fails.
    fn ancestors(&self, code: &str) -> Result<Vec<Code>, MedCodeError>;

    /// Get all descendants of a code in the hierarchy.
    ///
    /// # Errors
    ///
    /// Returns an error if the code is not found or hierarchy traversal fails.
    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError>;

    /// Get the immediate parent of a code.
    ///
    /// # Errors
    ///
    /// Returns an error if the code is not found or hierarchy traversal fails.
    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError>;

    /// Get all immediate children of a code.
    ///
    /// # Errors
    ///
    /// Returns an error if the code is not found or hierarchy traversal fails.
    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError>;
}

/// Trait for cross-system code mapping.
pub trait CrossMap {
    /// Map a code from one system to another.
    ///
    /// # Errors
    ///
    /// Returns an error if the code cannot be mapped to the target system.
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, MedCodeError>;

    /// Get the source system of this mapper.
    fn source_system(&self) -> System;

    /// Get the target system of this mapper.
    fn target_system(&self) -> System;
}

/// Errors that can occur when working with medical codes.
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MedCodeError {
    /// Code not found in the specified system.
    #[error("Code not found: `{code}` in `{system}`")]
    NotFound {
        /// The code that was not found
        code: String,
        /// The system where the code was searched
        system: System,
    },

    /// Invalid code format for the specified system.
    #[error("Invalid code format: `{code}` for `{system}`")]
    InvalidFormat {
        /// The code with invalid format
        code: String,
        /// The system where the format is invalid
        system: System,
    },

    /// No mapping found between systems.
    #[error("No mapping found: `{code}` from `{source_system}` to `{target_system}`")]
    NoMapping {
        /// The code that could not be mapped
        code: String,
        /// The source system
        source_system: System,
        /// The target system
        target_system: System,
    },

    /// Hierarchy traversal error.
    #[error("Hierarchy error: `{message}`")]
    Hierarchy {
        /// Error message describing the hierarchy issue
        message: String,
    },

    /// Data processing error.
    #[error("Data error: `{message}`")]
    Data {
        /// Error message describing the data issue
        message: String,
    },
}

impl MedCodeError {
    /// Create a new `NotFound` error.
    pub fn not_found(code: impl Into<String>, system: System) -> Self {
        Self::NotFound {
            code: code.into(),
            system,
        }
    }

    /// Create a new `InvalidFormat` error.
    pub fn invalid_format(code: impl Into<String>, system: System) -> Self {
        Self::InvalidFormat {
            code: code.into(),
            system,
        }
    }

    /// Create a new `NoMapping` error.
    pub fn no_mapping(code: impl Into<String>, source: System, target: System) -> Self {
        Self::NoMapping {
            code: code.into(),
            source_system: source,
            target_system: target,
        }
    }

    /// Create a new Hierarchy error.
    pub fn hierarchy(message: impl Into<String>) -> Self {
        Self::Hierarchy {
            message: message.into(),
        }
    }

    /// Create a new Data error.
    pub fn data(message: impl Into<String>) -> Self {
        Self::Data {
            message: message.into(),
        }
    }
}
