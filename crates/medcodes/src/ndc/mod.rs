//! National Drug Code (NDC) implementation.
//!
//! The NDC is a universal product identifier used for human drugs in the United States.
//! It consists of three segments: labeler code, product code, and package code.
//!
//! # NDC Structure
//!
//! NDC codes have the format: `XXXXXXX-XXXX-XX` or `XXXXXXXX-XXXX-XX`
//! - **Labeler code**: 4-5 digits (identifies the manufacturer/labeler)
//! - **Product code**: 3-4 digits (identifies the specific product)
//! - **Package code**: 2 digits (identifies the package size/type)
//!
//! # Examples
//!
//! ```ignore
//! use medcodes::ndc::Ndc;
//!
//! let ndc = Ndc::new();
//! if let Some(drug) = ndc.lookup("1234-5678-90") {
//!     println!("Drug: {}", drug.description());
//! }
//! ```

use crate::{Code, CodeSystem, MedCodeError, System};

/// NDC (National Drug Code) system implementation.
pub struct Ndc {
    /// Map of NDC codes to descriptions.
    descriptions: &'static phf::Map<&'static str, &'static str>,
    /// Map of NDC codes to labeler information.
    labelers: &'static phf::Map<&'static str, Option<&'static str>>,
    /// Map of NDC codes to product information.
    products: &'static phf::Map<&'static str, Option<&'static str>>,
    /// Map of NDC codes to package information.
    packages: &'static phf::Map<&'static str, Option<&'static str>>,
}

impl Default for Ndc {
    fn default() -> Self {
        Self {
            descriptions: &NDC_DESCRIPTIONS,
            labelers: &NDC_LABELERS,
            products: &NDC_PRODUCTS,
            packages: &NDC_PACKAGES,
        }
    }
}

impl Ndc {
    /// Create a new NDC instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse an NDC code into its components.
    ///
    /// # Returns
    ///
    /// Returns a tuple of (labeler, product, package) if the format is valid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use medcodes::ndc::Ndc;
    ///
    /// let ndc = Ndc::new();
    /// if let Some((labeler, product, package)) = ndc.parse_components("1234-5678-90") {
    ///     println!("Labeler: {}, Product: {}, Package: {}", labeler, product, package);
    /// }
    /// ```
    #[must_use]
    pub fn parse_components(&self, code: &str) -> Option<(String, String, String)> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return None;
        }

        // Split by hyphens
        let parts: Vec<&str> = normalized.split('-').collect();
        if parts.len() == 3 {
            Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ))
        } else {
            None
        }
    }

    /// Get the labeler code from an NDC code.
    #[must_use]
    pub fn labeler(&self, code: &str) -> Option<String> {
        self.parse_components(code).map(|(labeler, _, _)| labeler)
    }

    /// Get the product code from an NDC code.
    #[must_use]
    pub fn product(&self, code: &str) -> Option<String> {
        self.parse_components(code).map(|(_, product, _)| product)
    }

    /// Get the package code from an NDC code.
    #[must_use]
    pub fn package(&self, code: &str) -> Option<String> {
        self.parse_components(code).map(|(_, _, package)| package)
    }

    /// Get the stored labeler code for an NDC code from the data.
    #[must_use]
    pub fn stored_labeler(&self, code: &str) -> Option<&str> {
        let normalized = self.normalize(code);
        self.labelers.get(normalized.as_str()).and_then(|opt| *opt)
    }

    /// Get the stored product code for an NDC code from the data.
    #[must_use]
    pub fn stored_product(&self, code: &str) -> Option<&str> {
        let normalized = self.normalize(code);
        self.products.get(normalized.as_str()).and_then(|opt| *opt)
    }

    /// Get the stored package code for an NDC code from the data.
    #[must_use]
    pub fn stored_package(&self, code: &str) -> Option<&str> {
        let normalized = self.normalize(code);
        self.packages.get(normalized.as_str()).and_then(|opt| *opt)
    }

    /// Check if an NDC code has a valid format.
    ///
    /// This checks the structure but doesn't verify if the code exists in the database.
    #[must_use]
    pub fn is_valid_format(&self, code: &str) -> bool {
        let normalized = self.normalize(code);

        // Check for hyphen separators
        let parts: Vec<&str> = normalized.split('-').collect();
        if parts.len() != 3 {
            return false;
        }

        // Check each part contains only digits and has valid length
        let (labeler, product, package) = (parts[0], parts[1], parts[2]);

        // Labeler: 4-5 digits
        // Product: 3-4 digits
        // Package: 2 digits
        labeler.len() >= 4
            && labeler.len() <= 5
            && product.len() >= 3
            && product.len() <= 4
            && package.len() == 2
            && labeler.chars().all(|c| c.is_ascii_digit())
            && product.chars().all(|c| c.is_ascii_digit())
            && package.chars().all(|c| c.is_ascii_digit())
    }

    /// Normalize an NDC code to standard format.
    ///
    /// Removes whitespace and converts to uppercase.
    /// Does not change the digit grouping or add hyphens.
    #[must_use]
    pub fn normalize(&self, code: &str) -> String {
        code.trim()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase()
    }
}

