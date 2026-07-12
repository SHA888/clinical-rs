//! LOINC (Logical Observation Identifiers Names and Codes) terminology support.
//!
//! This module provides lookup and component extraction for LOINC codes, the standard code system
//! for identifying laboratory and clinical observations. LOINC is maintained by the Regenstrief Institute.
//!
//! ## The LOINC 6-Axis Classification
//!
//! Each LOINC code represents a unique observation through six axes:
//!
//! - **Component**: What is being measured (e.g., "Creatinine", "Glucose")
//! - **Property**: The kind of quantity or characteristic being measured (e.g., "Mass concentration", "Presence/Absence")
//! - **Timing**: The aspect of time relevant to the observation (e.g., "Specific time", "Avg of multiple specimens")
//! - **System**: The specimen or system being measured (e.g., "Serum", "Urine", "Blood")
//! - **Scale**: The scale of measurement (e.g., "Quantitative", "Ordinal", "Nominal")
//! - **Method**: Optional method of measurement (e.g., "Enzymatic", "HPLC")
//!
//! Together, these six axes create a unique identifier for the observation. For example:
//! - **2160-0**: Creatinine (component) in Serum (system) as Mass concentration (property)
//!   - Component: Creatinine
//!   - Property: Mass concentration
//!   - Timing: Specific time
//!   - System: Serum
//!   - Scale: Quantitative
//!   - Method: Not specified
//!
//! ## Data Source
//!
//! LOINC codes are compiled from the official [Regenstrief Institute LOINC distribution](https://loinc.org/).
//! This implementation uses LOINC version 2.82.
//!
//! ## Hierarchy
//!
//! Term hierarchy is derived from the Regenstrief `ComponentHierarchyBySystem`
//! (multiaxial hierarchy) file. The ancestors of a LOINC term are LOINC Part
//! numbers (e.g. `LP385359-7`), not other terms — the hierarchy groups
//! observations by Component, System, and Class rather than linking terms
//! directly to each other. [`Loinc`] implements the `parent`/`children`/
//! `ancestors`/`descendants` methods from [`CodeSystem`] to traverse this
//! structure from either a term or a Part code.
//!
//! ## Feature Flag
//!
//! This module is gated behind the `loinc` feature flag and must be enabled in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! medcodes = { version = "0.2", features = ["loinc"] }
//! ```
//!
//! ## Examples
//!
//! ### Basic code lookup
//!
//! ```ignore
//! use medcodes::loinc::Loinc;
//! use medcodes::CodeSystem;
//!
//! let loinc = Loinc::new();
//! let code = loinc.lookup("2160-0")?;  // Creatinine in Serum
//! assert!(code.description.contains("Creatinine"));
//! ```
//!
//! ### Extract component details
//!
//! ```ignore
//! use medcodes::loinc::Loinc;
//!
//! let loinc = Loinc::new();
//! let components = loinc.components("2160-0")?;
//! assert_eq!(components.component, "Creatinine");
//! assert_eq!(components.system, "Serum");
//! assert_eq!(components.property, "Mass concentration");
//! ```
//!
//! ### Common laboratory codes
//!
//! - **2160-0**: Creatinine (kidney function)
//! - **2345-7**: Glucose (diabetes screening)
//! - **3625-2**: Troponin T (cardiac marker)
//! - **2157-6**: Creatinine clearance (kidney function estimate)

use crate::CodeSystem;
use crate::types::{Code, MedCodeError, System};

/// Component details for a LOINC code.
///
/// Each field represents one axis of the LOINC 6-axis classification system.
/// Together, these axes uniquely identify a clinical observation.
///
/// # Example
///
/// For LOINC code 2160-0 (Creatinine in Serum):
/// ```text
/// LoincComponents {
///     component: "Creatinine",
///     property: "Mass concentration",
///     timing: "Specific time",
///     system: "Serum",
///     scale: "Quantitative",
///     method: None,
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoincComponents {
    /// **Component axis**: What is being measured.
    /// Examples: "Creatinine", "Glucose", "Hemoglobin", "Troponin T"
    pub component: &'static str,

    /// **Property axis**: The kind of quantity or characteristic being observed.
    /// Examples: "Mass concentration", "Presence/Absence", "Molarity", "Ratio"
    pub property: &'static str,

    /// **Timing axis**: The aspect of time relevant to the measurement.
    /// Examples: "Specific time", "Average", "Minimum", "Maximum", "Total"
    pub timing: &'static str,

    /// **System axis**: The specimen type or body system being measured.
    /// Examples: "Serum", "Plasma", "Whole blood", "Urine", "Cerebrospinal fluid"
    pub system: &'static str,

    /// **Scale axis**: The scale of measurement (scalar, ordinal, nominal, etc.).
    /// Examples: "Quantitative", "Ordinal", "Nominal", "Narrative", "Document"
    pub scale: &'static str,

    /// **Method axis**: Optional method of measurement or analysis.
    /// May be `None` if not specified.
    /// Examples: "Enzymatic", "HPLC", "Immunoassay", "Microscopy"
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

    /// Retrieve the 6-axis component breakdown for a given LOINC code.
    ///
    /// This method decomposes a LOINC code into its constituent axes, enabling analysis
    /// of what is being measured, how it's measured, and on what specimen type.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::loinc::Loinc;
    ///
    /// let loinc = Loinc::new();
    /// let components = loinc.components("2160-0")?;  // Creatinine
    /// println!("Measured: {}", components.component);
    /// println!("In system: {}", components.system);
    /// println!("Type: {}", components.property);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`MedCodeError::NotFound`] when the code does not exist in the LOINC database.
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

/// Resolve a display description for a LOINC code, whether it is a term
/// (from `LOINC_DESCRIPTIONS`) or a hierarchy Part/group node (from
/// `LOINC_PART_NAMES`).
fn loinc_display_name(code: &str) -> Option<&'static str> {
    LOINC_DESCRIPTIONS
        .get(code)
        .or_else(|| LOINC_PART_NAMES.get(code))
        .copied()
}

