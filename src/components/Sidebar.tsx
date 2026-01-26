/**
 * Sidebar Component
 * 
 * Navigation sidebar with Global Skills and Agents list.
 */

import { Terminal, Globe, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type { Agent, Skill } from "@/lib/types";

export interface SidebarProps {
  agents: Agent[];
  skills?: Skill[];
  currentView: "global-skills" | "agent-detail";
  selectedAgentId: string | null;
  onNavigateGlobalSkills: () => void;
  onSelectAgent: (agentId: string) => void;
  onRefresh?: () => void;
  loading?: boolean;
  width?: number;
  onWidthChange?: (width: number) => void;
}

function sortAgents(agents: Agent[], skills: Skill[]): Agent[] {
  const agentSkillCount = new Map<string, number>();
  for (const agent of agents) {
    const count = skills.filter(skill => skill.linked_agents.includes(agent.id)).length;
    agentSkillCount.set(agent.id, count);
  }

  return [...agents].sort((a, b) => {
    const aCount = agentSkillCount.get(a.id) || 0;
    const bCount = agentSkillCount.get(b.id) || 0;
    
    if (aCount > 0 && bCount === 0) return -1;
    if (aCount === 0 && bCount > 0) return 1;
    
    return a.name.localeCompare(b.name);
  });
}

export function Sidebar({
  agents,
  skills = [],
  currentView,
  selectedAgentId,
  onNavigateGlobalSkills,
  onSelectAgent,
  onRefresh,
  loading = false,
  width = 256,
  onWidthChange,
}: SidebarProps) {
  const isGlobalSkillsSelected = currentView === "global-skills";
  const sortedAgents = sortAgents(agents, skills);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (!onWidthChange) return;
    e.preventDefault();
    
    const startX = e.clientX;
    const startWidth = width;
    
    const handleMouseMove = (e: MouseEvent) => {
      const newWidth = Math.max(200, Math.min(400, startWidth + e.clientX - startX));
      onWidthChange(newWidth);
    };
    
    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  return (
    <aside 
      className="flex flex-col border-r border-border bg-card relative"
      style={{ width: `${width}px` }}
    >
      {/* Header with Global Skills */}
      <div className="p-3">
        <button
          onClick={onNavigateGlobalSkills}
          className={cn(
            "w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-sm cursor-pointer transition-all font-medium border",
            isGlobalSkillsSelected
              ? "bg-primary text-primary-foreground border-primary shadow-sm shadow-primary/25"
              : "bg-primary/10 text-primary border-primary/20 hover:bg-primary/20"
          )}
          aria-current={isGlobalSkillsSelected ? "page" : undefined}
        >
          <Globe className="size-4" />
          <span>Global Skills</span>
          <span className="ml-auto text-xs opacity-70">{skills.length}</span>
        </button>
      </div>

      {/* Agents List */}
      <div className="flex-1 min-h-0 flex flex-col px-3 pb-3">
        <div className="flex items-center justify-between mb-2 px-1">
          <h2 className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
            Agents
          </h2>
          {onRefresh && (
            <Button
              variant="ghost"
              size="icon-xs"
              onClick={onRefresh}
              disabled={loading}
              aria-label="Refresh data"
            >
              <RefreshCw className={cn("size-3", loading && "animate-spin-smooth")} />
            </Button>
          )}
        </div>

        <ScrollArea className="flex-1">
          <div className="space-y-0.5">
            {sortedAgents.map((agent) => {
              const isSelected = currentView === "agent-detail" && selectedAgentId === agent.id;
              const linkedSkillsCount = skills.filter(s => s.linked_agents.includes(agent.id)).length;
              const hasLinkedSkills = linkedSkillsCount > 0;

              return (
                <button
                  key={agent.id}
                  onClick={() => onSelectAgent(agent.id)}
                  className={cn(
                    "w-full flex items-center justify-between px-2.5 py-1.5 rounded-md text-sm cursor-pointer transition-colors",
                    isSelected
                      ? "bg-accent text-accent-foreground font-medium"
                      : "text-muted-foreground hover:bg-muted hover:text-foreground"
                  )}
                  aria-current={isSelected ? "page" : undefined}
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <Terminal className="size-3.5 shrink-0 opacity-60" />
                    <span className="truncate">{agent.name}</span>
                  </div>
                  <div className="flex items-center gap-1.5 shrink-0">
                    {linkedSkillsCount > 0 && (
                      <span className="text-[10px] text-muted-foreground tabular-nums">
                        {linkedSkillsCount}
                      </span>
                    )}
                    {hasLinkedSkills && (
                      <div
                        className="size-1.5 rounded-full bg-primary"
                        title="Has linked skills"
                        aria-label="Has linked skills"
                      />
                    )}
                  </div>
                </button>
              );
            })}
          </div>
        </ScrollArea>
      </div>

      {/* Resize handle */}
      {onWidthChange && (
        <div
          className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-primary/20 transition-colors"
          onMouseDown={handleMouseDown}
        />
      )}
    </aside>
  );
}

export default Sidebar;
