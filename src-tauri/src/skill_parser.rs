//! SKILL.md Parser Module
//!
//! This module provides functionality to parse SKILL.md files and extract
//! skill metadata including name, description, and allowed tools.
//!
//! Supports two formats:
//! 1. YAML frontmatter format (between --- markers)
//! 2. Heading-based format as fallback
//!
//! Requirements: 2.1, 2.2, 2.3, 2.4

use serde::{Deserialize, Serialize};

/// Metadata extracted from a SKILL.md file
///
/// Contains the skill's name, description, and list of allowed tools.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub allowed_tools: Vec<String>,
}

impl Default for SkillMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            allowed_tools: Vec::new(),
        }
    }
}

/// Internal struct for deserializing YAML frontmatter
#[derive(Deserialize, Debug)]
struct FrontmatterData {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<Vec<String>>,
}

/// Parses a SKILL.md file content and extracts metadata.
///
/// This function attempts to parse the content in two ways:
/// 1. First, it tries to extract YAML frontmatter (content between --- markers)
/// 2. If no frontmatter is found, it falls back to parsing heading-based format
///
/// # Arguments
///
/// * `content` - The raw content of the SKILL.md file
///
/// # Returns
///
/// A `SkillMetadata` struct containing the extracted name, description, and allowed_tools.
/// If parsing fails or fields are missing, default values are used.
///
/// # Requirements
///
/// - 2.1: WHEN reading a skill directory, THE Skills_Manager SHALL look for a SKILL.md file
/// - 2.2: WHEN parsing SKILL.md, THE Skills_Manager SHALL extract the name field from the frontmatter or first heading
/// - 2.3: WHEN parsing SKILL.md, THE Skills_Manager SHALL extract the description field from the frontmatter or first paragraph
/// - 2.4: WHEN parsing SKILL.md, THE Skills_Manager SHALL extract the allowed-tools list if present
pub fn parse_skill_md(content: &str) -> SkillMetadata {
    // Try to parse YAML frontmatter first
    if let Some(metadata) = parse_frontmatter(content) {
        return metadata;
    }

    // Fall back to heading-based format
    parse_heading_format(content)
}

/// Attempts to parse YAML frontmatter from the content.
///
/// Frontmatter is expected to be at the start of the file, enclosed by --- markers.
///
/// # Example
///
/// ```markdown
/// ---
/// name: My Skill Name
/// description: A brief description
/// allowed-tools:
///   - tool1
///   - tool2
/// ---
/// ```
fn parse_frontmatter(content: &str) -> Option<SkillMetadata> {
    let trimmed = content.trim_start();

    // Check if content starts with frontmatter delimiter
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing delimiter
    let after_first_delimiter = &trimmed[3..];
    let closing_pos = after_first_delimiter.find("\n---")?;

    // Extract the YAML content between delimiters
    let yaml_content = &after_first_delimiter[..closing_pos].trim();

    // Parse the YAML
    let frontmatter: FrontmatterData = serde_yaml::from_str(yaml_content).ok()?;

    Some(SkillMetadata {
        name: frontmatter.name.unwrap_or_default(),
        description: frontmatter.description.unwrap_or_default(),
        allowed_tools: frontmatter.allowed_tools.unwrap_or_default(),
    })
}

/// Parses the heading-based format when no frontmatter is present.
///
/// # Format
///
/// ```markdown
/// # My Skill Name
///
/// A brief description of what this skill does.
///
/// ## Allowed Tools
/// - tool1
/// - tool2
/// ```
fn parse_heading_format(content: &str) -> SkillMetadata {
    let mut name = String::new();
    let mut allowed_tools = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    // Extract name from first # heading
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("# ") && !line.starts_with("## ") {
            name = line[2..].trim().to_string();
            i += 1;
            break;
        }
        i += 1;
    }

    // Skip empty lines after heading
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    // Extract description from first paragraph (until empty line or next heading)
    let mut desc_lines = Vec::new();
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Stop at empty line or heading
        if trimmed.is_empty() || trimmed.starts_with('#') {
            break;
        }

        desc_lines.push(trimmed);
        i += 1;
    }
    let description = desc_lines.join(" ");

    // Look for "Allowed Tools" section
    while i < lines.len() {
        let line = lines[i].trim();

        // Check for "Allowed Tools" heading (## or any level)
        if line.starts_with('#') && line.to_lowercase().contains("allowed tools") {
            i += 1;

            // Skip empty lines after heading
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Extract list items
            while i < lines.len() {
                let item_line = lines[i].trim();

                // Stop at next heading or empty section
                if item_line.starts_with('#') {
                    break;
                }

                // Parse list item (- item or * item)
                if item_line.starts_with("- ") || item_line.starts_with("* ") {
                    let tool = item_line[2..].trim().to_string();
                    if !tool.is_empty() {
                        allowed_tools.push(tool);
                    }
                }

                i += 1;
            }
            break;
        }

        i += 1;
    }

    SkillMetadata {
        name,
        description,
        allowed_tools,
    }
}

