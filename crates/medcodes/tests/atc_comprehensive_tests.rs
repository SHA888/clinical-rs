//! Additional comprehensive tests for ATC edge cases and robustness

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use medcodes::{
    CodeSystem,
    atc::{Atc, AtcLevel},
};

#[test]
fn test_atc_all_hierarchy_levels() {
    let atc = Atc::new();

    // Test all 5 hierarchy levels have valid codes
    assert!(atc.is_valid("A")); // Level 1: Anatomical
    assert!(atc.is_valid("A10")); // Level 2: Therapeutic
    assert!(atc.is_valid("A10B")); // Level 3: Pharmacological
    assert!(atc.is_valid("A10BA")); // Level 4: Chemical subgroup
    assert!(atc.is_valid("A10BA02")); // Level 5: Chemical substance
}

#[test]
fn test_atc_comprehensive_hierarchy_traversal() {
    let atc = Atc::new();

    // Test complete hierarchy chain from level 5 to level 1
    let drug_code = "A10BA02"; // metformin
    let ancestors = atc.ancestors(drug_code).unwrap();

    // Should traverse up through all levels
    assert!(ancestors.len() >= 4);

    // Verify each ancestor level
    let mut found_levels = Vec::new();
    for ancestor in &ancestors {
        if let Some(level) = atc.level(ancestor.code()) {
            found_levels.push(level);
        }
    }

    // Should have found multiple levels
    assert!(found_levels.len() >= 3);
}

#[test]
fn test_atc_descendants_completeness() {
    let atc = Atc::new();

    // Test that descendants include all expected levels
    let descendants = atc.descendants("A").unwrap(); // From anatomical level

    // Should include drugs at level 5
    let has_level5 = descendants.iter().any(|c| c.code().len() == 7);
    assert!(has_level5, "Should have level 5 descendants");

    // Should include subgroups at various levels
    let has_multiple_levels = descendants
        .iter()
        .map(|c| c.code().len())
        .collect::<std::collections::HashSet<_>>()
        .len()
        > 1;
    assert!(
        has_multiple_levels,
        "Should have descendants at multiple levels"
    );
}

#[test]
fn test_atc_ddd_various_formats() {
    let atc = Atc::new();

    // Test DDD values with different units
    let metformin_ddd = atc.get_defined_daily_dose("A10BA02");
    assert!(metformin_ddd.is_some());
    assert_eq!(metformin_ddd.unwrap(), "2g");

    let warfarin_ddd = atc.ddd("B01AA03");
    assert!(warfarin_ddd.is_some());
    assert_eq!(warfarin_ddd.unwrap(), "7.5mg");

    let aspirin_ddd = atc.ddd("B01AC06");
    assert!(aspirin_ddd.is_some());
    assert_eq!(aspirin_ddd.unwrap(), "100mg");

    // Test that groups don't have DDD values
    let anatomical_ddd = atc.ddd("A");
    assert!(anatomical_ddd.is_none());

    let therapeutic_ddd = atc.ddd("A10");
    assert!(therapeutic_ddd.is_none());
}

#[test]
fn test_atc_case_insensitive_lookup() {
    let atc = Atc::new();

    // Test various case combinations
    let variations = ["c10aa01", "C10AA01", "c10AA01", "C10aa01"];

    for variation in variations {
        let result = atc.lookup(variation);
        assert!(result.is_ok(), "Should handle case variation: {variation}");

        let code = result.unwrap();
        assert_eq!(code.code(), "C10AA01"); // Should be normalized
    }
}

#[test]
fn test_atc_whitespace_handling() {
    let atc = Atc::new();

    // Test various whitespace patterns
    let variations = [
        " C10AA01 ",
        "  C10AA01  ",
        "C 10 AA 01",
        "\tC10AA01\t",
        "C10AA01\n",
    ];

    for variation in variations {
        let result = atc.lookup(variation);
        assert!(
            result.is_ok(),
            "Should handle whitespace variation: {variation:?}"
        );

        let code = result.unwrap();
        assert_eq!(code.code(), "C10AA01"); // Should be normalized
    }
}

