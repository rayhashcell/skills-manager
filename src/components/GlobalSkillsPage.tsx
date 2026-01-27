/**
 * GlobalSkillsPage Component
 *
 * Displays all skills from the global skills directory in a table format.
 */

import { useState } from "react";
import { FolderOpen, RefreshCw, Sparkles, Link, Unlink, Check } from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { homeDir, join } from "@tauri-apps/api/path";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import type { Skill, Agent } from "@/lib/types";

export interface GlobalSkillsPageProps {
  skills: Skill[];
  agents: Agent[];
  onLinkSkill: (skillName: string, agentIds: string[]) => Promise<void>;
  onUnlinkSkill: (skillName: string, agentIds: string[]) => Promise<void>;
  onRefresh: () => void;
  loading: boolean;
}

export function GlobalSkillsPage({
  skills,
  agents,
  onLinkSkill,
  onUnlinkSkill,
  onRefresh,
  loading,
}: GlobalSkillsPageProps) {
  const detectedAgentsCount = agents.filter(a => a.detected).length;

  const handleOpenFolder = async () => {
    try {
      const home = await homeDir();
      const folderPath = await join(home, ".agents/skills");
      await revealItemInDir(folderPath);
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  };

  return (
    <div className="flex-1 flex flex-col min-w-0 bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card px-6 py-4 shrink-0">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-foreground">Global Skills</h2>
            <p className="text-sm text-muted-foreground mt-0.5">
              {skills.length} skill{skills.length !== 1 ? 's' : ''} • {detectedAgentsCount} agent{detectedAgentsCount !== 1 ? 's' : ''} detected
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={onRefresh}
              disabled={loading}
            >
              <RefreshCw className={cn("size-4", loading && "animate-spin-smooth")} />
              Refresh
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleOpenFolder}
            >
              <FolderOpen className="size-4" />
              Open Folder
            </Button>
          </div>
        </div>
        <p className="text-sm text-muted-foreground mt-3">
          Skills stored in <code className="px-1.5 py-0.5 rounded bg-muted font-mono text-xs">~/.agents/skills</code>
        </p>
      </header>

      {/* Content */}
      <ScrollArea className="flex-1 min-h-0">
        {/* Tip banner */}
        <div className="mx-6 mt-6 p-3 rounded-lg bg-accent/50 border border-accent text-sm text-foreground">
          <p>
            Get more skills at{" "}
            <a 
              href="https://skills.sh/" 
              target="_blank" 
              rel="noopener noreferrer"
              className="text-primary font-medium hover:underline"
            >
              skills.sh
            </a>
            {" "}— skills from there install directly to the global folder. For other sources, manually copy the skill folder to{" "}
            <code className="px-1 py-0.5 rounded bg-muted font-mono text-xs">~/.agents/skills</code>
          </p>
        </div>

        {skills.length > 0 ? (
          <div className="p-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-auto">Name</TableHead>
                  <TableHead>Linked</TableHead>
                  <TableHead className="w-[140px] text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {skills.map((skill) => (
                  <SkillRow
                    key={skill.name}
                    skill={skill}
                    agents={agents}
                    onLink={onLinkSkill}
                    onUnlink={onUnlinkSkill}
                    loading={loading}
                  />
                ))}
              </TableBody>
            </Table>
          </div>
        ) : (
          <EmptyState onRefresh={onRefresh} loading={loading} />
        )}
      </ScrollArea>
    </div>
  );
}

interface SkillRowProps {
  skill: Skill;
  agents: Agent[];
  onLink: (skillName: string, agentIds: string[]) => Promise<void>;
  onUnlink: (skillName: string, agentIds: string[]) => Promise<void>;
  loading: boolean;
}

