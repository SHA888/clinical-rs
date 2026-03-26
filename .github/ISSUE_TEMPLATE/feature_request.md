---
name: Feature Request
about: Suggest an idea for this project
title: "[FEAT] "
labels: enhancement
assignees: ''
---

**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem is. Ex. I'm always frustrated when [...]

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Which crate(s) would this feature affect?**
- [ ] medcodes (medical code ontologies)
- [ ] mimic-etl (MIMIC dataset processing)
- [ ] clinical-tasks (ML task windowing)
- [ ] workspace (affects multiple crates)

**Use Case**
Describe the specific use case for this feature:
- What clinical problem does this solve?
- Who would benefit from this feature?
- What type of data would be processed?

**Proposed API**
If this is a code-related feature, please suggest a rough API:
```rust
// Example API design
pub fn proposed_function() -> ReturnType {
    // Implementation details
}
```

**Implementation Considerations**
- **Performance**: Any performance requirements or constraints?
- **Memory**: Memory usage considerations?
- **Safety**: Any security or data integrity concerns?
- **Dependencies**: Would this require new dependencies?

**Clinical Domain Knowledge**
If this feature involves medical codes or clinical concepts:
- Which medical coding systems are involved? (ICD-10, SNOMED, LOINC, etc.)
- Are there specific clinical guidelines or standards to follow?
- Any regulatory considerations (HIPAA, GDPR, etc.)?

**Additional context**
Add any other context, mockups, or screenshots about the feature request here.

**Would you be willing to contribute?**
- [ ] Yes, I'd like to implement this feature
- [ ] Yes, I'd like to help with documentation
- [ ] Yes, I'd like to help with testing
- [ ] No, I'd prefer the maintainers to implement it
