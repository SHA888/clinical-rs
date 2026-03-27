use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=data/april-1-2026-code-tables-tabular-and-index.zip");
    
    let data_path = "crates/medcodes/data/Table and Index/icd10cm_tabular_2026.xml";
    
    if Path::new(data_path).exists() {
        match parse_cms_xml(data_path) {
            Ok((codes, parents, children)) => {
                generate_phf_maps(&codes, &parents, &children);
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse CMS XML: {}", e);
                eprintln!("Using empty maps. Ensure the CMS data is extracted properly.");
            }
        }
    } else {
        eprintln!("Warning: CMS data file not found at {}", data_path);
        eprintln!("Extract the ZIP file to populate ICD-10-CM codes.");
    }
}

fn parse_cms_xml(
    path: &str,
) -> Result<
    (
        Vec<(String, String)>,
        Vec<(String, String)>,
        HashMap<String, Vec<String>>,
    ),
    Box<dyn std::error::Error>,
> {
    let content = fs::read_to_string(path)?;
    let mut codes = Vec::new();
    let mut parents = Vec::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();

    // Simple XML parsing - extract <name> and <desc> pairs
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with("<name>") && line.ends_with("</name>") {
            // Extract code
            let code = line
                .strip_prefix("<name>")
                .and_then(|s| s.strip_suffix("</name>"))
                .unwrap_or("")
                .to_uppercase();
            
            // Look for description in next few lines
            let mut desc = String::new();
            let mut j = i + 1;
            while j < lines.len() && j < i + 5 {
                let next_line = lines[j].trim();
                if next_line.starts_with("<desc>") && next_line.ends_with("</desc>") {
                    desc = next_line
                        .strip_prefix("<desc>")
                        .and_then(|s| s.strip_suffix("</desc>"))
                        .unwrap_or("")
                        .to_string();
                    break;
                }
                j += 1;
            }
            
            if !code.is_empty() && !desc.is_empty() {
                // Determine parent code (remove last character for hierarchical codes)
                if code.len() > 3 {
                    let parent = code[..code.len() - 1].to_string();
                    parents.push((code.clone(), parent.clone()));
                    children.entry(parent).or_insert_with(Vec::new).push(code.clone());
                }
                
                codes.push((code, desc));
            }
        }
        
        i += 1;
    }

    Ok((codes, parents, children))
}

fn generate_phf_maps(
    codes: &[(String, String)],
    _parents: &[(String, String)],
    children: &HashMap<String, Vec<String>>,
) {
    // For now, just log statistics
    eprintln!("Parsed {} ICD-10-CM codes", codes.len());
    eprintln!("Found {} parent-child relationships", children.len());
    eprintln!("Note: Full phf::Map generation requires proper code generation");
}