/// True if `code` is a known node in the LOINC hierarchy — either a term or a
/// Part/group node used as an intermediate hierarchy branch.
fn loinc_is_known_node(code: &str) -> bool {
    LOINC_DESCRIPTIONS.contains_key(code) || LOINC_PART_NAMES.contains_key(code)
}

impl CodeSystem for Loinc {
    /// Look up a LOINC term by its `LOINC_NUM`.
    ///
    /// Only term codes are recognized here — LOINC Part/group numbers (e.g.
    /// `LP385359-7`) used internally by the hierarchy are not "codes" in the
    /// `lookup`/`is_valid` sense, even though [`Self::parent`],
    /// [`Self::children`], [`Self::ancestors`], and [`Self::descendants`]
    /// accept them as traversal nodes.
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
        if !loinc_is_known_node(norm.as_str()) {
            return Err(MedCodeError::not_found(code, System::Loinc));
        }

        let mut ancestors = Vec::new();
        let mut current = norm.as_str();
        let mut visited = std::collections::HashSet::new();
        let _ = visited.insert(current);

        while let Some(Some(parent)) = LOINC_PARENTS.get(current) {
            if !visited.insert(parent) {
                // Defensive guard against a parent-chain cycle: the canonical
                // parent map is built from vendored LOINC data via first-seen
                // tie-breaking over a true polyhierarchy, so cycle-freedom is
                // an empirical property of that data, not a type-level
                // guarantee (unlike ICD-10-CM's strictly-shortening codes).
                break;
            }
            // A missing display name only excludes this node from the result
            // — it must not stop the climb, or every ancestor above it would
            // be silently dropped too.
            if let Some(desc) = loinc_display_name(parent) {
                ancestors.push(Code {
                    system: System::Loinc,
                    code: parent.to_string(),
                    description: desc.to_string(),
                });
            }
            current = parent;
        }

        Ok(ancestors)
    }

    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let norm = self.normalize(code);
        if !loinc_is_known_node(norm.as_str()) {
            return Err(MedCodeError::not_found(code, System::Loinc));
        }

        let mut descendants = Vec::new();
        let mut stack = vec![norm.as_str()];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(children) = LOINC_CHILDREN.get(current) {
                for &child in children.iter() {
                    // A missing display name only excludes this node from the
                    // result — it must not stop the walk, or the entire
                    // subtree below it would be silently dropped too.
                    if let Some(desc) = loinc_display_name(child) {
                        descendants.push(Code {
                            system: System::Loinc,
                            code: child.to_string(),
                            description: desc.to_string(),
                        });
                    }
                    stack.push(child);
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
        if !loinc_is_known_node(norm.as_str()) {
            return Err(MedCodeError::not_found(code, System::Loinc));
        }

        if let Some(Some(parent)) = LOINC_PARENTS.get(norm.as_str()) {
            if let Some(desc) = loinc_display_name(parent) {
                return Ok(Some(Code {
                    system: System::Loinc,
                    code: parent.to_string(),
                    description: desc.to_string(),
                }));
            }
        }

        Ok(None)
    }

    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let norm = self.normalize(code);
        if !loinc_is_known_node(norm.as_str()) {
            return Err(MedCodeError::not_found(code, System::Loinc));
        }

        if let Some(children) = LOINC_CHILDREN.get(norm.as_str()) {
            let mut result = Vec::new();
            for &child in children.iter() {
                if let Some(desc) = loinc_display_name(child) {
                    result.push(Code {
                        system: System::Loinc,
                        code: child.to_string(),
                        description: desc.to_string(),
                    });
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
}