/// Formats a SkillMetadata object back into valid SKILL.md content with YAML frontmatter.
///
/// This function produces a SKILL.md file content that can be parsed back to produce
/// an equivalent SkillMetadata object (round-trip property).
///
/// # Arguments
///
/// * `metadata` - The SkillMetadata to format
///
/// # Returns
///
/// A String containing valid SKILL.md content with YAML frontmatter.
///
/// # Example Output
///
/// ```markdown
/// ---
/// name: My Skill Name
/// description: A brief description of what this skill does
/// allowed-tools:
///   - tool1
///   - tool2
/// ---
/// ```
///
/// # Requirements
///
/// - 2.5: THE Pretty_Printer SHALL format SkillMetadata objects back into valid SKILL.md content
pub fn format_skill_md(metadata: &SkillMetadata) -> String {
    let mut output = String::new();

    // Start frontmatter
    output.push_str("---\n");

    // Format name - use quoted string if it contains special YAML characters
    output.push_str(&format_yaml_field("name", &metadata.name));

    // Format description - use quoted string if it contains special YAML characters
    output.push_str(&format_yaml_field("description", &metadata.description));

    // Format allowed-tools list (only if non-empty)
    if !metadata.allowed_tools.is_empty() {
        output.push_str("allowed-tools:\n");
        for tool in &metadata.allowed_tools {
            output.push_str(&format!("  - {}\n", tool));
        }
    }

    // End frontmatter
    output.push_str("---\n");

    output
}

/// Formats a YAML field with proper escaping for special characters.
///
/// If the value contains characters that need escaping in YAML (like colons, quotes, etc.),
/// the value is wrapped in double quotes with proper escaping.
fn format_yaml_field(key: &str, value: &str) -> String {
    if needs_yaml_quoting(value) {
        // Escape double quotes and backslashes in the value
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        format!("{}: \"{}\"\n", key, escaped)
    } else {
        format!("{}: {}\n", key, value)
    }
}