impl CodeSystem for Ndc {
    fn lookup(&self, code: &str) -> Result<Code, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Ndc));
        }

        if let Some(&description) = self.descriptions.get(normalized.as_str()) {
            Ok(Code::new(System::Ndc, normalized, description))
        } else {
            Err(MedCodeError::not_found(code, System::Ndc))
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
            return Err(MedCodeError::invalid_format(code, System::Ndc));
        }

        // NDC doesn't have a hierarchical structure like ATC or ICD
        // But we can return the labeler and product as "ancestors" if they exist in our data
        let mut ancestors = Vec::new();

        if let Some((labeler, product, _package)) = self.parse_components(&normalized) {
            // Return product code as ancestor (if it exists as a standalone entry)
            let product_code = format!("{labeler}-{product}");
            if let Some(&description) = self.descriptions.get(product_code.as_str()) {
                ancestors.push(Code::new(System::Ndc, product_code, description));
            }

            // Return labeler code as ancestor (if it exists as a standalone entry)
            // Note: Most NDC datasets don't have standalone labeler entries, so this might be empty
            if let Some(&description) = self.descriptions.get(labeler.as_str()) {
                ancestors.push(Code::new(System::Ndc, labeler.clone(), description));
            }
        }

        Ok(ancestors)
    }

    fn descendants(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Ndc));
        }

        // For NDC, descendants would be more specific codes
        // If it's a labeler, return all products
        // If it's a product, return all packages
        let mut descendants = Vec::new();

        // This is a simplified implementation - in practice, you'd need
        // reverse lookups or additional data structures for efficiency
        for (&ndc_code, &description) in self.descriptions {
            if let Some((labeler, product, package)) = self.parse_components(ndc_code) {
                let current_code = if package.is_empty() {
                    labeler.clone() // Product-level code (no package)
                } else {
                    format!("{labeler}-{product}") // Product code
                };

                if current_code == normalized {
                    descendants.push(Code::new(System::Ndc, ndc_code.to_string(), description));
                }
            }
        }

        Ok(descendants)
    }

    fn parent(&self, code: &str) -> Result<Option<Code>, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Ndc));
        }

        if let Some((labeler, product, _package)) = self.parse_components(&normalized) {
            // If it's a full NDC, parent is the product
            if !product.is_empty() {
                let product_code = format!("{labeler}-{product}");
                if let Some(&description) = self.descriptions.get(product_code.as_str()) {
                    return Ok(Some(Code::new(System::Ndc, product_code, description)));
                }
            }
            // If it's a product, parent is the labeler
            else if let Some(&description) = self.descriptions.get(labeler.as_str()) {
                return Ok(Some(Code::new(System::Ndc, labeler.clone(), description)));
            }
        }

        Ok(None)
    }

    fn children(&self, code: &str) -> Result<Vec<Code>, MedCodeError> {
        let normalized = self.normalize(code);

        if !self.is_valid_format(&normalized) {
            return Err(MedCodeError::invalid_format(code, System::Ndc));
        }

        let mut children = Vec::new();

        // Find codes that have this as their parent
        for (&ndc_code, &description) in self.descriptions {
            if let Some((labeler, product, package)) = self.parse_components(ndc_code) {
                let parent_code = if package.is_empty() {
                    labeler.clone() // Product-level code (no package)
                } else {
                    format!("{labeler}-{product}") // Product code
                };

                if parent_code == normalized {
                    children.push(Code::new(System::Ndc, ndc_code.to_string(), description));
                }
            }
        }

        Ok(children)
    }
}

