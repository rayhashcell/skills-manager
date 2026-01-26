import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SkillCard } from "./SkillCard";
import type { Agent, Skill } from "@/lib/types";

// Mock skill data
const mockSkill: Skill = {
  name: "test-skill",
  metadata: {
    name: "Test Skill",
    description: "A test skill for unit testing",
    allowed_tools: ["tool1", "tool2", "tool3"],
  },
  linked_agents: ["cursor", "claude-code"],
  symlinked_agents: ["cursor", "claude-code"],
};

// Mock agents data
const mockAgents: Agent[] = [
  { id: "cursor", name: "Cursor", path: ".cursor/skills", detected: true },
  { id: "claude-code", name: "Claude Code", path: ".claude/skills", detected: true },
  { id: "gemini-cli", name: "Gemini CLI", path: ".gemini/skills", detected: false },
];

describe("SkillCard", () => {
  describe("Common rendering (both variants)", () => {
    it("displays skill name from metadata", () => {
      render(
        <SkillCard skill={mockSkill} agents={mockAgents} variant="global" />
      );
      expect(screen.getByText("Test Skill")).toBeInTheDocument();
    });

    it("displays skill description from metadata", () => {
      render(
        <SkillCard skill={mockSkill} agents={mockAgents} variant="global" />
      );
      expect(
        screen.getByText("A test skill for unit testing")
      ).toBeInTheDocument();
    });

    it("renders empty description when description is empty", () => {
      const skillWithoutDescription: Skill = {
        ...mockSkill,
        metadata: { ...mockSkill.metadata, description: "" },
      };
      render(
        <SkillCard
          skill={skillWithoutDescription}
          agents={mockAgents}
          variant="global"
        />
      );
      // Component renders empty description - the CardDescription element exists but is empty
      const descriptionElement = document.querySelector('[data-slot="card-description"]');
      expect(descriptionElement).toBeInTheDocument();
      expect(descriptionElement?.textContent).toBe("");
    });

    it("displays allowed tools as badges", () => {
      render(
        <SkillCard skill={mockSkill} agents={mockAgents} variant="global" />
      );
      expect(screen.getByText("tool1")).toBeInTheDocument();
      expect(screen.getByText("tool2")).toBeInTheDocument();
      expect(screen.getByText("tool3")).toBeInTheDocument();
    });

    it("shows +N badge when more than 5 allowed tools", () => {
      const skillWithManyTools: Skill = {
        ...mockSkill,
        metadata: {
          ...mockSkill.metadata,
          allowed_tools: ["tool1", "tool2", "tool3", "tool4", "tool5", "tool6"],
        },
      };
      render(
        <SkillCard
          skill={skillWithManyTools}
          agents={mockAgents}
          variant="global"
        />
      );
      // Component shows first 5 tools, then +1 for the remaining
      expect(screen.getByText("+1")).toBeInTheDocument();
    });

    it("displays linked agents as badges", () => {
      render(
        <SkillCard skill={mockSkill} agents={mockAgents} variant="global" />
      );
      expect(screen.getByText("Cursor")).toBeInTheDocument();
      expect(screen.getByText("Claude Code")).toBeInTheDocument();
    });

    it("shows 'No agents linked' when no agents are linked", () => {
      const skillWithNoLinks: Skill = {
        ...mockSkill,
        linked_agents: [],
      };
      render(
        <SkillCard
          skill={skillWithNoLinks}
          agents={mockAgents}
          variant="global"
        />
      );
      expect(screen.getByText("No agents linked")).toBeInTheDocument();
    });

    it("shows +N badge when more than 3 agents are linked", () => {
      const skillWithManyLinks: Skill = {
        ...mockSkill,
        linked_agents: ["cursor", "claude-code", "gemini-cli", "cline", "codex"],
      };
      const moreAgents: Agent[] = [
        ...mockAgents,
        { id: "cline", name: "Cline", path: ".cline/skills", detected: true },
        { id: "codex", name: "Codex", path: ".codex/skills", detected: true },
      ];
      render(
        <SkillCard
          skill={skillWithManyLinks}
          agents={moreAgents}
          variant="global"
        />
      );
      expect(screen.getByText("+2")).toBeInTheDocument();
    });
  });

  describe("Global variant", () => {
    it("displays 'Link to All' button", () => {
      render(
        <SkillCard skill={mockSkill} agents={mockAgents} variant="global" />
      );
      expect(
        screen.getByRole("button", { name: /link to all/i })
      ).toBeInTheDocument();
    });

    it("displays 'Unlink from All' button", () => {
      render(
        <SkillCard skill={mockSkill} agents={mockAgents} variant="global" />
      );
      expect(
        screen.getByRole("button", { name: /unlink from all/i })
      ).toBeInTheDocument();
    });

    it("calls onLinkToAll when 'Link to All' button is clicked", () => {
      const onLinkToAll = vi.fn();
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="global"
          onLinkToAll={onLinkToAll}
        />
      );
      fireEvent.click(screen.getByRole("button", { name: /link to all/i }));
      expect(onLinkToAll).toHaveBeenCalledTimes(1);
    });

    it("calls onUnlinkFromAll when 'Unlink from All' button is clicked", () => {
      const onUnlinkFromAll = vi.fn();
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="global"
          onUnlinkFromAll={onUnlinkFromAll}
        />
      );
      fireEvent.click(screen.getByRole("button", { name: /unlink from all/i }));
      expect(onUnlinkFromAll).toHaveBeenCalledTimes(1);
    });

    it("disables buttons when disabled prop is true", () => {
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="global"
          disabled={true}
        />
      );
      expect(screen.getByRole("button", { name: /link to all/i })).toBeDisabled();
      expect(screen.getByRole("button", { name: /unlink from all/i })).toBeDisabled();
    });

    it("does not display toggle switch", () => {
      render(
        <SkillCard skill={mockSkill} agents={mockAgents} variant="global" />
      );
      expect(screen.queryByRole("switch")).not.toBeInTheDocument();
    });
  });

  describe("Agent variant", () => {
    it("displays toggle switch", () => {
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="agent"
          isLinked={false}
        />
      );
      expect(screen.getByRole("switch")).toBeInTheDocument();
    });

    it("shows toggle switch in 'on' position when isLinked is true", () => {
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="agent"
          isLinked={true}
        />
      );
      expect(screen.getByRole("switch")).toHaveAttribute(
        "data-state",
        "checked"
      );
    });

    it("shows toggle switch in 'off' position when isLinked is false", () => {
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="agent"
          isLinked={false}
        />
      );
      expect(screen.getByRole("switch")).toHaveAttribute(
        "data-state",
        "unchecked"
      );
    });

    it("calls onToggle with true when switch is toggled on", () => {
      const onToggle = vi.fn();
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="agent"
          isLinked={false}
          onToggle={onToggle}
        />
      );
      fireEvent.click(screen.getByRole("switch"));
      expect(onToggle).toHaveBeenCalledWith(true);
    });

    it("calls onToggle with false when switch is toggled off", () => {
      const onToggle = vi.fn();
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="agent"
          isLinked={true}
          onToggle={onToggle}
        />
      );
      fireEvent.click(screen.getByRole("switch"));
      expect(onToggle).toHaveBeenCalledWith(false);
    });

    it("disables toggle switch when disabled prop is true", () => {
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="agent"
          isLinked={false}
          disabled={true}
        />
      );
      expect(screen.getByRole("switch")).toBeDisabled();
    });

    it("does not display batch action buttons", () => {
      render(
        <SkillCard
          skill={mockSkill}
          agents={mockAgents}
          variant="agent"
          isLinked={false}
        />
      );
      expect(
        screen.queryByRole("button", { name: /link to all/i })
      ).not.toBeInTheDocument();
      expect(
        screen.queryByRole("button", { name: /unlink from all/i })
      ).not.toBeInTheDocument();
    });
  });
});