#[test]
fn test_atc_invalid_format_handling() {
    let atc = Atc::new();

    // Test various invalid formats
    let invalid_codes = [
        "",         // Empty
        "1",        // Starts with digit
        "C1",       // Invalid length (2)
        "C10AA0",   // Invalid length (6)
        "C10AA012", // Too long (8)
        "c10aa01x", // Invalid character at end
        "C10AA0X",  // Invalid character in middle
        "@10AA01",  // Invalid first character
    ];

    for invalid_code in invalid_codes {
        let result = atc.lookup(invalid_code);
        assert!(
            result.is_err(),
            "Should reject invalid format: {invalid_code}"
        );
        assert!(
            !atc.is_valid(invalid_code),
            "Should be invalid: {invalid_code}"
        );
    }
}

#[test]
fn test_atc_hierarchy_edge_cases() {
    let atc = Atc::new();

    // Test hierarchy operations on invalid codes
    let invalid_code = "INVALID123";

    assert!(atc.parent(invalid_code).is_err());
    assert!(atc.children(invalid_code).is_err());
    assert!(atc.ancestors(invalid_code).is_err());
    assert!(atc.descendants(invalid_code).is_err());
}

#[test]
fn test_atc_level_edge_cases() {
    let atc = Atc::new();

    // Test level detection on various inputs
    assert_eq!(atc.level("C"), Some(AtcLevel::Anatomical));
    assert_eq!(atc.level("C10"), Some(AtcLevel::Therapeutic));
    assert_eq!(atc.level("C10A"), Some(AtcLevel::Pharmacological));
    assert_eq!(atc.level("C10AA"), Some(AtcLevel::ChemicalSubgroup));
    assert_eq!(atc.level("C10AA01"), Some(AtcLevel::ChemicalSubstance));

    // Test invalid lengths
    assert_eq!(atc.level(""), None);
    assert_eq!(atc.level("C1"), None); // Length 2
    assert_eq!(atc.level("C10AA0"), None); // Length 6
    assert_eq!(atc.level("C10AA0123"), None); // Length 8
}

#[test]
fn test_atc_normalization_edge_cases() {
    let atc = Atc::new();

    // Test normalization preserves valid codes
    let valid_codes = ["C", "C10", "C10A", "C10AA", "C10AA01"];

    for code in valid_codes {
        let normalized = atc.normalize(code);
        assert_eq!(
            normalized,
            code.to_uppercase(),
            "Valid code should normalize to uppercase: {code}"
        );
    }

    // Test normalization handles whitespace
    assert_eq!(atc.normalize(" C10AA01 "), "C10AA01");
    assert_eq!(atc.normalize("\tC10AA01\n"), "C10AA01");
}

#[test]
fn test_atc_cross_system_consistency() {
    let atc = Atc::new();

    // Test that all codes in descriptions map can be looked up
    // This tests data consistency
    let test_codes = ["C10AA01", "A10BA02", "B01AA03", "B01AC06"];

    for code in test_codes {
        if atc.is_valid(code) {
            let result = atc.lookup(code);
            assert!(result.is_ok(), "Valid code should be lookup-able: {code}");

            let code_obj = result.expect("Valid code lookup should succeed");
            assert_eq!(code_obj.code(), code.to_uppercase());
            assert!(!code_obj.description().is_empty());
        }
    }
}

#[test]
fn test_atc_performance_large_hierarchy() {
    let atc = Atc::new();

    // Test performance with large hierarchy traversal
    let start = std::time::Instant::now();

    // Get all descendants from top level
    let descendants = atc
        .descendants("C")
        .expect("Descendants lookup should succeed");

    let duration = start.elapsed();

    // Should complete quickly (less than 100ms for reasonable performance)
    assert!(
        duration.as_millis() < 100,
        "Hierarchy traversal should be fast, took: {duration:?}"
    );

    // Should have some descendants
    assert!(
        !descendants.is_empty(),
        "Should have cardiovascular system descendants"
    );
}

#[test]
fn test_atc_error_handling() {
    let atc = Atc::new();

    // Test that error messages are informative
    let result = atc.lookup("INVALID");
    assert!(result.is_err());

    let error = result.expect_err("Invalid code lookup should fail");
    let error_str = error.to_string();
    assert!(error_str.contains("INVALID"));
    assert!(error_str.contains("ATC"));
}

#[test]
fn test_atc_ddd_method_consistency() {
    let atc = Atc::new();

    // Test that both ddd() and get_defined_daily_dose() return the same results
    let test_codes = ["A10BA02", "B01AA03", "B01AC06"];

    for code in test_codes {
        let ddd1 = atc.ddd(code);
        let ddd2 = atc.get_defined_daily_dose(code);

        assert_eq!(
            ddd1, ddd2,
            "Both DDD methods should return same result for {code}"
        );
    }
}
