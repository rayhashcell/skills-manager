/**
 * TypeScript type definitions for the Skills Manager application.
 *
 * These types match the Rust struct definitions in src-tauri/src/lib.rs
 * and src-tauri/src/skill_parser.rs.
 *
 * Note: Field names use snake_case to match Rust's serde serialization format.
 *
 * Requirements: 1.2, 3.1
 */

/**
 * Represents an AI agent that can use skills.
 *
 * Agents are IDE or CLI tools (e.g., Cursor, Claude Code, Gemini CLI)
 * that have a skills directory where skills can be linked via symlinks.
 *
 * Requirements: 3.1
 */
export interface Agent {
  /** Unique identifier for the agent (e.g., "cursor", "claude-code") */
  id: string;
  /** Display name of the agent (e.g., "Cursor", "Claude Code") */
  name: string;
  /** Relative path from home directory to the agent's skills directory (e.g., ".cursor/skills") */
  path: string;
  /** Whether the agent's skills directory exists on the user's system */
  detected: boolean;
}

/**
 * Metadata extracted from a SKILL.md file.
 *
 * Contains the skill's name, description, and list of allowed tools.
 *
 * Requirements: 1.2
 */
export interface SkillMetadata {
  /** The name of the skill */
  name: string;
  /** A brief description of what the skill does */
  description: string;
  /** List of tools that the skill is allowed to use */
  allowed_tools: string[];
}

/**
 * Represents a skill stored in the global skills directory.
 *
 * A skill is a reusable instruction set for AI agents, stored as a directory
 * containing a SKILL.md file with metadata.
 *
 * Requirements: 1.2
 */
export interface Skill {
  /** Directory name of the skill in the global skills directory */
  name: string;
  /** Metadata parsed from the skill's SKILL.md file */
  metadata: SkillMetadata;
  /** List of agent IDs that have this skill installed (symlink OR local) */
  linked_agents: string[];
  /** List of agent IDs that have this skill linked via symlink only */
  symlinked_agents: string[];
}

/**
 * Status of a skill in an agent's directory
 */
export type AgentSkillStatus = 'symlink' | 'local' | 'not_installed';

/**
 * Represents a skill as seen from an agent's perspective.
 * Includes status (symlink/local/not_installed) and source path.
 */
export interface AgentSkill {
  /** Directory name of the skill */
  name: string;
  /** Metadata parsed from SKILL.md */
  metadata: SkillMetadata;
  /** Status: symlink, local, or not_installed */
  status: AgentSkillStatus;
  /** Source path (symlink target or local path), null if not installed */
  source_path: string | null;
  /** Whether this skill exists in global skills directory */
  in_global: boolean;
}

/**
 * Data for agent detail page
 */
export interface AgentDetailData {
  agent: Agent;
  skills: AgentSkill[];
}

/**
 * Application data containing all agents and skills.
 *
 * This is the main data structure returned by the get_app_data command.
 *
 * Requirements: 1.2, 3.1
 */
export interface AppData {
  /** List of all supported agents with their detection status */
  agents: Agent[];
  /** List of all skills from the global skills directory */
  skills: Skill[];
}

/**
 * Result of a batch operation (link to all / unlink from all).
 *
 * Contains lists of successful and failed operations.
 */
export interface BatchResult {
  /** Agent IDs for which the operation succeeded */
  success: string[];
  /** List of failed operations with error details */
  failed: FailedOperation[];
}

/**
 * Represents a failed operation during batch processing.
 *
 * Contains the agent ID and error message for debugging.
 */
export interface FailedOperation {
  /** The agent ID for which the operation failed */
  agent_id: string;
  /** Error message describing why the operation failed */
  error: string;
}
