/**
 * Tauri API wrapper functions for the Skills Manager application.
 *
 * This module provides type-safe wrappers around Tauri invoke commands,
 * handling the communication between the React frontend and Rust backend.
 *
 * Note: Function names use camelCase but Tauri command names use snake_case.
 *
 * Requirements: 5.4, 5.5, 6.1, 6.2
 */

import { invoke } from "@tauri-apps/api/core";
import type { AppData, BatchResult } from "./types";

/**
 * Error class for API-related errors.
 * Provides structured error information for better error handling in the UI.
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public readonly command: string,
    public readonly cause?: unknown
  ) {
    super(message);
    this.name = "ApiError";
  }
}

/**
 * Fetches all application data including agents and skills.
 *
 * This is the main data fetching function that retrieves:
 * - All 27 supported agents with their detection status
 * - All skills from the global skills directory with parsed metadata
 * - Link status for each skill across all agents
 *
 * @returns Promise resolving to AppData containing agents and skills
 * @throws ApiError if the backend command fails
 *
 * Requirements: 1.2, 3.1
 */
export async function getAppData(): Promise<AppData> {
  try {
    return await invoke<AppData>("get_app_data");
  } catch (error) {
    throw new ApiError(
      "Failed to fetch application data",
      "get_app_data",
      error
    );
  }
}

/**
 * Toggles a skill's link status for a specific agent.
 *
 * When enabled, creates a symlink from the agent's skills directory
 * to the skill in the global skills directory.
 * When disabled, removes the symlink from the agent's skills directory.
 *
 * @param agentId - The unique identifier of the agent (e.g., "cursor", "claude-code")
 * @param skillName - The name of the skill (directory name in global skills)
 * @param enable - True to create symlink (link skill), false to remove symlink (unlink skill)
 * @throws ApiError if the toggle operation fails
 *
 * Requirements: 5.4, 5.5
 */
export async function toggleSkill(
  agentId: string,
  skillName: string,
  enable: boolean
): Promise<void> {
  try {
    await invoke<void>("toggle_skill", {
      agentId,
      skillName,
      enable,
    });
  } catch (error) {
    const action = enable ? "link" : "unlink";
    throw new ApiError(
      `Failed to ${action} skill "${skillName}" for agent "${agentId}"`,
      "toggle_skill",
      error
    );
  }
}

/**
 * Links a skill to all detected agents.
 *
 * Creates symlinks for the specified skill in all agents' skills directories
 * where the agent is detected (i.e., the agent's skills directory exists).
 * Non-detected agents are skipped.
 *
 * @param skillName - The name of the skill to link to all agents
 * @returns Promise resolving to BatchResult with success and failed operations
 * @throws ApiError if the batch operation fails entirely
 *
 * Requirements: 6.1
 */
export async function linkSkillToAll(skillName: string): Promise<BatchResult> {
  try {
    return await invoke<BatchResult>("link_skill_to_all", {
      skillName,
    });
  } catch (error) {
    throw new ApiError(
      `Failed to link skill "${skillName}" to all agents`,
      "link_skill_to_all",
      error
    );
  }
}

/**
 * Unlinks a skill from all agents.
 *
 * Removes symlinks for the specified skill from all agents' skills directories.
 * This operation attempts to remove symlinks from all agents, regardless of
 * detection status, and reports success/failure for each.
 *
 * @param skillName - The name of the skill to unlink from all agents
 * @returns Promise resolving to BatchResult with success and failed operations
 * @throws ApiError if the batch operation fails entirely
 *
 * Requirements: 6.2
 */
export async function unlinkSkillFromAll(
  skillName: string
): Promise<BatchResult> {
  try {
    return await invoke<BatchResult>("unlink_skill_from_all", {
      skillName,
    });
  } catch (error) {
    throw new ApiError(
      `Failed to unlink skill "${skillName}" from all agents`,
      "unlink_skill_from_all",
      error
    );
  }
}
