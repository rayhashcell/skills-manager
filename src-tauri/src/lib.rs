use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

pub mod skill_parser;

pub use skill_parser::{parse_skill_md, SkillMetadata};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub path: String, // Relative to home, e.g., ".cursor/skills"
    pub detected: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Skill {
    pub name: String,           // Directory name
    pub metadata: SkillMetadata,
    pub linked_agents: Vec<String>, // List of agent IDs with this skill installed (symlink OR local)
    pub symlinked_agents: Vec<String>, // List of agent IDs with this skill linked via symlink only
}

/// Status of a skill in an agent's directory
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentSkillStatus {
    /// Skill is linked via symlink from global skills
    Symlink,
    /// Skill is stored locally (not a symlink)
    Local,
    /// Skill is not installed
    NotInstalled,
}

/// Represents a skill as seen from an agent's perspective
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AgentSkill {
    /// Directory name of the skill
    pub name: String,
    /// Metadata parsed from SKILL.md
    pub metadata: SkillMetadata,
    /// Status: symlink, local, or not_installed
    pub status: AgentSkillStatus,
    /// Source path (symlink target or local path), None if not installed
    pub source_path: Option<String>,
    /// Whether this skill exists in global skills directory
    pub in_global: bool,
}

/// Data for agent detail page
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AgentDetailData {
    pub agent: Agent,
    pub skills: Vec<AgentSkill>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AppData {
    pub agents: Vec<Agent>,
    pub skills: Vec<Skill>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BatchResult {
    pub success: Vec<String>,     // Agent IDs that succeeded
    pub failed: Vec<FailedOperation>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FailedOperation {
    pub agent_id: String,
    pub error: String,
}

fn get_home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()))
}

fn get_global_skills_path() -> PathBuf {
    get_home_dir().join(".agents/skills")
}

/// Returns the list of agent definitions (id, name, relative_path)
pub fn get_agent_definition_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("amp", "Amp", ".config/agents/skills"),
        ("antigravity", "Antigravity", ".gemini/antigravity/global_skills"),
        ("claude-code", "Claude Code", ".claude/skills"),
        ("clawdbot", "Clawdbot", ".clawdbot/skills"),
        ("cline", "Cline", ".cline/skills"),
        ("codex", "Codex", ".codex/skills"),
        ("command-code", "Command Code", ".commandcode/skills"),
        ("continue", "Continue", ".continue/skills"),
        ("crush", "Crush", ".config/crush/skills"),
        ("cursor", "Cursor", ".cursor/skills"),
        ("droid", "Droid", ".factory/skills"),
        ("gemini-cli", "Gemini CLI", ".gemini/skills"),
        ("github-copilot", "GitHub Copilot", ".copilot/skills"),
        ("goose", "Goose", ".config/goose/skills"),
        ("kilo-code", "Kilo Code", ".kilocode/skills"),
        ("kiro-cli", "Kiro CLI", ".kiro/skills"),
        ("mcpjam", "MCPJam", ".mcpjam/skills"),
        ("opencode", "OpenCode", ".config/opencode/skills"),
        ("openhands", "OpenHands", ".openhands/skills"),
        ("pi", "Pi", ".pi/agent/skills"),
        ("qoder", "Qoder", ".qoder/skills"),
        ("qwen-code", "Qwen Code", ".qwen/skills"),
        ("roo-code", "Roo Code", ".roo/skills"),
        ("trae", "Trae", ".trae/skills"),
        ("windsurf", "Windsurf", ".codeium/windsurf/skills"),
        ("zencoder", "Zencoder", ".zencoder/skills"),
        ("neovate", "Neovate", ".neovate/skills"),
    ]
}

/// Detects agents based on whether their skills directory exists.
/// This function is testable by accepting a custom home directory.
/// 
/// Requirements: 3.2, 3.3
/// - 3.2: WHEN detecting agents, THE Skills_Manager SHALL check if each agent's skills directory exists
/// - 3.3: WHEN an agent's skills directory does not exist, THE Skills_Manager SHALL mark the agent as not detected
pub fn detect_agents_with_home(home: &PathBuf) -> Vec<Agent> {
    get_agent_definition_list()
        .into_iter()
        .map(|(id, name, rel_path)| {
            let full_path = home.join(rel_path);
            Agent {
                id: id.to_string(),
                name: name.to_string(),
                path: rel_path.to_string(),
                detected: full_path.exists(),
            }
        })
        .collect()
}

fn get_agent_definitions() -> Vec<Agent> {
    let home = get_home_dir();
    detect_agents_with_home(&home)
}

/// Loads skill metadata from a skill directory.
/// 
/// Requirements: 1.6, 2.1
/// - 2.1: WHEN reading a skill directory, THE Skills_Manager SHALL look for a SKILL.md file in the skill's root directory
/// - 1.6: IF parsing SKILL.md fails, THEN THE Skills_Manager SHALL display the skill name from the directory name and show "No description available"
pub fn load_skill_metadata(skill_dir: &std::path::Path, dir_name: &str) -> SkillMetadata {
    let skill_md_path = skill_dir.join("SKILL.md");
    
    if skill_md_path.exists() {
        // Try to read and parse the SKILL.md file
        match fs::read_to_string(&skill_md_path) {
            Ok(content) => {
                let mut parsed = parse_skill_md(&content);
                // If name is empty after parsing, use directory name as fallback
                if parsed.name.is_empty() {
                    parsed.name = dir_name.to_string();
                }
                // If description is empty after parsing, use fallback
                if parsed.description.is_empty() {
                    parsed.description = "No description available".to_string();
                }
                parsed
            }
            Err(_) => {
                // Read failed, use fallback values
                SkillMetadata {
                    name: dir_name.to_string(),
                    description: "No description available".to_string(),
                    allowed_tools: Vec::new(),
                }
            }
        }
    } else {
        // SKILL.md not found, use fallback values
        SkillMetadata {
            name: dir_name.to_string(),
            description: "No description available".to_string(),
            allowed_tools: Vec::new(),
        }
    }
}

/// Gets app data with a custom home directory for testing.
/// 
/// Requirements: 1.6, 2.1
/// - 2.1: WHEN reading a skill directory, THE Skills_Manager SHALL look for a SKILL.md file in the skill's root directory
/// - 1.6: IF parsing SKILL.md fails, THEN THE Skills_Manager SHALL display the skill name from the directory name and show "No description available"
pub fn get_app_data_with_home(home: &PathBuf) -> AppData {
    let agents = detect_agents_with_home(home);
    let global_skills_path = home.join(".agents/skills");

    let mut skills = Vec::new();

    if let Ok(entries) = fs::read_dir(&global_skills_path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    // Skip hidden directories (starting with ".")
                    if name.starts_with('.') {
                        continue;
                    }
                    
                    let mut linked_agents = Vec::new();
                    let mut symlinked_agents = Vec::new();

                    // Check which agents have this skill installed (symlink OR local)
                    for agent in &agents {
                        if !agent.detected {
                            continue;
                        }
                        let agent_skill_path = home.join(&agent.path).join(&name);
                        
                        // Check if it exists as symlink OR local directory
                        if let Ok(metadata) = fs::symlink_metadata(&agent_skill_path) {
                            let file_type = metadata.file_type();
                            if file_type.is_symlink() {
                                linked_agents.push(agent.id.clone());
                                symlinked_agents.push(agent.id.clone());
                            } else if file_type.is_dir() {
                                linked_agents.push(agent.id.clone());
                            }
                        }
                    }

                    // Parse SKILL.md file for metadata
                    let metadata = load_skill_metadata(&entry.path(), &name);

                    skills.push(Skill {
                        name,
                        metadata,
                        linked_agents,
                        symlinked_agents,
                    });
                }
            }
        }
    }

    AppData { agents, skills }
}

#[tauri::command]
fn get_app_data() -> AppData {
    let home = get_home_dir();
    get_app_data_with_home(&home)
}

#[tauri::command]
fn toggle_skill(agent_id: String, skill_name: String, enable: bool) -> Result<(), String> {
    let agents = get_agent_definitions();
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or("Agent not found")?;
    
    let home = get_home_dir();
    let global_skill_path = get_global_skills_path().join(&skill_name);
    let agent_skill_path = home.join(&agent.path).join(&skill_name);

    if enable {
        if !global_skill_path.exists() {
            return Err("Global skill does not exist".to_string());
        }
        
        // Create parent dir if needed
        if let Some(parent) = agent_skill_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Create symlink
        // Note: For VS Code extensions, specific structure might be needed, but sticking to direct link for now
        symlink(&global_skill_path, &agent_skill_path)
            .map_err(|e| format!("Failed to link: {}", e))?;
    } else {
        // Remove symlink
        if agent_skill_path.exists() || fs::symlink_metadata(&agent_skill_path).is_ok() {
             fs::remove_file(&agent_skill_path)
                .map_err(|e| format!("Failed to unlink: {}", e))?;
        }
    }

    Ok(())
}