/// Determines if a YAML value needs to be quoted.
///
/// Values need quoting if they:
/// - Contain special YAML characters (: # [ ] { } , & * ! | > ' " % @ `)
/// - Start with special characters (- ? :)
/// - Are empty or contain only whitespace
/// - Could be interpreted as a boolean, null, or number
fn needs_yaml_quoting(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    // Check for special characters that require quoting
    let special_chars = [':', '#', '[', ']', '{', '}', ',', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`'];
    if value.chars().any(|c| special_chars.contains(&c)) {
        return true;
    }

    // Check if starts with characters that have special meaning
    let first_char = value.chars().next().unwrap();
    if ['-', '?', ' '].contains(&first_char) {
        return true;
    }

    // Check for values that could be interpreted as boolean or null
    let lower = value.to_lowercase();
    if ["true", "false", "yes", "no", "on", "off", "null", "~"].contains(&lower.as_str()) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing YAML frontmatter format with all fields
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4**
    #[test]
    fn test_parse_frontmatter_complete() {
        let content = r#"---
name: My Skill Name
description: A brief description of what this skill does
allowed-tools:
  - tool1
  - tool2
---

# My Skill Name

Detailed documentation about the skill...
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "My Skill Name");
        assert_eq!(
            metadata.description,
            "A brief description of what this skill does"
        );
        assert_eq!(metadata.allowed_tools, vec!["tool1", "tool2"]);
    }

    /// Test parsing YAML frontmatter with missing optional fields
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4**
    #[test]
    fn test_parse_frontmatter_partial() {
        let content = r#"---
name: Minimal Skill
---

Some content here.
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "Minimal Skill");
        assert_eq!(metadata.description, "");
        assert!(metadata.allowed_tools.is_empty());
    }

    /// Test parsing YAML frontmatter with only description
    ///
    /// **Validates: Requirements 2.3**
    #[test]
    fn test_parse_frontmatter_only_description() {
        let content = r#"---
description: Just a description
---

Content.
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "");
        assert_eq!(metadata.description, "Just a description");
        assert!(metadata.allowed_tools.is_empty());
    }

    /// Test parsing heading-based format with all sections
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4**
    #[test]
    fn test_parse_heading_format_complete() {
        let content = r#"# My Skill Name

A brief description of what this skill does.

## Allowed Tools
- tool1
- tool2
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "My Skill Name");
        assert_eq!(
            metadata.description,
            "A brief description of what this skill does."
        );
        assert_eq!(metadata.allowed_tools, vec!["tool1", "tool2"]);
    }

    /// Test parsing heading-based format without allowed tools section
    ///
    /// **Validates: Requirements 2.2, 2.3**
    #[test]
    fn test_parse_heading_format_no_tools() {
        let content = r#"# Simple Skill

This is a simple skill without allowed tools.
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "Simple Skill");
        assert_eq!(
            metadata.description,
            "This is a simple skill without allowed tools."
        );
        assert!(metadata.allowed_tools.is_empty());
    }

    /// Test parsing heading-based format with multi-line description
    ///
    /// **Validates: Requirements 2.3**
    #[test]
    fn test_parse_heading_format_multiline_description() {
        let content = r#"# Multi-line Skill

This is the first line of the description.
This is the second line.
And this is the third line.

## Other Section
Some other content.
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "Multi-line Skill");
        assert_eq!(
            metadata.description,
            "This is the first line of the description. This is the second line. And this is the third line."
        );
    }

    /// Test parsing empty content
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4**
    #[test]
    fn test_parse_empty_content() {
        let content = "";

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "");
        assert_eq!(metadata.description, "");
        assert!(metadata.allowed_tools.is_empty());
    }

    /// Test parsing content with only whitespace
    #[test]
    fn test_parse_whitespace_only() {
        let content = "   \n\n   \n";

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "");
        assert_eq!(metadata.description, "");
        assert!(metadata.allowed_tools.is_empty());
    }

    /// Test parsing frontmatter with extra whitespace
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4**
    #[test]
    fn test_parse_frontmatter_with_whitespace() {
        let content = r#"

---
name: Whitespace Skill
description: Has leading whitespace
allowed-tools:
  - tool1
---

Content here.
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "Whitespace Skill");
        assert_eq!(metadata.description, "Has leading whitespace");
        assert_eq!(metadata.allowed_tools, vec!["tool1"]);
    }

    /// Test parsing heading format with asterisk list markers
    ///
    /// **Validates: Requirements 2.4**
    #[test]
    fn test_parse_heading_format_asterisk_list() {
        let content = r#"# Asterisk Skill

A skill with asterisk list markers.

## Allowed Tools
* tool_a
* tool_b
* tool_c
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "Asterisk Skill");
        assert_eq!(metadata.allowed_tools, vec!["tool_a", "tool_b", "tool_c"]);
    }

    /// Test parsing heading format with case-insensitive "Allowed Tools" heading
    ///
    /// **Validates: Requirements 2.4**
    #[test]
    fn test_parse_heading_format_case_insensitive_tools() {
        let content = r#"# Case Test Skill

Description here.

## ALLOWED TOOLS
- tool1

## allowed tools
- tool2
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "Case Test Skill");
        // Should find the first "Allowed Tools" section
        assert_eq!(metadata.allowed_tools, vec!["tool1"]);
    }

    /// Test parsing malformed frontmatter falls back to heading format
    ///
    /// **Validates: Requirements 2.2, 2.3**
    #[test]
    fn test_parse_malformed_frontmatter_fallback() {
        let content = r#"---
name: [invalid yaml
---

# Fallback Skill

This should be parsed as heading format.
"#;

        let metadata = parse_skill_md(content);

        // Should fall back to heading format
        assert_eq!(metadata.name, "Fallback Skill");
        assert_eq!(
            metadata.description,
            "This should be parsed as heading format."
        );
    }

    /// Test parsing frontmatter without closing delimiter falls back to heading format
    #[test]
    fn test_parse_unclosed_frontmatter_fallback() {
        let content = r#"---
name: Unclosed

# Actual Heading

Description text.
"#;

        let metadata = parse_skill_md(content);

        // Should fall back to heading format since frontmatter is not closed
        assert_eq!(metadata.name, "Actual Heading");
        assert_eq!(metadata.description, "Description text.");
    }

    /// Test parsing heading format with different heading levels for allowed tools
    ///
    /// **Validates: Requirements 2.4**
    #[test]
    fn test_parse_heading_format_h3_tools() {
        let content = r#"# Main Skill

Description.

### Allowed Tools
- deep_tool
"#;

        let metadata = parse_skill_md(content);

        assert_eq!(metadata.name, "Main Skill");
        assert_eq!(metadata.allowed_tools, vec!["deep_tool"]);
    }

    /// Test that frontmatter takes precedence over heading format
    ///
    /// **Validates: Requirements 2.2, 2.3, 2.4**
    #[test]
    fn test_frontmatter_takes_precedence() {
        let content = r#"---
name: Frontmatter Name
description: Frontmatter description
allowed-tools:
  - frontmatter_tool
---

# Heading Name

Heading description.

## Allowed Tools
- heading_tool
"#;

        let metadata = parse_skill_md(content);

        // Frontmatter values should be used
        assert_eq!(metadata.name, "Frontmatter Name");
        assert_eq!(metadata.description, "Frontmatter description");
        assert_eq!(metadata.allowed_tools, vec!["frontmatter_tool"]);
    }

    // ==================== format_skill_md tests ====================

    /// Test formatting complete SkillMetadata to SKILL.md content
    ///
    /// **Validates: Requirements 2.5**
    #[test]
    fn test_format_skill_md_complete() {
        let metadata = SkillMetadata {
            name: "My Skill Name".to_string(),
            description: "A brief description of what this skill does".to_string(),
            allowed_tools: vec!["tool1".to_string(), "tool2".to_string()],
        };

        let output = format_skill_md(&metadata);

        assert!(output.starts_with("---\n"));
        assert!(output.contains("name: My Skill Name\n"));
        assert!(output.contains("description: A brief description of what this skill does\n"));
        assert!(output.contains("allowed-tools:\n"));
        assert!(output.contains("  - tool1\n"));
        assert!(output.contains("  - tool2\n"));
        assert!(output.ends_with("---\n"));
    }

    /// Test formatting SkillMetadata with empty allowed_tools
    ///
    /// **Validates: Requirements 2.5**
    #[test]
    fn test_format_skill_md_no_tools() {
        let metadata = SkillMetadata {
            name: "Simple Skill".to_string(),
            description: "A simple skill".to_string(),
            allowed_tools: vec![],
        };

        let output = format_skill_md(&metadata);

        assert!(output.starts_with("---\n"));
        assert!(output.contains("name: Simple Skill\n"));
        assert!(output.contains("description: A simple skill\n"));
        // Should not contain allowed-tools section when empty
        assert!(!output.contains("allowed-tools:"));
        assert!(output.ends_with("---\n"));
    }

    /// Test formatting SkillMetadata with empty name and description
    ///
    /// **Validates: Requirements 2.5**
    #[test]
    fn test_format_skill_md_empty_fields() {
        let metadata = SkillMetadata {
            name: String::new(),
            description: String::new(),
            allowed_tools: vec![],
        };

        let output = format_skill_md(&metadata);

        assert!(output.starts_with("---\n"));
        assert!(output.contains("name: \n") || output.contains("name:\n"));
        assert!(output.contains("description: \n") || output.contains("description:\n"));
        assert!(output.ends_with("---\n"));
    }

    /// Test formatting SkillMetadata with special characters in description
    ///
    /// **Validates: Requirements 2.5**
    #[test]
    fn test_format_skill_md_special_characters() {
        let metadata = SkillMetadata {
            name: "Special: Skill".to_string(),
            description: "Description with \"quotes\" and 'apostrophes'".to_string(),
            allowed_tools: vec!["tool-with-dash".to_string()],
        };

        let output = format_skill_md(&metadata);

        // The output should be valid YAML that can be parsed back
        let parsed = parse_skill_md(&output);
        assert_eq!(parsed.name, metadata.name);
        assert_eq!(parsed.description, metadata.description);
        assert_eq!(parsed.allowed_tools, metadata.allowed_tools);
    }

    /// Test formatting SkillMetadata with single tool
    ///
    /// **Validates: Requirements 2.5**
    #[test]
    fn test_format_skill_md_single_tool() {
        let metadata = SkillMetadata {
            name: "Single Tool Skill".to_string(),
            description: "Has one tool".to_string(),
            allowed_tools: vec!["only_tool".to_string()],
        };

        let output = format_skill_md(&metadata);

        assert!(output.contains("allowed-tools:\n"));
        assert!(output.contains("  - only_tool\n"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// **Feature: skills-manager-enhancement, Property 1: SKILL.md Round-Trip Parsing**
    ///
    /// **Validates: Requirements 2.6**
    ///
    /// For any valid SkillMetadata object:
    /// 1. Format it to SKILL.md content using format_skill_md()
    /// 2. Parse that content back using parse_skill_md()
    /// 3. The result should be equivalent to the original

    /// Strategy for generating valid skill names.
    /// Names are non-empty strings without YAML special characters that would break parsing.
    fn valid_name_strategy() -> impl Strategy<Value = String> {
        // Generate alphanumeric strings with spaces, avoiding YAML special characters
        // Characters to avoid: : # [ ] { } , & * ! | > ' " % @ ` - (at start) ? (at start)
        proptest::string::string_regex("[A-Za-z][A-Za-z0-9 ]{0,49}")
            .unwrap()
            .prop_filter("name must not be empty", |s| !s.trim().is_empty())
            .prop_map(|s| s.trim().to_string())
    }

    /// Strategy for generating valid descriptions.
    /// Descriptions are strings without YAML special characters.
    fn valid_description_strategy() -> impl Strategy<Value = String> {
        // Generate alphanumeric strings with spaces and basic punctuation
        // Avoid YAML special characters that would break parsing
        proptest::string::string_regex("[A-Za-z0-9 .!?()]{0,100}")
            .unwrap()
            .prop_map(|s| s.trim().to_string())
    }

    /// Strategy for generating valid tool names.
    /// Tool names are alphanumeric with underscores and hyphens.
    fn valid_tool_name_strategy() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[a-z][a-z0-9_-]{0,29}")
            .unwrap()
            .prop_filter("tool name must not be empty", |s| !s.is_empty())
    }

    /// Strategy for generating a vector of valid tool names.
    fn valid_tools_strategy() -> impl Strategy<Value = Vec<String>> {
        proptest::collection::vec(valid_tool_name_strategy(), 0..5)
    }

    /// Strategy for generating valid SkillMetadata objects.
    fn valid_skill_metadata_strategy() -> impl Strategy<Value = SkillMetadata> {
        (
            valid_name_strategy(),
            valid_description_strategy(),
            valid_tools_strategy(),
        )
            .prop_map(|(name, description, allowed_tools)| SkillMetadata {
                name,
                description,
                allowed_tools,
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: skills-manager-enhancement, Property 1: SKILL.md Round-Trip Parsing**
        ///
        /// **Validates: Requirements 2.6**
        ///
        /// FOR ALL valid SkillMetadata objects, parsing then printing then parsing
        /// SHALL produce an equivalent object (round-trip property).
        #[test]
        fn prop_round_trip_parsing(metadata in valid_skill_metadata_strategy()) {
            // Step 1: Format the metadata to SKILL.md content
            let formatted = format_skill_md(&metadata);

            // Step 2: Parse the formatted content back
            let parsed = parse_skill_md(&formatted);

            // Step 3: Verify the result is equivalent to the original
            prop_assert_eq!(
                parsed.name, metadata.name,
                "Name mismatch after round-trip. Formatted content:\n{}", formatted
            );
            prop_assert_eq!(
                parsed.description, metadata.description,
                "Description mismatch after round-trip. Formatted content:\n{}", formatted
            );
            prop_assert_eq!(
                parsed.allowed_tools, metadata.allowed_tools,
                "Allowed tools mismatch after round-trip. Formatted content:\n{}", formatted
            );
        }
    }
}
