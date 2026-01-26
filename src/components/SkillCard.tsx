/**
 * SkillCard Component
 *
 * A reusable card component for displaying skill information.
 * Supports two variants:
 * - 'global': Shows batch action buttons (Link to All, Unlink from All) and linked agents badges
 * - 'agent': Shows toggle switch for linking/unlinking a skill to a specific agent
 *
 * Requirements: 1.2, 1.3, 5.2, 5.3
 */

import { Link2, Unlink2, Wrench } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { Skill, Agent } from "@/lib/types";

export interface SkillCardProps {
  /** The skill to display */
  skill: Skill;
  /** List of all agents (used to display linked agent names) */
  agents: Agent[];
  /** Card variant: 'global' for Global Skills page, 'agent' for Agent Detail page */
  variant: 'global' | 'agent';
  /** Whether the skill is linked to the current agent (only used in 'agent' variant) */
  isLinked?: boolean;
  /** Callback when toggle switch is changed (only used in 'agent' variant) */
  onToggle?: (enable: boolean) => void;
  /** Callback when "Link to All" button is clicked (only used in 'global' variant) */
  onLinkToAll?: () => void;
  /** Callback when "Unlink from All" button is clicked (only used in 'global' variant) */
  onUnlinkFromAll?: () => void;
  /** Whether the card controls are disabled */
  disabled?: boolean;
}

/**
 * SkillCard displays skill information with variant-specific controls.
 *
 * In 'global' variant:
 * - Shows "Link to All" and "Unlink from All" buttons
 * - Displays badges for all linked agents
 *
 * In 'agent' variant:
 * - Shows a toggle switch reflecting the isLinked state
 * - Toggle controls linking/unlinking the skill
 */
export function SkillCard({
  skill,
  agents,
  variant,
  isLinked = false,
  onToggle,
  onLinkToAll,
  onUnlinkFromAll,
  disabled = false,
}: SkillCardProps) {
  const { metadata, linked_agents } = skill;
  const hasLinkedAgents = linked_agents.length > 0;

  // Get agent names for linked agents badges
  const getAgentName = (agentId: string): string => {
    const agent = agents.find(a => a.id === agentId);
    return agent?.name ?? agentId;
  };

  // Determine if the card should show "active" styling
  const isActive = variant === 'agent' ? isLinked : hasLinkedAgents;

  return (
    <Card
      className={cn(
        "group relative overflow-hidden transition-all duration-300 border-border/50 bg-card/40 hover:bg-card/60 hover:border-primary/30",
        isActive && "border-primary/20 shadow-[0_0_20px_-10px_hsl(var(--primary)/0.1)]"
      )}
    >
      <CardHeader className="pb-3 pt-5 px-5">
        <div className="flex justify-between items-start gap-4">
          <div className="flex-1 min-w-0">
            <CardTitle className="font-mono text-base tracking-tight mb-1 group-hover:text-primary transition-colors">
              {metadata.name}
            </CardTitle>
            <CardDescription className="line-clamp-2 text-xs">
              {metadata.description}
            </CardDescription>
          </div>

          {/* Agent variant: Toggle switch */}
          {variant === 'agent' && (
            <Switch
              checked={isLinked}
              onCheckedChange={onToggle}
              disabled={disabled}
              className="data-[state=checked]:bg-primary data-[state=checked]:shadow-[0_0_12px_hsl(var(--primary))]"
            />
          )}
        </div>
      </CardHeader>

      <CardContent className="px-5 pb-4 space-y-3">
        {/* Allowed tools section */}
        {metadata.allowed_tools.length > 0 && (
          <div className="flex items-start gap-2">
            <Wrench className="h-3.5 w-3.5 text-muted-foreground mt-0.5 shrink-0" />
            <div className="flex flex-wrap gap-1">
              {metadata.allowed_tools.slice(0, 5).map((tool) => (
                <Badge
                  key={tool}
                  variant="outline"
                  className="text-[10px] px-1.5 h-5 font-mono"
                >
                  {tool}
                </Badge>
              ))}
              {metadata.allowed_tools.length > 5 && (
                <Badge variant="outline" className="text-[10px] px-1.5 h-5">
                  +{metadata.allowed_tools.length - 5}
                </Badge>
              )}
            </div>
          </div>
        )}

        {/* Linked agents badges (shown in both variants) */}
        <div className="flex flex-wrap gap-1.5">
          {hasLinkedAgents ? (
            <>
              {linked_agents.slice(0, 3).map((agentId) => (
                <Badge
                  key={agentId}
                  variant="secondary"
                  className="text-[10px] px-1.5 h-5 bg-secondary/50 text-secondary-foreground/70"
                >
                  {getAgentName(agentId)}
                </Badge>
              ))}
              {linked_agents.length > 3 && (
                <Badge variant="secondary" className="text-[10px] px-1.5 h-5">
                  +{linked_agents.length - 3}
                </Badge>
              )}
            </>
          ) : (
            <span className="text-[10px] text-muted-foreground/50 italic">
              No agents linked
            </span>
          )}
        </div>

        {/* Global variant: Batch action buttons */}
        {variant === 'global' && (
          <div className="flex gap-2 pt-2">
            <Button
              variant="outline"
              size="xs"
              onClick={onLinkToAll}
              disabled={disabled}
              className="flex-1 gap-1.5"
            >
              <Link2 className="h-3 w-3" />
              Link to All
            </Button>
            <Button
              variant="outline"
              size="xs"
              onClick={onUnlinkFromAll}
              disabled={disabled || !hasLinkedAgents}
              className="flex-1 gap-1.5"
            >
              <Unlink2 className="h-3 w-3" />
              Unlink from All
            </Button>
          </div>
        )}
      </CardContent>

      {/* Decorator Line - shows when skill is active/linked */}
      <div
        className={cn(
          "absolute bottom-0 left-0 h-0.5 bg-gradient-to-r from-primary to-transparent transition-all duration-300",
          isActive ? "w-full opacity-100" : "w-0 opacity-0"
        )}
      />
    </Card>
  );
}

export default SkillCard;
