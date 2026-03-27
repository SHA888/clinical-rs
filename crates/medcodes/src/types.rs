//! Core types for medical code systems.

use std::fmt::{self, Display, Formatter};
use thiserror::Error;

/// Supported medical code systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// RxNorm
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
            System::Icd9Cm => write!(f, "ICD-9-CM"),
            System::Icd10Cm => write!(f, "ICD-10-CM"),
            System::Icd10Pcs => write!(f, "ICD-10-PCS"),
            System::Atc => write!(f, "ATC"),
            System::Ndc => write!(f, "NDC"),
            System::Loinc => write!(f, "LOINC"),
            System::SnoMed => write!(f, "SNOMED CT"),
            System::RxNorm => write!(f, "RxNorm"),
            System::Ccs => write!(f, "CCS"),
            System::Ccsr => write!(f, "CCSR"),
            System::Cpt => write!(f, "CPT"),
        }
    }
}

/// A medical code with its system and description.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub fn new(system: System, code: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            system,
            code: code.into(),
            description: description.into(),
        }
    }

    /// Get the system this code belongs to.
    pub fn system(&self) -> System {
        self.system
    }

    /// Get the raw code value.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Get the description.
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.system, self.code, self.description)
    }
}

/// Trait for code system implementations.
pub trait CodeSystem {
    /// Look up a code by its value.
    fn lookup(&self, code: &str) -> Result<Code, Error>;

    /// Check if a code is valid in this system.
    fn is_valid(&self, code: &str) -> bool;

    /// Normalize a code (remove dots, uppercase, etc.).
    fn normalize(&self, code: &str) -> String;

    /// Get all ancestors of a code in the hierarchy.
    fn ancestors(&self, code: &str) -> Result<Vec<Code>, Error>;

    /// Get all descendants of a code in the hierarchy.
    fn descendants(&self, code: &str) -> Result<Vec<Code>, Error>;

    /// Get the immediate parent of a code.
    fn parent(&self, code: &str) -> Result<Option<Code>, Error>;

    /// Get all immediate children of a code.
    fn children(&self, code: &str) -> Result<Vec<Code>, Error>;
}

/// Trait for cross-system code mapping.
pub trait CrossMap {
    /// Map a code from one system to another.
    fn map(&self, code: &str, target_system: System) -> Result<Vec<Code>, Error>;

    /// Get the source system of this mapper.
    fn source_system(&self) -> System;

    /// Get the target system of this mapper.
    fn target_system(&self) -> System;
}

/// Errors that can occur when working with medical codes.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Code not found: {code} in {system}")]
    NotFound { code: String, system: System },

    #[error("Invalid code format: {code} for {system}")]
    InvalidFormat { code: String, system: System },

    #[error("No mapping found: {code} from {source_system} to {target_system}")]
    NoMapping {
        code: String,
        source_system: System,
        target_system: System,
    },

    #[error("Hierarchy error: {message}")]
    Hierarchy { message: String },

    #[error("Data error: {message}")]
    Data { message: String },
}

impl Error {
    pub fn not_found(code: impl Into<String>, system: System) -> Self {
        Self::NotFound {
            code: code.into(),
            system,
        }
    }

    pub fn invalid_format(code: impl Into<String>, system: System) -> Self {
        Self::InvalidFormat {
            code: code.into(),
            system,
        }
    }

    pub fn no_mapping(code: impl Into<String>, source: System, target: System) -> Self {
        Self::NoMapping {
            code: code.into(),
            source_system: source,
            target_system: target,
        }
    }

    pub fn hierarchy(message: impl Into<String>) -> Self {
        Self::Hierarchy {
            message: message.into(),
        }
    }

    pub fn data(message: impl Into<String>) -> Self {
        Self::Data {
            message: message.into(),
        }
    }
}