// Include generated data
include!(concat!(env!("OUT_DIR"), "/ndc_data.rs"));

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_ndc_new() {
        let ndc = Ndc::new();
        assert!(!ndc.descriptions.is_empty());
    }

    #[test]
    fn test_normalize() {
        let ndc = Ndc::new();

        assert_eq!(ndc.normalize("1234-5678-90"), "1234-5678-90");
        assert_eq!(ndc.normalize(" 1234-5678-90 "), "1234-5678-90");
        assert_eq!(ndc.normalize("1234-5678-90\n"), "1234-5678-90");
    }

    #[test]
    fn test_normalize_idempotent() {
        let ndc = Ndc::new();
        let code = "1234-5678-90";
        let normalized = ndc.normalize(code);
        let renormalized = ndc.normalize(&normalized);
        assert_eq!(normalized, renormalized);
    }

    #[test]
    fn test_valid_format() {
        let ndc = Ndc::new();

        // Valid formats
        assert!(ndc.is_valid_format("1234-5678-90"));
        assert!(ndc.is_valid_format("12345-6789-01"));
        assert!(ndc.is_valid_format("1234-567-01"));

        // Invalid formats
        assert!(!ndc.is_valid_format(""));
        assert!(!ndc.is_valid_format("1234567890")); // No hyphens
        assert!(!ndc.is_valid_format("1234-5678")); // Missing package
        assert!(!ndc.is_valid_format("1234-5678-90-12")); // Too many parts
        assert!(!ndc.is_valid_format("ABCD-EFGH-IJ")); // Non-digit characters
    }

    #[test]
    fn test_parse_components() {
        let ndc = Ndc::new();

        let components = ndc.parse_components("1234-5678-90");
        assert!(components.is_some());
        let (labeler, product, package) = components.expect("components should be Some");
        assert_eq!(labeler, "1234");
        assert_eq!(product, "5678");
        assert_eq!(package, "90");

        // Invalid format should return None
        assert!(ndc.parse_components("invalid").is_none());
    }

    #[test]
    fn test_component_accessors() {
        let ndc = Ndc::new();

        assert_eq!(ndc.labeler("1234-5678-90"), Some("1234".to_string()));
        assert_eq!(ndc.product("1234-5678-90"), Some("5678".to_string()));
        assert_eq!(ndc.package("1234-5678-90"), Some("90".to_string()));

        // Invalid code
        assert!(ndc.labeler("invalid").is_none());
        assert!(ndc.product("invalid").is_none());
        assert!(ndc.package("invalid").is_none());
    }

    #[test]
    fn test_level_detection() {
        let ndc = Ndc::new();

        // Test different component lengths
        assert!(ndc.parse_components("1234-5678-90").is_some()); // 4-4-2 format
        assert!(ndc.parse_components("12345-6789-01").is_some()); // 5-4-2 format
        assert!(ndc.parse_components("1234-567-01").is_some()); // 4-3-2 format
    }

    #[test]
    fn test_stored_methods() {
        let ndc = Ndc::new();

        // Test stored methods return values from data
        assert_eq!(ndc.stored_labeler("1234-5678-90"), Some("1234"));
        assert_eq!(ndc.stored_product("1234-5678-90"), Some("5678"));
        assert_eq!(ndc.stored_package("1234-5678-90"), Some("90"));

        // Test invalid code
        assert!(ndc.stored_labeler("invalid").is_none());
        assert!(ndc.stored_product("invalid").is_none());
        assert!(ndc.stored_package("invalid").is_none());
    }
}
