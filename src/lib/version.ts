/**
 * Version checking utilities
 */

import { getVersion } from "@tauri-apps/api/app";

const GITHUB_RELEASES_API = "https://api.github.com/repos/rayhashcell/skills-manager/releases/latest";

export interface VersionInfo {
  currentVersion: string;
  latestVersion: string | null;
  hasUpdate: boolean;
  releaseUrl: string | null;
}

/**
 * Compare two semver versions
 * Returns true if latest > current
 */
function isNewerVersion(current: string, latest: string): boolean {
  const currentParts = current.replace(/^v/, '').split('.').map(Number);
  const latestParts = latest.replace(/^v/, '').split('.').map(Number);
  
  for (let i = 0; i < 3; i++) {
    const c = currentParts[i] || 0;
    const l = latestParts[i] || 0;
    if (l > c) return true;
    if (l < c) return false;
  }
  return false;
}

/**
 * Get current app version from Tauri
 */
export async function getCurrentVersion(): Promise<string> {
  try {
    return await getVersion();
  } catch {
    return "0.0.0";
  }
}

/**
 * Check for updates from GitHub releases
 */
export async function checkForUpdates(): Promise<VersionInfo> {
  const currentVersion = await getCurrentVersion();
  
  const result: VersionInfo = {
    currentVersion,
    latestVersion: null,
    hasUpdate: false,
    releaseUrl: null,
  };

  try {
    const response = await fetch(GITHUB_RELEASES_API);
    if (!response.ok) return result;
    
    const data = await response.json();
    const latestVersion = data.tag_name?.replace(/^v/, '') || null;
    
    if (latestVersion) {
      result.latestVersion = latestVersion;
      result.hasUpdate = isNewerVersion(currentVersion, latestVersion);
      result.releaseUrl = data.html_url || null;
    }
  } catch {
    // Silently fail - network issues shouldn't break the app
  }

  return result;
}
