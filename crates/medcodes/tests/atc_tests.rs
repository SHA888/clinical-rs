//! Tests for ATC (Anatomical Therapeutic Chemical) code system

use medcodes::{CodeSystem, atc::Atc};

#[test]
fn test_atc_lookup() {
    let atc = Atc::new();

    // Test lookup of a chemical substance (level 5)
    let result = atc.lookup("C10AA01");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert_eq!(code.code(), "C10AA01");
    assert!(code.description().contains("simvastatin"));

    // Test lookup of anatomical group (level 1)
    let result = atc.lookup("C");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert_eq!(code.code(), "C");
    assert_eq!(code.description(), "CARDIOVASCULAR SYSTEM");
}

#[test]
fn test_atc_hierarchy() {
    let atc = Atc::new();

    // Test parent relationship - simvastatin -> HMG CoA reductase inhibitors
    let parent = atc.parent("C10AA01").unwrap();
    assert!(parent.is_some());
    let parent_code = parent.unwrap();
    assert_eq!(parent_code.code(), "C10AA");
    assert!(parent_code.description().contains("HMG COA"));

    // Test children relationship
    let children = atc.children("C10AA").unwrap();
    assert!(!children.is_empty());
    // Should have simvastatin and other statins
    assert!(children.iter().any(|c| c.code() == "C10AA01"));

    // Test ancestors chain
    let ancestors = atc.ancestors("C10AA01").unwrap();
    assert!(ancestors.len() >= 4); // Should have multiple levels
    // First ancestor should be C10AA
    assert_eq!(ancestors[0].code(), "C10AA");

    // Test descendants
    let descendants = atc.descendants("C").unwrap();
    assert!(!descendants.is_empty());
    // Should include cardiovascular drugs
    assert!(descendants.iter().any(|c| c.code().starts_with("C10")));
}

#[test]
fn test_atc_ddd() {
    let atc = Atc::new();

    // Test DDD lookup for simvastatin
    let ddd = atc.ddd("C10AA01");
    assert!(ddd.is_some());
    assert_eq!(ddd.unwrap(), "20mg");

    // Test DDD lookup for metformin
    let ddd = atc.ddd("A10BA02");
    assert!(ddd.is_some());
    assert_eq!(ddd.unwrap(), "2g");

    // Test DDD for a group (should be None)
    let ddd = atc.ddd("C10");
    assert!(ddd.is_none());
}

#[test]
fn test_atc_validation() {
    let atc = Atc::new();

    // Valid codes at different levels
    assert!(atc.is_valid("C")); // Level 1
    assert!(atc.is_valid("C10")); // Level 2
    assert!(atc.is_valid("C10A")); // Level 3
    assert!(atc.is_valid("C10AA")); // Level 4
    assert!(atc.is_valid("C10AA01")); // Level 5

    // Invalid format codes
    assert!(!atc.is_valid("1")); // Must start with letter
    assert!(!atc.is_valid("C1")); // Invalid length
    assert!(!atc.is_valid("C10AA0")); // Invalid length (6 chars)
    assert!(!atc.is_valid("")); // Empty
    assert!(!atc.is_valid("C10AA012")); // Too long

    // Non-existent but valid format
    assert!(!atc.is_valid("Z99ZZ99")); // Valid format but not in data
}

#[test]
fn test_atc_level_detection() {
    use medcodes::atc::AtcLevel;

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
fn test_atc_normalization() {
    let atc = Atc::new();

    // Case conversion
    assert_eq!(atc.normalize("c10aa01"), "C10AA01");
    assert_eq!(atc.normalize("C10AA01"), "C10AA01");

    // Whitespace removal
    assert_eq!(atc.normalize(" c10aa01 "), "C10AA01");
    assert_eq!(atc.normalize("C 10 AA 01"), "C10AA01");
}

#[test]
fn test_atc_error_handling() {
    let atc = Atc::new();

    // Invalid format should return InvalidFormat error
    let result = atc.lookup("C1"); // Invalid length
    assert!(result.is_err());

    // Non-existent valid format should return NotFound error
    let result = atc.lookup("Z99ZZ99"); // Valid format but not in data
    assert!(result.is_err());

    // Invalid format for hierarchy operations
    let result = atc.parent("C1");
    assert!(result.is_err());

    let result = atc.children("C1");
    assert!(result.is_err());
}

#[test]
fn test_atc_level_properties() {
    use medcodes::atc::AtcLevel;

    assert_eq!(AtcLevel::Anatomical.code_length(), 1);
    assert_eq!(AtcLevel::Therapeutic.code_length(), 3);
    assert_eq!(AtcLevel::Pharmacological.code_length(), 4);
    assert_eq!(AtcLevel::ChemicalSubgroup.code_length(), 5);
    assert_eq!(AtcLevel::ChemicalSubstance.code_length(), 7);

    assert_eq!(AtcLevel::Anatomical.name(), "Anatomical main group");
    assert_eq!(AtcLevel::Therapeutic.name(), "Therapeutic subgroup");
    assert_eq!(AtcLevel::Pharmacological.name(), "Pharmacological subgroup");
    assert_eq!(AtcLevel::ChemicalSubgroup.name(), "Chemical subgroup");
    assert_eq!(AtcLevel::ChemicalSubstance.name(), "Chemical substance");
}

#[test]
fn test_atc_hierarchy_traversal_comprehensive() {
    let atc = Atc::new();

    // Start from a specific drug and traverse up
    let drug_code = "C10AA01"; // simvastatin

    // Get parent (chemical subgroup)
    let parent = atc.parent(drug_code).unwrap().unwrap();
    assert_eq!(parent.code(), "C10AA");

    // Get children of the parent
    let siblings = atc.children(parent.code()).unwrap();
    assert!(siblings.iter().any(|c| c.code() == drug_code));

    // Get all ancestors
    let ancestors = atc.ancestors(drug_code).unwrap();
    let ancestor_codes: Vec<&str> = ancestors.iter().map(medcodes::Code::code).collect();

    // Should have C10AA, C10A, C10, C in the chain
    assert!(ancestor_codes.contains(&"C10AA"));
    assert!(ancestor_codes.contains(&"C10A"));
    assert!(ancestor_codes.contains(&"C10"));
    assert!(ancestor_codes.contains(&"C"));
}

#[test]
fn test_atc_diabetes_hierarchy() {
    let atc = Atc::new();

    // Test diabetes drug hierarchy
    let result = atc.lookup("A10BA02"); // metformin
    assert!(result.is_ok());

    let ancestors = atc.ancestors("A10BA02").unwrap();
    assert!(!ancestors.is_empty());

    // Should be part of biguanides (A10BA)
    let parent = atc.parent("A10BA02").unwrap();
    assert!(parent.is_some());
    assert_eq!(parent.expect("Parent should exist").code(), "A10BA");
}

#[test]
fn test_atc_antithrombotic_agents() {
    let atc = Atc::new();

    // Test warfarin DDD
    let ddd = atc.ddd("B01AA03");
    assert!(ddd.is_some());
    assert_eq!(ddd.expect("DDD should exist"), "7.5mg");

    // Test aspirin DDD
    let ddd = atc.ddd("B01AC06");
    assert!(ddd.is_some());
    assert_eq!(ddd.expect("DDD should exist"), "100mg");

    // Test clopidogrel DDD
    let ddd = atc.ddd("B01AC04");
    assert!(ddd.is_some());
    assert_eq!(ddd.expect("DDD should exist"), "75mg");
}