/// Links a skill to all detected agents by creating symlinks.
/// 
/// Requirements: 1.4, 6.1, 6.3
/// - 1.4: WHEN the user clicks "Link to All" on a skill card, THE Skills_Manager SHALL create symlinks for that skill in all detected agents' skills directories
/// - 6.1: WHEN the user clicks "Link to All Agents" for a skill, THE Skills_Manager SHALL create symlinks in all detected agents' skills directories
/// - 6.3: WHEN performing batch operations, THE Skills_Manager SHALL skip agents that are not detected
pub fn link_skill_to_all_with_home(skill_name: &str, home: &PathBuf) -> Result<BatchResult, String> {
    let agents = detect_agents_with_home(home);
    let global_skill_path = home.join(".agents/skills").join(skill_name);
    
    // Verify the global skill exists
    if !global_skill_path.exists() {
        return Err(format!("Global skill '{}' does not exist", skill_name));
    }
    
    let mut success: Vec<String> = Vec::new();
    let mut failed: Vec<FailedOperation> = Vec::new();
    
    for agent in agents {
        // Skip non-detected agents (Requirement 6.3)
        if !agent.detected {
            continue;
        }
        
        let agent_skill_path = home.join(&agent.path).join(skill_name);
        
        // Check if symlink already exists
        if let Ok(metadata) = fs::symlink_metadata(&agent_skill_path) {
            if metadata.file_type().is_symlink() {
                // Already linked, count as success
                success.push(agent.id);
                continue;
            } else {
                // A file or directory exists at the target path that is not a symlink
                failed.push(FailedOperation {
                    agent_id: agent.id,
                    error: "A file or directory already exists at the target path".to_string(),
                });
                continue;
            }
        }
        
        // Create parent directory if needed (Requirement 5.7)
        if let Some(parent) = agent_skill_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                failed.push(FailedOperation {
                    agent_id: agent.id,
                    error: format!("Failed to create parent directory: {}", e),
                });
                continue;
            }
        }
        
        // Create symlink
        match symlink(&global_skill_path, &agent_skill_path) {
            Ok(_) => {
                success.push(agent.id);
            }
            Err(e) => {
                failed.push(FailedOperation {
                    agent_id: agent.id,
                    error: format!("Failed to create symlink: {}", e),
                });
            }
        }
    }
    
    Ok(BatchResult { success, failed })
}

#[tauri::command]
fn link_skill_to_all(skill_name: String) -> Result<BatchResult, String> {
    let home = get_home_dir();
    link_skill_to_all_with_home(&skill_name, &home)
}

/// Unlinks a skill from all agents by removing symlinks.
/// Unlike link_skill_to_all, this attempts to remove symlinks from ALL agents
/// (not just detected ones) to ensure cleanup.
/// 
/// Requirements: 1.5, 6.2
/// - 1.5: WHEN the user clicks "Unlink from All" on a skill card, THE Skills_Manager SHALL remove symlinks for that skill from all agents' skills directories
/// - 6.2: WHEN the user clicks "Unlink from All Agents" for a skill, THE Skills_Manager SHALL remove symlinks from all agents' skills directories
pub fn unlink_skill_from_all_with_home(skill_name: &str, home: &PathBuf) -> Result<BatchResult, String> {
    let agent_definitions = get_agent_definition_list();
    
    let mut success: Vec<String> = Vec::new();
    let mut failed: Vec<FailedOperation> = Vec::new();
    
    for (id, _name, rel_path) in agent_definitions {
        let agent_skill_path = home.join(rel_path).join(skill_name);
        
        // Check if symlink exists at agent's skills directory
        match fs::symlink_metadata(&agent_skill_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    // Symlink exists, try to remove it
                    match fs::remove_file(&agent_skill_path) {
                        Ok(_) => {
                            success.push(id.to_string());
                        }
                        Err(e) => {
                            failed.push(FailedOperation {
                                agent_id: id.to_string(),
                                error: format!("Failed to remove symlink: {}", e),
                            });
                        }
                    }
                }
                // If it exists but is not a symlink, we don't touch it (not our symlink)
            }
            Err(_) => {
                // Path doesn't exist or can't be accessed - nothing to unlink
                // This is not a failure, just means there's no symlink to remove
            }
        }
    }
    
    Ok(BatchResult { success, failed })
}

#[tauri::command]
fn unlink_skill_from_all(skill_name: String) -> Result<BatchResult, String> {
    let home = get_home_dir();
    unlink_skill_from_all_with_home(&skill_name, &home)
}

/// Gets detailed skill information for a specific agent.
/// This includes both global skills and local-only skills in the agent's directory.
pub fn get_agent_detail_with_home(agent_id: &str, home: &PathBuf) -> Result<AgentDetailData, String> {
    let agents = detect_agents_with_home(home);
    let agent = agents.into_iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;
    
    let global_skills_path = home.join(".agents/skills");
    let agent_skills_path = home.join(&agent.path);
    
    // Collect global skill names (excluding hidden directories)
    let mut global_skill_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(entries) = fs::read_dir(&global_skills_path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('.') {
                        global_skill_names.insert(name);
                    }
                }
            }
        }
    }
    
    let mut skills: Vec<AgentSkill> = Vec::new();
    let mut seen_skills: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    // First, scan agent's skills directory for installed skills (symlinks and local)
    if agent.detected {
        if let Ok(entries) = fs::read_dir(&agent_skills_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip hidden directories
                if name.starts_with('.') {
                    continue;
                }
                
                if seen_skills.contains(&name) {
                    continue;
                }
                
                if let Ok(metadata) = fs::symlink_metadata(entry.path()) {
                    let file_type = metadata.file_type();
                    
                    if file_type.is_symlink() {
                        // It's a symlink - get the target
                        let target = fs::read_link(entry.path())
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| "unknown".to_string());
                        
                        // Load metadata from the symlink target
                        let skill_metadata = if let Ok(resolved) = fs::canonicalize(entry.path()) {
                            load_skill_metadata(&resolved, &name)
                        } else {
                            load_skill_metadata(&entry.path(), &name)
                        };
                        
                        skills.push(AgentSkill {
                            name: name.clone(),
                            metadata: skill_metadata,
                            status: AgentSkillStatus::Symlink,
                            source_path: Some(target),
                            in_global: global_skill_names.contains(&name),
                        });
                        seen_skills.insert(name);
                    } else if file_type.is_dir() {
                        // It's a local directory (not a symlink)
                        let skill_metadata = load_skill_metadata(&entry.path(), &name);
                        let local_path = entry.path().to_string_lossy().to_string();
                        
                        skills.push(AgentSkill {
                            name: name.clone(),
                            metadata: skill_metadata,
                            status: AgentSkillStatus::Local,
                            source_path: Some(local_path),
                            in_global: global_skill_names.contains(&name),
                        });
                        seen_skills.insert(name);
                    }
                }
            }
        }
    }
    
    // Then, add global skills that are not installed
    for global_name in &global_skill_names {
        if !seen_skills.contains(global_name) {
            let global_skill_path = global_skills_path.join(global_name);
            let skill_metadata = load_skill_metadata(&global_skill_path, global_name);
            
            skills.push(AgentSkill {
                name: global_name.clone(),
                metadata: skill_metadata,
                status: AgentSkillStatus::NotInstalled,
                source_path: None,
                in_global: true,
            });
        }
    }
    
    // Sort skills by name
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(AgentDetailData { agent, skills })
}

#[tauri::command]
fn get_agent_detail(agent_id: String) -> Result<AgentDetailData, String> {
    let home = get_home_dir();
    get_agent_detail_with_home(&agent_id, &home)
}

/// Deletes a local skill directory (not a symlink) from an agent's skills directory.
#[tauri::command]
fn delete_local_skill(agent_id: String, skill_name: String) -> Result<(), String> {
    let agents = get_agent_definitions();
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or("Agent not found")?;
    
    let home = get_home_dir();
    let skill_path = home.join(&agent.path).join(&skill_name);
    
    // Check if it exists and is NOT a symlink
    match fs::symlink_metadata(&skill_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err("Cannot delete: this is a symlink, use unlink instead".to_string());
            }
            if metadata.file_type().is_dir() {
                fs::remove_dir_all(&skill_path)
                    .map_err(|e| format!("Failed to delete directory: {}", e))?;
                Ok(())
            } else {
                Err("Path is not a directory".to_string())
            }
        }
        Err(_) => Err("Skill directory not found".to_string()),
    }
}

/// Recursively copies a directory and its contents
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    let entries = fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    
    Ok(())
}

