//! Tests for ICD-9-CM code system

use medcodes::{CodeSystem, icd9::Icd9Cm};

#[test]
#[allow(clippy::unwrap_used)]
fn test_icd9cm_lookup() {
    let icd9 = Icd9Cm::new();

    // Test lookup of a known code
    let result = icd9.lookup("001.0");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert_eq!(code.code(), "001.0");
    assert!(code.description().contains("Cholera"));

    // Test lookup of parent code
    let result = icd9.lookup("001");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert_eq!(code.code(), "001");
    assert_eq!(code.description(), "CHOLERA");
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_icd9cm_hierarchy() {
    let icd9 = Icd9Cm::new();

    // Test parent relationship
    let parent = icd9.parent("001.0").unwrap();
    assert!(parent.is_some());
    let parent_code = parent.unwrap();
    assert_eq!(parent_code.code(), "001");
    assert_eq!(parent_code.description(), "CHOLERA");

    // Test children relationship
    let children = icd9.children("001").unwrap();
    assert!(!children.is_empty());
    assert!(children.iter().any(|c| c.code() == "001.0"));
    assert!(children.iter().any(|c| c.code() == "001.1"));
    assert!(children.iter().any(|c| c.code() == "001.9"));

    // Test ancestors
    let ancestors = icd9.ancestors("001.0").unwrap();
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0].code(), "001");

    // Test descendants
    let descendants = icd9.descendants("001").unwrap();
    assert!(descendants.len() >= 3); // Should have at least 3 children
}

#[test]
fn test_icd9cm_validation() {
    let icd9 = Icd9Cm::new();

    // Valid codes
    assert!(icd9.is_valid("001"));
    assert!(icd9.is_valid("001.0"));
    assert!(icd9.is_valid("002.0"));

    // Invalid codes
    assert!(!icd9.is_valid("999.999")); // Non-existent code
    assert!(!icd9.is_valid("")); // Empty code
    assert!(!icd9.is_valid("ABC")); // Invalid format
}

#[test]
fn test_icd9cm_normalization() {
    let icd9 = Icd9Cm::new();

    // Basic normalization
    assert_eq!(icd9.normalize("001.0"), "001.0");
    assert_eq!(icd9.normalize(" 001 .0 "), "001.0");
    assert_eq!(icd9.normalize("0010"), "001.0"); // Adds dot

    // Case conversion
    assert_eq!(icd9.normalize("cholera"), "CHOLERA");
    assert_eq!(icd9.normalize("Cholera"), "CHOLERA");

    // 3-digit codes stay as-is
    assert_eq!(icd9.normalize("001"), "001");
    assert_eq!(icd9.normalize("V01"), "V01");
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_icd9cm_error_handling() {
    let icd9 = Icd9Cm::new();

    // Test lookup of non-existent code
    let result = icd9.lookup("999.999");
    assert!(result.is_err());

    // Test hierarchy operations on non-existent code
    let result = icd9.parent("999.999").unwrap();
    assert!(result.is_none()); // Parent returns Ok(None) for non-existent codes

    let result = icd9.children("999.999").unwrap();
    assert!(result.is_empty()); // Children returns Ok(empty) for non-existent codes

    // Note: ancestors and descendants might return empty results for non-existent codes
    // rather than errors, which is also valid behavior
    let result = icd9.ancestors("999.999").unwrap_or_default();
    assert!(result.is_empty()); // Should be empty for non-existent codes

    let result = icd9.descendants("999.999").unwrap_or_default();
    assert!(result.is_empty()); // Should be empty for non-existent codes
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_icd9comprehensive_traversal() {
    let icd9 = Icd9Cm::new();

    // Test that we can traverse from child to parent and back
    let child_code = "001.0";
    let parent = icd9.parent(child_code).unwrap().unwrap();
    let children = icd9.children(parent.code()).unwrap();

    // The original child should be in the parent's children list
    assert!(children.iter().any(|c| c.code() == child_code));

    // Test full ancestor chain
    let ancestors = icd9.ancestors(child_code).unwrap();
    if !ancestors.is_empty() {
        // If we have ancestors, we should be able to traverse back
        let top_ancestor = &ancestors.last().unwrap();
        let all_descendants = icd9.descendants(top_ancestor.code()).unwrap();
        assert!(all_descendants.iter().any(|c| c.code() == child_code));
    }
}
