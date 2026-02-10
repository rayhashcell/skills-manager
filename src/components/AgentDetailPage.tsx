/**
 * AgentDetailPage Component
 *
 * Displays skills for a specific agent in a table format.
 * Supports batch link/unlink operations via checkbox selection.
 */

import { useState, useMemo } from "react";
import { AlertCircle, FolderOpen, RefreshCw, Sparkles, Link, Trash2, Unlink, ExternalLink, Upload } from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { homeDir, join } from "@tauri-apps/api/path";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import type { Agent, AgentSkill } from "@/lib/types";

export interface AgentDetailPageProps {
  agent: Agent;
  skills: AgentSkill[];
  onLinkSkill: (skillName: string) => Promise<void>;
  onUnlinkSkill: (skillName: string) => Promise<void>;
  onBatchLinkSkills: (skillNames: string[]) => Promise<void>;
  onBatchUnlinkSkills: (skillNames: string[]) => Promise<void>;
  onDeleteSkill: (skillName: string) => Promise<void>;
  onUploadToGlobal: (skillName: string) => Promise<void>;
  onRefresh: () => void;
  loading: boolean;
}

export function AgentDetailPage({
  agent,
  skills,
  onLinkSkill,
  onUnlinkSkill,
  onBatchLinkSkills,
  onBatchUnlinkSkills,
  onDeleteSkill,
  onUploadToGlobal,
  onRefresh,
  loading,
}: AgentDetailPageProps) {
  const installedCount = skills.filter(s => s.status !== 'not_installed').length;
  const [selected, setSelected] = useState<Set<string>>(new Set());

  // Skills that can be linked (not_installed + in_global)
  const linkableSkills = useMemo(
    () => skills.filter(s => s.status === 'not_installed' && s.in_global),
    [skills]
  );
  // Skills that can be unlinked (symlink)
  const unlinkableSkills = useMemo(
    () => skills.filter(s => s.status === 'symlink'),
    [skills]
  );
  // All selectable skills (linkable or unlinkable)
  const selectableSkills = useMemo(
    () => skills.filter(s => (s.status === 'not_installed' && s.in_global) || s.status === 'symlink'),
    [skills]
  );

  const selectedLinkable = useMemo(
    () => linkableSkills.filter(s => selected.has(s.name)),
    [linkableSkills, selected]
  );
  const selectedUnlinkable = useMemo(
    () => unlinkableSkills.filter(s => selected.has(s.name)),
    [unlinkableSkills, selected]
  );
  const hasSelection = selected.size > 0;
  const allSelectableSelected = selectableSkills.length > 0 && selectableSkills.every(s => selected.has(s.name));

  const handleToggleSelect = (skillName: string) => {
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(skillName)) {
        next.delete(skillName);
      } else {
        next.add(skillName);
      }
      return next;
    });
  };

  const handleToggleAll = () => {
    if (allSelectableSelected) {
      setSelected(new Set());
    } else {
      setSelected(new Set(selectableSkills.map(s => s.name)));
    }
  };

  const handleBatchLink = async () => {
    const names = selectedLinkable.map(s => s.name);
    if (names.length > 0) {
      await onBatchLinkSkills(names);
      setSelected(new Set());
    }
  };

  const handleBatchUnlink = async () => {
    const names = selectedUnlinkable.map(s => s.name);
    if (names.length > 0) {
      await onBatchUnlinkSkills(names);
      setSelected(new Set());
    }
  };

  const handleOpenFolder = async () => {
    try {
      const home = await homeDir();
      const folderPath = await join(home, agent.path);
      await revealItemInDir(folderPath);
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  };

  const handleOpenPath = async (path: string) => {
    try {
      await revealItemInDir(path);
    } catch (error) {
      console.error("Failed to open path:", error);
    }
  };

  const isSelectable = (skill: AgentSkill) =>
    (skill.status === 'not_installed' && skill.in_global) || skill.status === 'symlink';

  return (
    <div className="flex-1 flex flex-col min-w-0 bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card px-6 py-4 shrink-0">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold">
              <span className="text-muted-foreground">Configuring</span>{" "}
              <span className="text-primary">{agent.name}</span>
            </h2>
            <p className="text-sm text-muted-foreground mt-0.5">
              {agent.detected ? (
                <code className="px-1.5 py-0.5 rounded bg-primary/15 text-primary font-mono text-xs border border-primary/20">~/{agent.path}</code>
              ) : (
                <>
                  <code className="px-1.5 py-0.5 rounded bg-destructive/15 text-destructive font-mono text-xs border border-destructive/20">~/{agent.path}</code>
                  <span className="text-destructive ml-1">(not found)</span>
                </>
              )}
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
              disabled={!agent.detected}
            >
              <FolderOpen className="size-4" />
              Open Folder
            </Button>
          </div>
        </div>

        {skills.length > 0 && (
          <p className="text-sm text-muted-foreground mt-3">
            {installedCount} of {skills.length} skill{skills.length !== 1 ? 's' : ''} installed
          </p>
        )}
      </header>

      {/* Batch action bar */}
      {hasSelection && (
        <div className="border-b border-border bg-accent/50 px-6 py-2.5 shrink-0 flex items-center gap-3">
          <span className="text-sm text-foreground font-medium">
            {selected.size} selected
          </span>
          <div className="flex gap-2 ml-auto">
            {selectedLinkable.length > 0 && (
              <Button
                size="sm"
                onClick={handleBatchLink}
                disabled={loading}
              >
                <Link className="size-3.5" />
                Link ({selectedLinkable.length})
              </Button>
            )}
            {selectedUnlinkable.length > 0 && (
              <Button
                size="sm"
                variant="destructive"
                onClick={handleBatchUnlink}
                disabled={loading}
              >
                <Unlink className="size-3.5" />
                Unlink ({selectedUnlinkable.length})
              </Button>
            )}
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setSelected(new Set())}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {/* Content */}
      <ScrollArea className="flex-1 min-h-0">
        {!agent.detected && (
          <div className="mx-6 mt-6 p-4 rounded-lg border border-destructive/30 bg-destructive/5 text-destructive flex items-center gap-3">
            <AlertCircle className="size-5 shrink-0" />
            <p className="text-sm">
              Directory not found. Create <code className="px-1.5 py-0.5 rounded bg-destructive/10 font-mono text-xs">~/{agent.path}</code> to enable.
            </p>
          </div>
        )}

        {skills.length > 0 ? (
          <div className="p-6">
            <Table>
              <TableHeader>
                <TableRow>
                  {selectableSkills.length > 0 && (
                    <TableHead className="w-[40px]">
                      <Checkbox
                        checked={allSelectableSelected}
                        onCheckedChange={handleToggleAll}
                      />
                    </TableHead>
                  )}
                  <TableHead className="w-[180px]">Name</TableHead>
                  <TableHead className="w-[80px]">Status</TableHead>
                  <TableHead>Path</TableHead>
                  <TableHead className="w-[100px] text-right">Action</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {skills.map((skill) => (
                  <TableRow
                    key={skill.name}
                    className={cn(selected.has(skill.name) && "bg-accent/50")}
                  >
                    {selectableSkills.length > 0 && (
                      <TableCell>
                        {isSelectable(skill) ? (
                          <Checkbox
                            checked={selected.has(skill.name)}
                            onCheckedChange={() => handleToggleSelect(skill.name)}
                          />
                        ) : (
                          <span className="block size-4" />
                        )}
                      </TableCell>
                    )}
                    <TableCell className="font-medium">
                      {skill.metadata.name || skill.name}
                    </TableCell>
                    <TableCell>
                      <StatusBadge status={skill.status} />
                    </TableCell>
                    <TableCell>
                      <PathCell path={skill.source_path} onOpen={handleOpenPath} />
                    </TableCell>
                    <TableCell className="text-right">
                      <SkillActions
                        skill={skill}
                        agentDetected={agent.detected}
                        loading={loading}
                        onLink={onLinkSkill}
                        onUnlink={onUnlinkSkill}
                        onDelete={onDeleteSkill}
                        onUpload={onUploadToGlobal}
                      />
                    </TableCell>
                  </TableRow>
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

function StatusBadge({ status }: { status: AgentSkill['status'] }) {
  switch (status) {
    case 'symlink':
      return <Badge variant="secondary" className="bg-accent text-accent-foreground">Symlink</Badge>;
    case 'local':
      return <Badge variant="secondary" className="bg-warning/20 text-warning-foreground border-warning/30">Local</Badge>;
    case 'not_installed':
      return <Badge variant="outline" className="text-muted-foreground">Not Installed</Badge>;
  }
}

function PathCell({ path, onOpen }: { path: string | null; onOpen: (path: string) => void }) {
  if (!path) {
    return <span className="text-muted-foreground text-xs">—</span>;
  }

  return (
    <div className="group relative">
      <button
        onClick={() => onOpen(path)}
        className="font-mono text-xs text-muted-foreground hover:text-foreground flex items-center gap-1 cursor-pointer transition-colors"
      >
        <span className="truncate max-w-[400px]">{path}</span>
        <ExternalLink className="size-3 shrink-0 opacity-50" />
      </button>
      {/* Tooltip on hover */}
      <div className="absolute left-0 top-full mt-1 z-50 hidden group-hover:block">
        <div className="bg-popover text-popover-foreground border border-border rounded-md px-2 py-1 text-xs font-mono shadow-md max-w-[400px] break-all">
          {path}
        </div>
      </div>
    </div>
  );
}

interface SkillActionsProps {
  skill: AgentSkill;
  agentDetected: boolean;
  loading: boolean;
  onLink: (name: string) => Promise<void>;
  onUnlink: (name: string) => Promise<void>;
  onDelete: (name: string) => Promise<void>;
  onUpload: (name: string) => Promise<void>;
}

function SkillActions({ skill, agentDetected, loading, onLink, onUnlink, onDelete, onUpload }: SkillActionsProps) {
  if (skill.status === 'symlink') {
    return (
      <Button
        variant="ghost"
        size="sm"
        className="text-destructive hover:text-destructive hover:bg-destructive/10"
        onClick={() => onUnlink(skill.name)}
        disabled={loading}
      >
        <Unlink className="size-3.5" />
        Unlink
      </Button>
    );
  }

  if (skill.status === 'local') {
    return (
      <div className="flex gap-1 justify-end">
        {!skill.in_global && (
          <Button
            variant="ghost"
            size="sm"
            className="text-primary hover:text-primary hover:bg-primary/10"
            onClick={() => onUpload(skill.name)}
            disabled={loading}
          >
            <Upload className="size-3.5" />
            Upload
          </Button>
        )}
        <Button
          variant="ghost"
          size="sm"
          className="text-destructive hover:text-destructive hover:bg-destructive/10"
          onClick={() => onDelete(skill.name)}
          disabled={loading}
        >
          <Trash2 className="size-3.5" />
          Delete
        </Button>
      </div>
    );
  }

  if (skill.status === 'not_installed' && skill.in_global) {
    return (
      <Button
        variant="ghost"
        size="sm"
        className="text-primary hover:text-primary hover:bg-primary/10"
        onClick={() => onLink(skill.name)}
        disabled={loading || !agentDetected}
      >
        <Link className="size-3.5" />
        Link
      </Button>
    );
  }

  return null;
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

export default AgentDetailPage;