/// Uploads a local skill from an agent's directory to the global skills directory.
#[tauri::command]
fn upload_to_global(agent_id: String, skill_name: String) -> Result<(), String> {
    let agents = get_agent_definitions();
    let agent = agents.iter().find(|a| a.id == agent_id).ok_or("Agent not found")?;
    
    let home = get_home_dir();
    let local_skill_path = home.join(&agent.path).join(&skill_name);
    let global_skill_path = get_global_skills_path().join(&skill_name);
    
    // Check if local skill exists and is NOT a symlink
    match fs::symlink_metadata(&local_skill_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err("Cannot upload: this is already a symlink".to_string());
            }
            if !metadata.file_type().is_dir() {
                return Err("Path is not a directory".to_string());
            }
        }
        Err(_) => return Err("Local skill directory not found".to_string()),
    }
    
    // Check if global skill already exists
    if global_skill_path.exists() {
        return Err(format!("Skill '{}' already exists in global skills", skill_name));
    }
    
    // Create global skills directory if it doesn't exist
    let global_skills_dir = get_global_skills_path();
    if !global_skills_dir.exists() {
        fs::create_dir_all(&global_skills_dir)
            .map_err(|e| format!("Failed to create global skills directory: {}", e))?;
    }
    
    // Copy the skill directory to global
    copy_dir_recursive(&local_skill_path, &global_skill_path)?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_data, 
            toggle_skill, 
            link_skill_to_all, 
            unlink_skill_from_all,
            get_agent_detail,
            delete_local_skill,
            upload_to_global
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a temporary home directory for testing
    fn create_temp_home() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    /// Test that agent is marked as detected when directory exists
    /// 
    /// **Validates: Requirements 3.2**
    /// - 3.2: WHEN detecting agents, THE Skills_Manager SHALL check if each agent's skills directory exists
    #[test]
    fn test_agent_detected_when_directory_exists() {
        // Arrange: Create a temp home directory with a cursor skills directory
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create the cursor skills directory
        let cursor_skills_path = home_path.join(".cursor/skills");
        fs::create_dir_all(&cursor_skills_path).expect("Failed to create cursor skills directory");
        
        // Act: Detect agents
        let agents = detect_agents_with_home(&home_path);
        
        // Assert: Cursor agent should be detected
        let cursor_agent = agents.iter().find(|a| a.id == "cursor").expect("Cursor agent not found");
        assert!(cursor_agent.detected, "Cursor agent should be detected when directory exists");
        assert_eq!(cursor_agent.path, ".cursor/skills");
        assert_eq!(cursor_agent.name, "Cursor");
    }

    /// Test that agent is marked as not detected when directory doesn't exist
    /// 
    /// **Validates: Requirements 3.3**
    /// - 3.3: WHEN an agent's skills directory does not exist, THE Skills_Manager SHALL mark the agent as not detected
    #[test]
    fn test_agent_not_detected_when_directory_does_not_exist() {
        // Arrange: Create an empty temp home directory (no agent directories)
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Act: Detect agents
        let agents = detect_agents_with_home(&home_path);
        
        // Assert: All agents should be not detected
        for agent in &agents {
            assert!(!agent.detected, "Agent {} should not be detected when directory doesn't exist", agent.id);
        }
        
        // Verify we have all 27 agents
        assert_eq!(agents.len(), 27, "Should have 27 agent definitions");
    }

    /// Test detection for multiple agents with mixed existence states
    /// 
    /// **Validates: Requirements 3.2, 3.3**
    /// - 3.2: WHEN detecting agents, THE Skills_Manager SHALL check if each agent's skills directory exists
    /// - 3.3: WHEN an agent's skills directory does not exist, THE Skills_Manager SHALL mark the agent as not detected
    #[test]
    fn test_mixed_agent_detection_states() {
        // Arrange: Create a temp home directory with some agent directories
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create directories for specific agents
        let agents_to_create = vec![
            ".cursor/skills",           // cursor
            ".claude/skills",           // claude-code
            ".config/agents/skills",    // amp
            ".gemini/skills",           // gemini-cli
        ];
        
        for path in &agents_to_create {
            let full_path = home_path.join(path);
            fs::create_dir_all(&full_path).expect(&format!("Failed to create directory: {}", path));
        }
        
        // Act: Detect agents
        let agents = detect_agents_with_home(&home_path);
        
        // Assert: Check specific agents are detected
        let detected_ids = vec!["cursor", "claude-code", "amp", "gemini-cli"];
        let not_detected_ids = vec!["cline", "codex", "neovate", "windsurf"];
        
        for id in detected_ids {
            let agent = agents.iter().find(|a| a.id == id).expect(&format!("Agent {} not found", id));
            assert!(agent.detected, "Agent {} should be detected", id);
        }
        
        for id in not_detected_ids {
            let agent = agents.iter().find(|a| a.id == id).expect(&format!("Agent {} not found", id));
            assert!(!agent.detected, "Agent {} should not be detected", id);
        }
        
        // Verify total count
        assert_eq!(agents.len(), 27, "Should have 27 agent definitions");
        
        // Count detected vs not detected
        let detected_count = agents.iter().filter(|a| a.detected).count();
        let not_detected_count = agents.iter().filter(|a| !a.detected).count();
        
        assert_eq!(detected_count, 4, "Should have 4 detected agents");
        assert_eq!(not_detected_count, 23, "Should have 23 not detected agents");
    }

    /// Test that agent definitions contain correct data
    #[test]
    fn test_agent_definitions_are_complete() {
        let definitions = get_agent_definition_list();
        
        // Verify we have all 27 agents
        assert_eq!(definitions.len(), 27, "Should have 27 agent definitions");
        
        // Verify each definition has non-empty values
        for (id, name, path) in &definitions {
            assert!(!id.is_empty(), "Agent ID should not be empty");
            assert!(!name.is_empty(), "Agent name should not be empty");
            assert!(!path.is_empty(), "Agent path should not be empty");
        }
        
        // Verify specific agents exist with correct paths
        let expected_agents = vec![
            ("cursor", "Cursor", ".cursor/skills"),
            ("claude-code", "Claude Code", ".claude/skills"),
            ("windsurf", "Windsurf", ".codeium/windsurf/skills"),
            ("pi", "Pi", ".pi/agent/skills"),
        ];
        
        for (expected_id, expected_name, expected_path) in expected_agents {
            let found = definitions.iter().find(|(id, _, _)| *id == expected_id);
            assert!(found.is_some(), "Agent {} should exist", expected_id);
            let (_, name, path) = found.unwrap();
            assert_eq!(*name, expected_name, "Agent {} should have correct name", expected_id);
            assert_eq!(*path, expected_path, "Agent {} should have correct path", expected_id);
        }
    }

    /// Test detection with nested directory paths
    /// 
    /// **Validates: Requirements 3.2**
    #[test]
    fn test_agent_detection_with_nested_paths() {
        // Arrange: Create a temp home directory
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create agents with deeply nested paths
        let nested_agents = vec![
            (".gemini/antigravity/global_skills", "antigravity"),  // 3 levels deep
            (".codeium/windsurf/skills", "windsurf"),              // 3 levels deep
            (".pi/agent/skills", "pi"),                            // 3 levels deep
        ];
        
        for (path, _) in &nested_agents {
            let full_path = home_path.join(path);
            fs::create_dir_all(&full_path).expect(&format!("Failed to create directory: {}", path));
        }
        
        // Act: Detect agents
        let agents = detect_agents_with_home(&home_path);
        
        // Assert: Nested path agents should be detected
        for (_, id) in &nested_agents {
            let agent = agents.iter().find(|a| a.id == *id).expect(&format!("Agent {} not found", id));
            assert!(agent.detected, "Agent {} with nested path should be detected", id);
        }
    }

    // ==================== SKILL.md Parsing Tests ====================

    /// Test that get_app_data parses SKILL.md files with frontmatter format
    /// 
    /// **Validates: Requirements 2.1**
    /// - 2.1: WHEN reading a skill directory, THE Skills_Manager SHALL look for a SKILL.md file in the skill's root directory
    #[test]
    fn test_get_app_data_parses_skill_md_frontmatter() {
        // Arrange: Create a temp home directory with a skill containing SKILL.md
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skills directory with a skill
        let skill_dir = home_path.join(".agents/skills/my-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create SKILL.md with frontmatter
        let skill_md_content = r#"---
name: My Awesome Skill
description: This skill does amazing things
allowed-tools:
  - tool1
  - tool2
---

# My Awesome Skill

Detailed documentation here.
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_md_content).expect("Failed to write SKILL.md");
        
        // Act: Get app data
        let app_data = get_app_data_with_home(&home_path);
        
        // Assert: Skill should have parsed metadata
        assert_eq!(app_data.skills.len(), 1, "Should have 1 skill");
        let skill = &app_data.skills[0];
        assert_eq!(skill.name, "my-skill", "Skill directory name should be preserved");
        assert_eq!(skill.metadata.name, "My Awesome Skill", "Skill name should be parsed from frontmatter");
        assert_eq!(skill.metadata.description, "This skill does amazing things", "Description should be parsed from frontmatter");
        assert_eq!(skill.metadata.allowed_tools, vec!["tool1", "tool2"], "Allowed tools should be parsed from frontmatter");
    }

    /// Test that get_app_data parses SKILL.md files with heading format
    /// 
    /// **Validates: Requirements 2.1**
    #[test]
    fn test_get_app_data_parses_skill_md_heading_format() {
        // Arrange: Create a temp home directory with a skill containing SKILL.md
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skills directory with a skill
        let skill_dir = home_path.join(".agents/skills/heading-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create SKILL.md with heading format
        let skill_md_content = r#"# Heading Based Skill

This is a skill using the heading format.

## Allowed Tools
- read_file
- write_file
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_md_content).expect("Failed to write SKILL.md");
        
        // Act: Get app data
        let app_data = get_app_data_with_home(&home_path);
        
        // Assert: Skill should have parsed metadata
        assert_eq!(app_data.skills.len(), 1, "Should have 1 skill");
        let skill = &app_data.skills[0];
        assert_eq!(skill.metadata.name, "Heading Based Skill", "Skill name should be parsed from heading");
        assert_eq!(skill.metadata.description, "This is a skill using the heading format.", "Description should be parsed from first paragraph");
        assert_eq!(skill.metadata.allowed_tools, vec!["read_file", "write_file"], "Allowed tools should be parsed");
    }

    /// Test that get_app_data uses fallback values when SKILL.md is missing
    /// 
    /// **Validates: Requirements 1.6**
    /// - 1.6: IF parsing SKILL.md fails, THEN THE Skills_Manager SHALL display the skill name from the directory name and show "No description available"
    #[test]
    fn test_get_app_data_fallback_when_skill_md_missing() {
        // Arrange: Create a temp home directory with a skill without SKILL.md
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skills directory with a skill (no SKILL.md)
        let skill_dir = home_path.join(".agents/skills/no-skill-md");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Act: Get app data
        let app_data = get_app_data_with_home(&home_path);
        
        // Assert: Skill should have fallback metadata
        assert_eq!(app_data.skills.len(), 1, "Should have 1 skill");
        let skill = &app_data.skills[0];
        assert_eq!(skill.name, "no-skill-md", "Skill directory name should be preserved");
        assert_eq!(skill.metadata.name, "no-skill-md", "Skill name should fallback to directory name");
        assert_eq!(skill.metadata.description, "No description available", "Description should be fallback message");
        assert!(skill.metadata.allowed_tools.is_empty(), "Allowed tools should be empty");
    }

    /// Test that get_app_data uses fallback values when SKILL.md has empty name
    /// 
    /// **Validates: Requirements 1.6**
    #[test]
    fn test_get_app_data_fallback_when_skill_md_has_empty_name() {
        // Arrange: Create a temp home directory with a skill with empty name in SKILL.md
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skills directory with a skill
        let skill_dir = home_path.join(".agents/skills/empty-name-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create SKILL.md with empty name
        let skill_md_content = r#"---
description: Has description but no name
---
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_md_content).expect("Failed to write SKILL.md");
        
        // Act: Get app data
        let app_data = get_app_data_with_home(&home_path);
        
        // Assert: Skill should have directory name as fallback
        assert_eq!(app_data.skills.len(), 1, "Should have 1 skill");
        let skill = &app_data.skills[0];
        assert_eq!(skill.metadata.name, "empty-name-skill", "Skill name should fallback to directory name");
        assert_eq!(skill.metadata.description, "Has description but no name", "Description should be parsed");
    }

    /// Test that get_app_data uses fallback description when SKILL.md has empty description
    /// 
    /// **Validates: Requirements 1.6**
    #[test]
    fn test_get_app_data_fallback_when_skill_md_has_empty_description() {
        // Arrange: Create a temp home directory with a skill with empty description in SKILL.md
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skills directory with a skill
        let skill_dir = home_path.join(".agents/skills/empty-desc-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create SKILL.md with name but no description
        let skill_md_content = r#"---
name: Named Skill
---
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_md_content).expect("Failed to write SKILL.md");
        
        // Act: Get app data
        let app_data = get_app_data_with_home(&home_path);
        
        // Assert: Skill should have fallback description
        assert_eq!(app_data.skills.len(), 1, "Should have 1 skill");
        let skill = &app_data.skills[0];
        assert_eq!(skill.metadata.name, "Named Skill", "Skill name should be parsed");
        assert_eq!(skill.metadata.description, "No description available", "Description should be fallback message");
    }

    /// Test that get_app_data handles multiple skills with mixed SKILL.md states
    /// 
    /// **Validates: Requirements 1.6, 2.1**
    #[test]
    fn test_get_app_data_multiple_skills_mixed_states() {
        // Arrange: Create a temp home directory with multiple skills
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skills directory
        let skills_base = home_path.join(".agents/skills");
        
        // Skill 1: Has complete SKILL.md
        let skill1_dir = skills_base.join("complete-skill");
        fs::create_dir_all(&skill1_dir).expect("Failed to create skill1 directory");
        fs::write(skill1_dir.join("SKILL.md"), r#"---
name: Complete Skill
description: A fully documented skill
allowed-tools:
  - tool_a
---
"#).expect("Failed to write SKILL.md");
        
        // Skill 2: No SKILL.md
        let skill2_dir = skills_base.join("no-md-skill");
        fs::create_dir_all(&skill2_dir).expect("Failed to create skill2 directory");
        
        // Skill 3: Empty SKILL.md
        let skill3_dir = skills_base.join("empty-md-skill");
        fs::create_dir_all(&skill3_dir).expect("Failed to create skill3 directory");
        fs::write(skill3_dir.join("SKILL.md"), "").expect("Failed to write empty SKILL.md");
        
        // Act: Get app data
        let app_data = get_app_data_with_home(&home_path);
        
        // Assert: All skills should be loaded with appropriate metadata
        assert_eq!(app_data.skills.len(), 3, "Should have 3 skills");
        
        // Find each skill by name
        let complete_skill = app_data.skills.iter().find(|s| s.name == "complete-skill").expect("complete-skill not found");
        let no_md_skill = app_data.skills.iter().find(|s| s.name == "no-md-skill").expect("no-md-skill not found");
        let empty_md_skill = app_data.skills.iter().find(|s| s.name == "empty-md-skill").expect("empty-md-skill not found");
        
        // Verify complete skill
        assert_eq!(complete_skill.metadata.name, "Complete Skill");
        assert_eq!(complete_skill.metadata.description, "A fully documented skill");
        assert_eq!(complete_skill.metadata.allowed_tools, vec!["tool_a"]);
        
        // Verify no-md skill (fallback values)
        assert_eq!(no_md_skill.metadata.name, "no-md-skill");
        assert_eq!(no_md_skill.metadata.description, "No description available");
        assert!(no_md_skill.metadata.allowed_tools.is_empty());
        
        // Verify empty-md skill (fallback values)
        assert_eq!(empty_md_skill.metadata.name, "empty-md-skill");
        assert_eq!(empty_md_skill.metadata.description, "No description available");
        assert!(empty_md_skill.metadata.allowed_tools.is_empty());
    }

    /// Test load_skill_metadata function directly
    /// 
    /// **Validates: Requirements 1.6, 2.1**
    #[test]
    fn test_load_skill_metadata_with_valid_skill_md() {
        // Arrange: Create a temp directory with SKILL.md
        let temp_dir = create_temp_home();
        let skill_dir = temp_dir.path().to_path_buf();
        
        let skill_md_content = r#"---
name: Test Skill
description: Test description
allowed-tools:
  - test_tool
---
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_md_content).expect("Failed to write SKILL.md");
        
        // Act
        let metadata = load_skill_metadata(&skill_dir, "fallback-name");
        
        // Assert
        assert_eq!(metadata.name, "Test Skill");
        assert_eq!(metadata.description, "Test description");
        assert_eq!(metadata.allowed_tools, vec!["test_tool"]);
    }

    /// Test load_skill_metadata function with missing SKILL.md
    /// 
    /// **Validates: Requirements 1.6**
    #[test]
    fn test_load_skill_metadata_without_skill_md() {
        // Arrange: Create a temp directory without SKILL.md
        let temp_dir = create_temp_home();
        let skill_dir = temp_dir.path().to_path_buf();
        
        // Act
        let metadata = load_skill_metadata(&skill_dir, "my-fallback-name");
        
        // Assert: Should use fallback values
        assert_eq!(metadata.name, "my-fallback-name");
        assert_eq!(metadata.description, "No description available");
        assert!(metadata.allowed_tools.is_empty());
    }

    // ==================== link_skill_to_all Tests ====================

    /// Test that link_skill_to_all creates symlinks for all detected agents
    /// 
    /// **Validates: Requirements 1.4, 6.1**
    /// - 1.4: WHEN the user clicks "Link to All" on a skill card, THE Skills_Manager SHALL create symlinks for that skill in all detected agents' skills directories
    /// - 6.1: WHEN the user clicks "Link to All Agents" for a skill, THE Skills_Manager SHALL create symlinks in all detected agents' skills directories
    #[test]
    fn test_link_skill_to_all_creates_symlinks_for_detected_agents() {
        // Arrange: Create a temp home directory with a global skill and some detected agents
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        fs::write(skill_dir.join("SKILL.md"), "# Test Skill\nA test skill.").expect("Failed to write SKILL.md");
        
        // Create detected agent directories
        let detected_agents = vec![
            ".cursor/skills",
            ".claude/skills",
            ".config/agents/skills",  // amp
        ];
        
        for path in &detected_agents {
            let full_path = home_path.join(path);
            fs::create_dir_all(&full_path).expect(&format!("Failed to create directory: {}", path));
        }
        
        // Act: Link skill to all
        let result = link_skill_to_all_with_home("test-skill", &home_path).expect("link_skill_to_all should succeed");
        
        // Assert: All detected agents should be in success list
        assert_eq!(result.success.len(), 3, "Should have 3 successful links");
        assert!(result.success.contains(&"cursor".to_string()), "cursor should be in success list");
        assert!(result.success.contains(&"claude-code".to_string()), "claude-code should be in success list");
        assert!(result.success.contains(&"amp".to_string()), "amp should be in success list");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify symlinks were created
        for path in &detected_agents {
            let symlink_path = home_path.join(path).join("test-skill");
            assert!(symlink_path.exists(), "Symlink should exist at {}", symlink_path.display());
            let metadata = fs::symlink_metadata(&symlink_path).expect("Should be able to read symlink metadata");
            assert!(metadata.file_type().is_symlink(), "Should be a symlink");
        }
    }

    /// Test that link_skill_to_all skips non-detected agents
    /// 
    /// **Validates: Requirements 6.3**
    /// - 6.3: WHEN performing batch operations, THE Skills_Manager SHALL skip agents that are not detected
    #[test]
    fn test_link_skill_to_all_skips_non_detected_agents() {
        // Arrange: Create a temp home directory with a global skill and only one detected agent
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create only one detected agent directory
        let cursor_path = home_path.join(".cursor/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        
        // Act: Link skill to all
        let result = link_skill_to_all_with_home("test-skill", &home_path).expect("link_skill_to_all should succeed");
        
        // Assert: Only cursor should be in success list (other 26 agents are not detected)
        assert_eq!(result.success.len(), 1, "Should have 1 successful link");
        assert!(result.success.contains(&"cursor".to_string()), "cursor should be in success list");
        assert!(result.failed.is_empty(), "Should have no failures (non-detected agents are skipped, not failed)");
        
        // Verify symlink was created for cursor
        let symlink_path = cursor_path.join("test-skill");
        assert!(symlink_path.exists(), "Symlink should exist for cursor");
        
        // Verify no symlinks were created for non-detected agents
        let claude_symlink = home_path.join(".claude/skills/test-skill");
        assert!(!claude_symlink.exists(), "Symlink should not exist for non-detected agent");
    }

    /// Test that link_skill_to_all returns error when skill doesn't exist
    /// 
    /// **Validates: Requirements 1.4**
    #[test]
    fn test_link_skill_to_all_fails_when_skill_not_found() {
        // Arrange: Create a temp home directory without the skill
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skills directory but not the specific skill
        fs::create_dir_all(home_path.join(".agents/skills")).expect("Failed to create skills directory");
        
        // Create a detected agent
        fs::create_dir_all(home_path.join(".cursor/skills")).expect("Failed to create cursor directory");
        
        // Act: Try to link non-existent skill
        let result = link_skill_to_all_with_home("non-existent-skill", &home_path);
        
        // Assert: Should return error
        assert!(result.is_err(), "Should return error when skill doesn't exist");
        let error = result.unwrap_err();
        assert!(error.contains("non-existent-skill"), "Error should mention the skill name");
    }

    /// Test that link_skill_to_all handles already linked skills
    /// 
    /// **Validates: Requirements 1.4, 6.1**
    #[test]
    fn test_link_skill_to_all_handles_already_linked_skills() {
        // Arrange: Create a temp home directory with a global skill and a pre-existing symlink
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create detected agent directories
        let cursor_path = home_path.join(".cursor/skills");
        let claude_path = home_path.join(".claude/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        fs::create_dir_all(&claude_path).expect("Failed to create claude directory");
        
        // Pre-create symlink for cursor
        let cursor_symlink = cursor_path.join("test-skill");
        symlink(&skill_dir, &cursor_symlink).expect("Failed to create pre-existing symlink");
        
        // Act: Link skill to all
        let result = link_skill_to_all_with_home("test-skill", &home_path).expect("link_skill_to_all should succeed");
        
        // Assert: Both agents should be in success list (cursor already linked, claude newly linked)
        assert_eq!(result.success.len(), 2, "Should have 2 successful links");
        assert!(result.success.contains(&"cursor".to_string()), "cursor should be in success list");
        assert!(result.success.contains(&"claude-code".to_string()), "claude-code should be in success list");
        assert!(result.failed.is_empty(), "Should have no failures");
    }

    /// Test that link_skill_to_all creates parent directories if needed
    /// 
    /// **Validates: Requirements 5.7**
    /// - 5.7: WHEN creating a symlink, THE Skills_Manager SHALL create the parent directory if it does not exist
    #[test]
    fn test_link_skill_to_all_creates_parent_directories() {
        // Arrange: Create a temp home directory with a global skill
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create only the base directory for cursor (not the full skills path)
        // This simulates an agent that exists but doesn't have a skills directory yet
        let cursor_base = home_path.join(".cursor");
        fs::create_dir_all(&cursor_base).expect("Failed to create cursor base directory");
        
        // Also create the full path for another agent to make it detected
        let claude_path = home_path.join(".claude/skills");
        fs::create_dir_all(&claude_path).expect("Failed to create claude directory");
        
        // Act: Link skill to all
        let result = link_skill_to_all_with_home("test-skill", &home_path).expect("link_skill_to_all should succeed");
        
        // Assert: Only claude should be in success list (cursor is not detected because .cursor/skills doesn't exist)
        assert_eq!(result.success.len(), 1, "Should have 1 successful link");
        assert!(result.success.contains(&"claude-code".to_string()), "claude-code should be in success list");
    }

    /// Test that link_skill_to_all returns BatchResult with correct structure
    /// 
    /// **Validates: Requirements 1.4, 6.1**
    #[test]
    fn test_link_skill_to_all_returns_batch_result() {
        // Arrange: Create a temp home directory with a global skill
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create detected agent directories
        fs::create_dir_all(home_path.join(".cursor/skills")).expect("Failed to create cursor directory");
        fs::create_dir_all(home_path.join(".claude/skills")).expect("Failed to create claude directory");
        
        // Act: Link skill to all
        let result = link_skill_to_all_with_home("test-skill", &home_path).expect("link_skill_to_all should succeed");
        
        // Assert: BatchResult should have correct structure
        assert!(result.success.len() >= 2, "Should have at least 2 successful links");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify success list contains agent IDs (strings)
        for agent_id in &result.success {
            assert!(!agent_id.is_empty(), "Agent ID should not be empty");
        }
    }

    /// Test that link_skill_to_all handles file existing at target path
    /// 
    /// **Validates: Requirements 6.5**
    /// - 6.5: IF any individual link/unlink operation fails during batch, THEN THE Skills_Manager SHALL continue with remaining agents and report errors
    #[test]
    fn test_link_skill_to_all_handles_file_at_target_path() {
        // Arrange: Create a temp home directory with a global skill
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create detected agent directories
        let cursor_path = home_path.join(".cursor/skills");
        let claude_path = home_path.join(".claude/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        fs::create_dir_all(&claude_path).expect("Failed to create claude directory");
        
        // Create a regular file (not symlink) at cursor's target path
        let cursor_target = cursor_path.join("test-skill");
        fs::write(&cursor_target, "blocking file").expect("Failed to create blocking file");
        
        // Act: Link skill to all
        let result = link_skill_to_all_with_home("test-skill", &home_path).expect("link_skill_to_all should succeed");
        
        // Assert: cursor should fail, claude should succeed
        assert!(result.success.contains(&"claude-code".to_string()), "claude-code should be in success list");
        assert_eq!(result.failed.len(), 1, "Should have 1 failure");
        assert_eq!(result.failed[0].agent_id, "cursor", "cursor should be in failed list");
        assert!(!result.failed[0].error.is_empty(), "Error message should not be empty");
    }

    // ==================== unlink_skill_from_all Tests ====================

    /// Test that unlink_skill_from_all removes symlinks from all agents
    /// 
    /// **Validates: Requirements 1.5, 6.2**
    /// - 1.5: WHEN the user clicks "Unlink from All" on a skill card, THE Skills_Manager SHALL remove symlinks for that skill from all agents' skills directories
    /// - 6.2: WHEN the user clicks "Unlink from All Agents" for a skill, THE Skills_Manager SHALL remove symlinks from all agents' skills directories
    #[test]
    fn test_unlink_skill_from_all_removes_symlinks() {
        // Arrange: Create a temp home directory with a global skill and symlinks
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create agent directories and symlinks
        let cursor_path = home_path.join(".cursor/skills");
        let claude_path = home_path.join(".claude/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        fs::create_dir_all(&claude_path).expect("Failed to create claude directory");
        
        // Create symlinks
        let cursor_symlink = cursor_path.join("test-skill");
        let claude_symlink = claude_path.join("test-skill");
        symlink(&skill_dir, &cursor_symlink).expect("Failed to create cursor symlink");
        symlink(&skill_dir, &claude_symlink).expect("Failed to create claude symlink");
        
        // Verify symlinks exist before unlink
        assert!(cursor_symlink.exists(), "Cursor symlink should exist before unlink");
        assert!(claude_symlink.exists(), "Claude symlink should exist before unlink");
        
        // Act: Unlink skill from all
        let result = unlink_skill_from_all_with_home("test-skill", &home_path).expect("unlink_skill_from_all should succeed");
        
        // Assert: Both agents should be in success list
        assert_eq!(result.success.len(), 2, "Should have 2 successful unlinks");
        assert!(result.success.contains(&"cursor".to_string()), "cursor should be in success list");
        assert!(result.success.contains(&"claude-code".to_string()), "claude-code should be in success list");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify symlinks were removed
        assert!(!cursor_symlink.exists(), "Cursor symlink should be removed");
        assert!(!claude_symlink.exists(), "Claude symlink should be removed");
    }

    /// Test that unlink_skill_from_all attempts to remove from ALL agents (not just detected)
    /// 
    /// **Validates: Requirements 1.5, 6.2**
    /// - Unlike link_skill_to_all, unlink should attempt to remove symlinks from ALL agents
    #[test]
    fn test_unlink_skill_from_all_removes_from_non_detected_agents() {
        // Arrange: Create a temp home directory with symlinks but agent not "detected"
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create only the symlink path (not the full agent skills directory)
        // This simulates a case where the agent was previously detected but now isn't
        let cursor_path = home_path.join(".cursor/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        
        // Create symlink
        let cursor_symlink = cursor_path.join("test-skill");
        symlink(&skill_dir, &cursor_symlink).expect("Failed to create cursor symlink");
        
        // Verify symlink exists
        assert!(cursor_symlink.exists(), "Cursor symlink should exist before unlink");
        
        // Act: Unlink skill from all
        let result = unlink_skill_from_all_with_home("test-skill", &home_path).expect("unlink_skill_from_all should succeed");
        
        // Assert: cursor should be in success list even though it might not be "detected"
        assert!(result.success.contains(&"cursor".to_string()), "cursor should be in success list");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify symlink was removed
        assert!(!cursor_symlink.exists(), "Cursor symlink should be removed");
    }

    /// Test that unlink_skill_from_all handles no existing symlinks gracefully
    /// 
    /// **Validates: Requirements 1.5, 6.2**
    #[test]
    fn test_unlink_skill_from_all_handles_no_symlinks() {
        // Arrange: Create a temp home directory with no symlinks
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create agent directories but no symlinks
        let cursor_path = home_path.join(".cursor/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        
        // Act: Unlink skill from all
        let result = unlink_skill_from_all_with_home("test-skill", &home_path).expect("unlink_skill_from_all should succeed");
        
        // Assert: Success list should be empty (nothing to unlink), no failures
        assert!(result.success.is_empty(), "Should have no successful unlinks (nothing to unlink)");
        assert!(result.failed.is_empty(), "Should have no failures");
    }

    /// Test that unlink_skill_from_all does not remove regular files (only symlinks)
    /// 
    /// **Validates: Requirements 1.5, 6.2**
    #[test]
    fn test_unlink_skill_from_all_ignores_regular_files() {
        // Arrange: Create a temp home directory with a regular file instead of symlink
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create agent directory with a regular file (not symlink)
        let cursor_path = home_path.join(".cursor/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        let cursor_file = cursor_path.join("test-skill");
        fs::write(&cursor_file, "regular file content").expect("Failed to create regular file");
        
        // Act: Unlink skill from all
        let result = unlink_skill_from_all_with_home("test-skill", &home_path).expect("unlink_skill_from_all should succeed");
        
        // Assert: Success list should be empty (regular file is not a symlink)
        assert!(result.success.is_empty(), "Should have no successful unlinks (file is not a symlink)");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify regular file still exists
        assert!(cursor_file.exists(), "Regular file should not be removed");
    }

    /// Test that unlink_skill_from_all does not remove directories (only symlinks)
    /// 
    /// **Validates: Requirements 1.5, 6.2**
    #[test]
    fn test_unlink_skill_from_all_ignores_directories() {
        // Arrange: Create a temp home directory with a directory instead of symlink
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create agent directory with a subdirectory (not symlink)
        let cursor_path = home_path.join(".cursor/skills");
        let cursor_subdir = cursor_path.join("test-skill");
        fs::create_dir_all(&cursor_subdir).expect("Failed to create cursor subdirectory");
        
        // Act: Unlink skill from all
        let result = unlink_skill_from_all_with_home("test-skill", &home_path).expect("unlink_skill_from_all should succeed");
        
        // Assert: Success list should be empty (directory is not a symlink)
        assert!(result.success.is_empty(), "Should have no successful unlinks (directory is not a symlink)");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify directory still exists
        assert!(cursor_subdir.exists(), "Directory should not be removed");
    }

    /// Test that unlink_skill_from_all returns BatchResult with correct structure
    /// 
    /// **Validates: Requirements 1.5, 6.2**
    #[test]
    fn test_unlink_skill_from_all_returns_batch_result() {
        // Arrange: Create a temp home directory with symlinks
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create agent directories and symlinks
        let cursor_path = home_path.join(".cursor/skills");
        let claude_path = home_path.join(".claude/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        fs::create_dir_all(&claude_path).expect("Failed to create claude directory");
        
        // Create symlinks
        symlink(&skill_dir, cursor_path.join("test-skill")).expect("Failed to create cursor symlink");
        symlink(&skill_dir, claude_path.join("test-skill")).expect("Failed to create claude symlink");
        
        // Act: Unlink skill from all
        let result = unlink_skill_from_all_with_home("test-skill", &home_path).expect("unlink_skill_from_all should succeed");
        
        // Assert: BatchResult should have correct structure
        assert_eq!(result.success.len(), 2, "Should have 2 successful unlinks");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify success list contains agent IDs (strings)
        for agent_id in &result.success {
            assert!(!agent_id.is_empty(), "Agent ID should not be empty");
        }
    }

    /// Test that unlink_skill_from_all handles mixed states (some symlinks, some not)
    /// 
    /// **Validates: Requirements 1.5, 6.2**
    #[test]
    fn test_unlink_skill_from_all_handles_mixed_states() {
        // Arrange: Create a temp home directory with mixed states
        let temp_home = create_temp_home();
        let home_path = temp_home.path().to_path_buf();
        
        // Create global skill
        let skill_dir = home_path.join(".agents/skills/test-skill");
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create agent directories
        let cursor_path = home_path.join(".cursor/skills");
        let claude_path = home_path.join(".claude/skills");
        let amp_path = home_path.join(".config/agents/skills");
        fs::create_dir_all(&cursor_path).expect("Failed to create cursor directory");
        fs::create_dir_all(&claude_path).expect("Failed to create claude directory");
        fs::create_dir_all(&amp_path).expect("Failed to create amp directory");
        
        // cursor: has symlink
        let cursor_symlink = cursor_path.join("test-skill");
        symlink(&skill_dir, &cursor_symlink).expect("Failed to create cursor symlink");
        
        // claude: has regular file
        let claude_file = claude_path.join("test-skill");
        fs::write(&claude_file, "regular file").expect("Failed to create claude file");
        
        // amp: no file at all
        
        // Act: Unlink skill from all
        let result = unlink_skill_from_all_with_home("test-skill", &home_path).expect("unlink_skill_from_all should succeed");
        
        // Assert: Only cursor should be in success list
        assert_eq!(result.success.len(), 1, "Should have 1 successful unlink");
        assert!(result.success.contains(&"cursor".to_string()), "cursor should be in success list");
        assert!(result.failed.is_empty(), "Should have no failures");
        
        // Verify cursor symlink was removed
        assert!(!cursor_symlink.exists(), "Cursor symlink should be removed");
        
        // Verify claude file still exists
        assert!(claude_file.exists(), "Claude regular file should not be removed");
    }
}


#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    // ==================== Generator Strategies ====================

    /// Strategy for generating valid skill names.
    /// Skill names are alphanumeric with underscores and hyphens.
    fn valid_skill_name_strategy() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[a-zA-Z][a-zA-Z0-9_-]{0,29}")
            .unwrap()
            .prop_filter("skill name must not be empty", |s| !s.is_empty())
    }

    /// Strategy for generating a random subset of agent indices.
    /// This determines which agents will be "detected" (have their directories created).
    fn agent_subset_strategy() -> impl Strategy<Value = Vec<usize>> {
        let num_agents = get_agent_definition_list().len();
        proptest::collection::vec(0..num_agents, 0..num_agents)
            .prop_map(|indices| {
                // Remove duplicates by converting to HashSet and back
                let unique: HashSet<usize> = indices.into_iter().collect();
                unique.into_iter().collect()
            })
    }

    // ==================== Test Helpers ====================

    /// Creates a temporary home directory for testing.
    fn create_temp_home() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    /// Creates the global skills directory and a skill within it.
    fn create_global_skill(home: &PathBuf, skill_name: &str) -> PathBuf {
        let skill_dir = home.join(".agents/skills").join(skill_name);
        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        
        // Create a minimal SKILL.md file
        let skill_md = format!("# {}\n\nA test skill.\n", skill_name);
        fs::write(skill_dir.join("SKILL.md"), skill_md).expect("Failed to write SKILL.md");
        
        skill_dir
    }

    /// Creates agent directories for the specified agent indices.
    /// Returns the list of agent IDs that were created (detected).
    fn create_agent_directories(home: &PathBuf, agent_indices: &[usize]) -> Vec<String> {
        let definitions = get_agent_definition_list();
        let mut created_ids = Vec::new();
        
        for &idx in agent_indices {
            if idx < definitions.len() {
                let (id, _, rel_path) = definitions[idx];
                let agent_path = home.join(rel_path);
                fs::create_dir_all(&agent_path).expect("Failed to create agent directory");
                created_ids.push(id.to_string());
            }
        }
        
        created_ids
    }

    /// Checks if a symlink exists at the given path.
    fn symlink_exists(path: &PathBuf) -> bool {
        if let Ok(metadata) = fs::symlink_metadata(path) {
            metadata.file_type().is_symlink()
        } else {
            false
        }
    }

    /// Gets the symlink path for a skill in an agent's directory.
    fn get_agent_skill_path(home: &PathBuf, agent_rel_path: &str, skill_name: &str) -> PathBuf {
        home.join(agent_rel_path).join(skill_name)
    }

    /// Creates a symlink for a skill in an agent's directory.
    fn create_skill_symlink(home: &PathBuf, agent_rel_path: &str, skill_name: &str) {
        let global_skill_path = home.join(".agents/skills").join(skill_name);
        let agent_skill_path = home.join(agent_rel_path).join(skill_name);
        
        // Ensure parent directory exists
        if let Some(parent) = agent_skill_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        
        symlink(&global_skill_path, &agent_skill_path).expect("Failed to create symlink");
    }

    /// Creates a blocking file (not a symlink) at the agent's skill path.
    fn create_blocking_file(home: &PathBuf, agent_rel_path: &str, skill_name: &str) {
        let agent_skill_path = home.join(agent_rel_path).join(skill_name);
        
        // Ensure parent directory exists
        if let Some(parent) = agent_skill_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        
        fs::write(&agent_skill_path, "blocking file content").expect("Failed to create blocking file");
    }

    // ==================== Property Tests ====================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: skills-manager-enhancement, Property 5: Batch Link Creates Symlinks for All Detected Agents**
        ///
        /// **Validates: Requirements 1.4, 6.1, 6.3**
        ///
        /// For any skill and set of detected agents, after executing "Link to All":
        /// - All detected agents SHALL have a symlink to that skill
        /// - Non-detected agents SHALL remain unchanged (no symlink created)
        #[test]
        fn prop_batch_link_creates_symlinks_for_all_detected_agents(
            skill_name in valid_skill_name_strategy(),
            detected_indices in agent_subset_strategy()
        ) {
            // Arrange: Create temp home with global skill and detected agent directories
            let temp_home = create_temp_home();
            let home_path = temp_home.path().to_path_buf();
            
            // Create the global skill
            create_global_skill(&home_path, &skill_name);
            
            // Create directories for detected agents
            let detected_agent_ids = create_agent_directories(&home_path, &detected_indices);
            
            // Get all agent definitions for verification
            let all_definitions = get_agent_definition_list();
            
            // Act: Link skill to all
            let result = link_skill_to_all_with_home(&skill_name, &home_path)
                .expect("link_skill_to_all should succeed");
            
            // Assert Property 5a: All detected agents should have symlinks
            for agent_id in &detected_agent_ids {
                // Find the agent's path
                let agent_def = all_definitions.iter()
                    .find(|(id, _, _)| *id == agent_id.as_str())
                    .expect("Agent definition should exist");
                let (_, _, rel_path) = agent_def;
                
                let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                prop_assert!(
                    symlink_exists(&symlink_path),
                    "Detected agent '{}' should have symlink at {:?}",
                    agent_id, symlink_path
                );
                
                // Verify it's in the success list
                prop_assert!(
                    result.success.contains(agent_id),
                    "Detected agent '{}' should be in success list",
                    agent_id
                );
            }
            
            // Assert Property 5b: Non-detected agents should NOT have symlinks
            let detected_set: HashSet<&String> = detected_agent_ids.iter().collect();
            for (id, _, rel_path) in &all_definitions {
                if !detected_set.contains(&id.to_string()) {
                    let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                    prop_assert!(
                        !symlink_exists(&symlink_path),
                        "Non-detected agent '{}' should NOT have symlink at {:?}",
                        id, symlink_path
                    );
                }
            }
            
            // Assert: No failures for detected agents (unless there was a blocking file)
            // Since we're creating fresh directories, there should be no failures
            prop_assert!(
                result.failed.is_empty(),
                "Should have no failures when creating fresh symlinks, but got: {:?}",
                result.failed
            );
        }

        /// **Feature: skills-manager-enhancement, Property 6: Batch Unlink Removes Symlinks from All Agents**
        ///
        /// **Validates: Requirements 1.5, 6.2**
        ///
        /// For any skill that is linked to one or more agents, after executing "Unlink from All":
        /// - No agents SHALL have a symlink to that skill
        #[test]
        fn prop_batch_unlink_removes_symlinks_from_all_agents(
            skill_name in valid_skill_name_strategy(),
            linked_indices in agent_subset_strategy()
        ) {
            // Arrange: Create temp home with global skill and symlinks for some agents
            let temp_home = create_temp_home();
            let home_path = temp_home.path().to_path_buf();
            
            // Create the global skill
            create_global_skill(&home_path, &skill_name);
            
            // Get all agent definitions
            let all_definitions = get_agent_definition_list();
            
            // Create symlinks for the specified agents
            let mut linked_agent_ids = Vec::new();
            for &idx in &linked_indices {
                if idx < all_definitions.len() {
                    let (id, _, rel_path) = all_definitions[idx];
                    create_skill_symlink(&home_path, rel_path, &skill_name);
                    linked_agent_ids.push(id.to_string());
                }
            }
            
            // Verify symlinks exist before unlink
            for agent_id in &linked_agent_ids {
                let agent_def = all_definitions.iter()
                    .find(|(id, _, _)| *id == agent_id.as_str())
                    .expect("Agent definition should exist");
                let (_, _, rel_path) = agent_def;
                let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                prop_assert!(
                    symlink_exists(&symlink_path),
                    "Symlink should exist before unlink for agent '{}'",
                    agent_id
                );
            }
            
            // Act: Unlink skill from all
            let result = unlink_skill_from_all_with_home(&skill_name, &home_path)
                .expect("unlink_skill_from_all should succeed");
            
            // Assert Property 6: No agents should have symlinks after unlink
            for (id, _, rel_path) in &all_definitions {
                let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                prop_assert!(
                    !symlink_exists(&symlink_path),
                    "Agent '{}' should NOT have symlink after unlink at {:?}",
                    id, symlink_path
                );
            }
            
            // Assert: All previously linked agents should be in success list
            for agent_id in &linked_agent_ids {
                prop_assert!(
                    result.success.contains(agent_id),
                    "Previously linked agent '{}' should be in success list",
                    agent_id
                );
            }
            
            // Assert: No failures
            prop_assert!(
                result.failed.is_empty(),
                "Should have no failures when removing symlinks, but got: {:?}",
                result.failed
            );
        }

        /// **Feature: skills-manager-enhancement, Property 7: Batch Operations Handle Partial Failures**
        ///
        /// **Validates: Requirements 6.5**
        ///
        /// For any batch operation where some individual operations fail:
        /// - The operation SHALL complete for all non-failing agents
        /// - The result SHALL contain both successful and failed operations
        #[test]
        fn prop_batch_operations_handle_partial_failures(
            skill_name in valid_skill_name_strategy(),
            detected_indices in agent_subset_strategy().prop_filter(
                "need at least 2 detected agents for partial failure test",
                |indices| indices.len() >= 2
            )
        ) {
            // Arrange: Create temp home with global skill
            let temp_home = create_temp_home();
            let home_path = temp_home.path().to_path_buf();
            
            // Create the global skill
            create_global_skill(&home_path, &skill_name);
            
            // Create directories for detected agents
            let detected_agent_ids = create_agent_directories(&home_path, &detected_indices);
            
            // Get all agent definitions
            let all_definitions = get_agent_definition_list();
            
            // Create a blocking file for the first detected agent to cause a failure
            let first_agent_id = &detected_agent_ids[0];
            let first_agent_def = all_definitions.iter()
                .find(|(id, _, _)| *id == first_agent_id.as_str())
                .expect("Agent definition should exist");
            let (_, _, first_rel_path) = first_agent_def;
            create_blocking_file(&home_path, first_rel_path, &skill_name);
            
            // Act: Link skill to all
            let result = link_skill_to_all_with_home(&skill_name, &home_path)
                .expect("link_skill_to_all should succeed even with partial failures");
            
            // Assert Property 7a: The first agent should be in the failed list
            prop_assert!(
                result.failed.iter().any(|f| f.agent_id == *first_agent_id),
                "Agent '{}' with blocking file should be in failed list",
                first_agent_id
            );
            
            // Assert Property 7b: All other detected agents should succeed
            for agent_id in detected_agent_ids.iter().skip(1) {
                prop_assert!(
                    result.success.contains(agent_id),
                    "Agent '{}' without blocking file should be in success list",
                    agent_id
                );
                
                // Verify symlink was created
                let agent_def = all_definitions.iter()
                    .find(|(id, _, _)| *id == agent_id.as_str())
                    .expect("Agent definition should exist");
                let (_, _, rel_path) = agent_def;
                let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                prop_assert!(
                    symlink_exists(&symlink_path),
                    "Agent '{}' should have symlink created despite other failures",
                    agent_id
                );
            }
            
            // Assert Property 7c: Failed operations should have error messages
            for failed_op in &result.failed {
                prop_assert!(
                    !failed_op.error.is_empty(),
                    "Failed operation for agent '{}' should have an error message",
                    failed_op.agent_id
                );
            }
            
            // Assert Property 7d: Total success + failed should equal detected agents
            let total_operations = result.success.len() + result.failed.len();
            prop_assert_eq!(
                total_operations, detected_agent_ids.len(),
                "Total operations ({}) should equal number of detected agents ({})",
                total_operations, detected_agent_ids.len()
            );
        }
    }

    // ==================== Additional Property Tests for Edge Cases ====================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: skills-manager-enhancement, Property 5 (Edge Case): Link to All with No Detected Agents**
        ///
        /// **Validates: Requirements 6.3**
        ///
        /// When no agents are detected, link_skill_to_all should succeed with empty results.
        #[test]
        fn prop_batch_link_with_no_detected_agents(
            skill_name in valid_skill_name_strategy()
        ) {
            // Arrange: Create temp home with global skill but no agent directories
            let temp_home = create_temp_home();
            let home_path = temp_home.path().to_path_buf();
            
            // Create the global skill
            create_global_skill(&home_path, &skill_name);
            
            // Act: Link skill to all (no agents detected)
            let result = link_skill_to_all_with_home(&skill_name, &home_path)
                .expect("link_skill_to_all should succeed");
            
            // Assert: Both success and failed should be empty
            prop_assert!(
                result.success.is_empty(),
                "Success list should be empty when no agents detected"
            );
            prop_assert!(
                result.failed.is_empty(),
                "Failed list should be empty when no agents detected"
            );
            
            // Assert: No symlinks should exist
            let all_definitions = get_agent_definition_list();
            for (_, _, rel_path) in &all_definitions {
                let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                prop_assert!(
                    !symlink_exists(&symlink_path),
                    "No symlinks should exist when no agents detected"
                );
            }
        }

        /// **Feature: skills-manager-enhancement, Property 6 (Edge Case): Unlink with No Existing Symlinks**
        ///
        /// **Validates: Requirements 6.2**
        ///
        /// When no symlinks exist, unlink_skill_from_all should succeed with empty results.
        #[test]
        fn prop_batch_unlink_with_no_existing_symlinks(
            skill_name in valid_skill_name_strategy()
        ) {
            // Arrange: Create temp home with global skill but no symlinks
            let temp_home = create_temp_home();
            let home_path = temp_home.path().to_path_buf();
            
            // Create the global skill
            create_global_skill(&home_path, &skill_name);
            
            // Act: Unlink skill from all (no symlinks exist)
            let result = unlink_skill_from_all_with_home(&skill_name, &home_path)
                .expect("unlink_skill_from_all should succeed");
            
            // Assert: Both success and failed should be empty
            prop_assert!(
                result.success.is_empty(),
                "Success list should be empty when no symlinks exist"
            );
            prop_assert!(
                result.failed.is_empty(),
                "Failed list should be empty when no symlinks exist"
            );
        }

        /// **Feature: skills-manager-enhancement, Property 5 (Idempotency): Link to All is Idempotent**
        ///
        /// **Validates: Requirements 1.4, 6.1**
        ///
        /// Calling link_skill_to_all twice should produce the same result.
        #[test]
        fn prop_batch_link_is_idempotent(
            skill_name in valid_skill_name_strategy(),
            detected_indices in agent_subset_strategy()
        ) {
            // Arrange: Create temp home with global skill and detected agents
            let temp_home = create_temp_home();
            let home_path = temp_home.path().to_path_buf();
            
            // Create the global skill
            create_global_skill(&home_path, &skill_name);
            
            // Create directories for detected agents
            let detected_agent_ids = create_agent_directories(&home_path, &detected_indices);
            
            // Act: Link skill to all twice
            let result1 = link_skill_to_all_with_home(&skill_name, &home_path)
                .expect("First link_skill_to_all should succeed");
            let result2 = link_skill_to_all_with_home(&skill_name, &home_path)
                .expect("Second link_skill_to_all should succeed");
            
            // Assert: Both results should have the same success list
            let success1: HashSet<_> = result1.success.iter().collect();
            let success2: HashSet<_> = result2.success.iter().collect();
            prop_assert_eq!(
                success1, success2,
                "Success lists should be identical for idempotent operation"
            );
            
            // Assert: Both should have no failures
            prop_assert!(
                result1.failed.is_empty() && result2.failed.is_empty(),
                "Both operations should have no failures"
            );
            
            // Assert: All detected agents should still have symlinks
            let all_definitions = get_agent_definition_list();
            for agent_id in &detected_agent_ids {
                let agent_def = all_definitions.iter()
                    .find(|(id, _, _)| *id == agent_id.as_str())
                    .expect("Agent definition should exist");
                let (_, _, rel_path) = agent_def;
                let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                prop_assert!(
                    symlink_exists(&symlink_path),
                    "Symlink should still exist after idempotent operation for agent '{}'",
                    agent_id
                );
            }
        }

        /// **Feature: skills-manager-enhancement, Property 6 (Idempotency): Unlink from All is Idempotent**
        ///
        /// **Validates: Requirements 1.5, 6.2**
        ///
        /// Calling unlink_skill_from_all twice should produce the same final state.
        #[test]
        fn prop_batch_unlink_is_idempotent(
            skill_name in valid_skill_name_strategy(),
            linked_indices in agent_subset_strategy()
        ) {
            // Arrange: Create temp home with global skill and symlinks
            let temp_home = create_temp_home();
            let home_path = temp_home.path().to_path_buf();
            
            // Create the global skill
            create_global_skill(&home_path, &skill_name);
            
            // Get all agent definitions
            let all_definitions = get_agent_definition_list();
            
            // Create symlinks for the specified agents
            for &idx in &linked_indices {
                if idx < all_definitions.len() {
                    let (_, _, rel_path) = all_definitions[idx];
                    create_skill_symlink(&home_path, rel_path, &skill_name);
                }
            }
            
            // Act: Unlink skill from all twice
            let _result1 = unlink_skill_from_all_with_home(&skill_name, &home_path)
                .expect("First unlink_skill_from_all should succeed");
            let result2 = unlink_skill_from_all_with_home(&skill_name, &home_path)
                .expect("Second unlink_skill_from_all should succeed");
            
            // Assert: Second call should have empty success (nothing to unlink)
            prop_assert!(
                result2.success.is_empty(),
                "Second unlink should have empty success list (nothing to unlink)"
            );
            
            // Assert: No failures
            prop_assert!(
                result2.failed.is_empty(),
                "Second unlink should have no failures"
            );
            
            // Assert: No symlinks should exist
            for (_, _, rel_path) in &all_definitions {
                let symlink_path = get_agent_skill_path(&home_path, rel_path, &skill_name);
                prop_assert!(
                    !symlink_exists(&symlink_path),
                    "No symlinks should exist after idempotent unlink"
                );
            }
        }
    }
}
