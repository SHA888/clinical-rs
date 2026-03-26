# Security Policy

## Supported Versions

Only the latest stable version of clinical-rs receives security updates.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly.

### How to Report

**Preferred Method**: Use GitHub's private vulnerability reporting:
1. Go to the GitHub repository
2. Click "Security" tab
3. Click "Report a vulnerability"
4. Follow the prompts to submit a private report

**Alternative Method**: Email us at security@clinical-rs.org

### What to Include

Please include:
- Type of vulnerability (e.g., buffer overflow, injection, etc.)
- Affected version(s)
- Steps to reproduce the vulnerability
- Potential impact assessment
- Any proposed mitigations (if known)

### Response Timeline

- **Within 48 hours**: Initial acknowledgment and assessment
- **Within 7 days**: Detailed response and remediation timeline
- **Within 30 days**: Security patch release for critical vulnerabilities

## Security Scope

The following types of issues are considered security-critical for clinical-rs:

### Critical Security Issues
- **Data Integrity**: Any bug that could corrupt clinical data or medical codes
- **Data Privacy**: Unauthorized access to patient data or sensitive information
- **Code Injection**: Execution of arbitrary code through malformed inputs
- **Denial of Service**: Bugs that could crash clinical data processing systems

### High Priority Issues
- **Incorrect Medical Code Mapping**: Wrong ICD-10, SNOMED, or other medical code translations
- **Data Loss**: Accidental deletion or corruption of clinical datasets
- **Memory Safety**: Buffer overflows, use-after-free in data processing
- **Dependency Vulnerabilities**: Security issues in third-party dependencies

### Normal Issues
- **Performance Issues**: Slow processing that doesn't affect correctness
- **Documentation Errors**: Incorrect API documentation
- **Build/Installation Issues**: Problems with setup or compilation

## Disclosure Policy

We follow responsible disclosure principles:
1. **Private Coordination**: Work with reporters privately before public disclosure
2. **Timely Fixes**: Prioritize security patches based on severity
3. **Coordinated Disclosure**: Public disclosure after patches are available
4. **Credit**: Acknowledge responsible reporters (with permission)

## Security Best Practices

When using clinical-rs in production:
- Keep dependencies updated
- Use the latest stable version
- Validate all input data
- Implement proper access controls
- Regular security audits

## Security Team

The clinical-rs security team reviews all vulnerability reports and coordinates fixes.

## Legal Disclaimer

This security policy is provided for informational purposes. The clinical-rs team reserves the right to modify this policy at any time.