function SkillRow({ skill, agents, onLink, onUnlink, loading }: SkillRowProps) {
  const [linkOpen, setLinkOpen] = useState(false);
  const [unlinkOpen, setUnlinkOpen] = useState(false);
  const [selectedForLink, setSelectedForLink] = useState<Set<string>>(new Set());
  const [selectedForUnlink, setSelectedForUnlink] = useState<Set<string>>(new Set());

  const linkedCount = skill.linked_agents.length;
  
  // Get linked agent details
  const linkedAgentDetails = skill.linked_agents
    .map(agentId => agents.find(a => a.id === agentId))
    .filter((a): a is Agent => a !== undefined);
  
  // Agents that can be linked (detected and not already installed - symlink or local)
  const linkableAgents = agents.filter(a => a.detected && !skill.linked_agents.includes(a.id));
  // Agents that can be unlinked (have this skill linked via symlink only - not local)
  const unlinkableAgents = agents.filter(a => skill.symlinked_agents.includes(a.id));

  const handleLinkToggle = (agentId: string) => {
    const newSet = new Set(selectedForLink);
    if (newSet.has(agentId)) {
      newSet.delete(agentId);
    } else {
      newSet.add(agentId);
    }
    setSelectedForLink(newSet);
  };

  const handleUnlinkToggle = (agentId: string) => {
    const newSet = new Set(selectedForUnlink);
    if (newSet.has(agentId)) {
      newSet.delete(agentId);
    } else {
      newSet.add(agentId);
    }
    setSelectedForUnlink(newSet);
  };

  const handleSelectAllForLink = () => {
    if (selectedForLink.size === linkableAgents.length) {
      setSelectedForLink(new Set());
    } else {
      setSelectedForLink(new Set(linkableAgents.map(a => a.id)));
    }
  };

  const handleSelectAllForUnlink = () => {
    if (selectedForUnlink.size === unlinkableAgents.length) {
      setSelectedForUnlink(new Set());
    } else {
      setSelectedForUnlink(new Set(unlinkableAgents.map(a => a.id)));
    }
  };

  const handleLinkConfirm = async () => {
    if (selectedForLink.size > 0) {
      await onLink(skill.name, Array.from(selectedForLink));
      setSelectedForLink(new Set());
      setLinkOpen(false);
    }
  };

  const handleUnlinkConfirm = async () => {
    if (selectedForUnlink.size > 0) {
      await onUnlink(skill.name, Array.from(selectedForUnlink));
      setSelectedForUnlink(new Set());
      setUnlinkOpen(false);
    }
  };

  return (
    <TableRow>
      <TableCell className="font-medium whitespace-nowrap">
        {skill.metadata.name || skill.name}
      </TableCell>
      <TableCell>
        {linkedCount > 0 ? (
          <div className="flex flex-wrap gap-1">
            {linkedAgentDetails.map(agent => (
              <Badge 
                key={agent.id} 
                variant="secondary"
                className="text-xs font-normal"
              >
                {agent.name}
              </Badge>
            ))}
          </div>
        ) : (
          <span className="text-sm text-muted-foreground">—</span>
        )}
      </TableCell>
      <TableCell className="text-right whitespace-nowrap">
        <div className="flex gap-1 justify-end">
          {/* Link Popover */}
          <Popover open={linkOpen} onOpenChange={setLinkOpen}>
            <PopoverTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                disabled={loading || linkableAgents.length === 0}
              >
                <Link className="size-3.5" />
                Link
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-64 p-0" align="end">
              <div className="p-3 border-b border-border">
                <p className="text-sm font-medium">Link to Agents</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Select agents to link this skill
                </p>
              </div>
              <div className="max-h-[200px] overflow-y-auto p-2">
                {linkableAgents.length > 0 ? (
                  <>
                    <label className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer">
                      <Checkbox
                        checked={selectedForLink.size === linkableAgents.length}
                        onCheckedChange={handleSelectAllForLink}
                      />
                      <span className="text-sm font-medium">Select All</span>
                    </label>
                    <div className="h-px bg-border my-1" />
                    {linkableAgents.map(agent => (
                      <label
                        key={agent.id}
                        className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer"
                      >
                        <Checkbox
                          checked={selectedForLink.has(agent.id)}
                          onCheckedChange={() => handleLinkToggle(agent.id)}
                        />
                        <span className="text-sm">{agent.name}</span>
                      </label>
                    ))}
                  </>
                ) : (
                  <p className="text-sm text-muted-foreground px-2 py-4 text-center">
                    All detected agents already have this skill
                  </p>
                )}
              </div>
              {linkableAgents.length > 0 && (
                <div className="p-2 border-t border-border">
                  <Button
                    size="sm"
                    className="w-full"
                    disabled={selectedForLink.size === 0 || loading}
                    onClick={handleLinkConfirm}
                  >
                    <Check className="size-3.5" />
                    Link ({selectedForLink.size})
                  </Button>
                </div>
              )}
            </PopoverContent>
          </Popover>

          {/* Unlink Popover */}
          <Popover open={unlinkOpen} onOpenChange={setUnlinkOpen}>
            <PopoverTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                className="text-destructive hover:text-destructive hover:bg-destructive/10"
                disabled={loading || unlinkableAgents.length === 0}
              >
                <Unlink className="size-3.5" />
                Unlink
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-64 p-0" align="end">
              <div className="p-3 border-b border-border">
                <p className="text-sm font-medium">Unlink from Agents</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Select agents to unlink this skill
                </p>
              </div>
              <div className="max-h-[200px] overflow-y-auto p-2">
                {unlinkableAgents.length > 0 ? (
                  <>
                    <label className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer">
                      <Checkbox
                        checked={selectedForUnlink.size === unlinkableAgents.length}
                        onCheckedChange={handleSelectAllForUnlink}
                      />
                      <span className="text-sm font-medium">Select All</span>
                    </label>
                    <div className="h-px bg-border my-1" />
                    {unlinkableAgents.map(agent => (
                      <label
                        key={agent.id}
                        className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-muted cursor-pointer"
                      >
                        <Checkbox
                          checked={selectedForUnlink.has(agent.id)}
                          onCheckedChange={() => handleUnlinkToggle(agent.id)}
                        />
                        <span className="text-sm">{agent.name}</span>
                      </label>
                    ))}
                  </>
                ) : (
                  <p className="text-sm text-muted-foreground px-2 py-4 text-center">
                    No agents have this skill linked
                  </p>
                )}
              </div>
              {unlinkableAgents.length > 0 && (
                <div className="p-2 border-t border-border">
                  <Button
                    size="sm"
                    variant="destructive"
                    className="w-full"
                    disabled={selectedForUnlink.size === 0 || loading}
                    onClick={handleUnlinkConfirm}
                  >
                    <Check className="size-3.5" />
                    Unlink ({selectedForUnlink.size})
                  </Button>
                </div>
              )}
            </PopoverContent>
          </Popover>
        </div>
      </TableCell>
    </TableRow>
  );
}

function EmptyState({ onRefresh, loading }: { onRefresh: () => void; loading: boolean }) {
  return (
    <div className="flex flex-col items-center justify-center h-64 p-6">
      <div className="size-16 rounded-2xl bg-accent flex items-center justify-center mb-4">
        <Sparkles className="size-8 text-accent-foreground" />
      </div>
      <h3 className="text-lg font-medium mb-2">No Skills Found</h3>
      <p className="text-sm text-muted-foreground max-w-md text-center mb-4">
        Add skills to <code className="px-1.5 py-0.5 rounded bg-muted font-mono text-xs">~/.agents/skills</code>
      </p>
      <Button variant="outline" size="sm" onClick={onRefresh} disabled={loading}>
        <FolderOpen className="size-4" />
        Check Again
      </Button>
    </div>
  );
}

export default GlobalSkillsPage;
