/**
 * Property-based tests for Sidebar component.
 *
 * **Feature: skills-manager-enhancement**
 *
 * Tests:
 * - Property 8: Sidebar Linked Skills Indicator Matches Skills State
 * - Property 9: Selected Navigation Item is Highlighted
 *
 * **Validates: Requirements 4.4, 4.6**
 */

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import * as fc from 'fast-check';
import { Sidebar, type SidebarProps } from './Sidebar';
import type { Agent, Skill } from '@/lib/types';

// Clean up after each test to prevent DOM pollution
afterEach(() => {
  cleanup();
});

/**
 * Generator for Agent objects with unique, readable names.
 * Uses alphanumeric strings to ensure valid, unique agent names.
 */
const agentArbitrary = (index: number) =>
  fc.record({
    id: fc.constant(`agent-${index}`),
    name: fc
      .stringMatching(/^[A-Za-z][A-Za-z0-9 ]{0,19}$/)
      .map((s) => `${s.trim() || 'Agent'} ${index}`),
    path: fc.constant(`.agent-${index}/skills`),
    detected: fc.boolean(),
  });

/**
 * Generator for arrays of Agent objects with unique IDs and names.
 */
const agentsArrayArbitrary = fc
  .integer({ min: 1, max: 15 })
  .chain((count) => fc.tuple(...Array.from({ length: count }, (_, i) => agentArbitrary(i))))
  .map((agents) => agents as Agent[]);

/**
 * Generator for Skill objects that can be linked to agents.
 */
const skillArbitrary = (agentIds: string[]) =>
  fc.subarray(agentIds).chain((linkedAgents) =>
    fc.record({
      name: fc.stringMatching(/^[a-z][a-z0-9-]{2,15}$/).map(s => s || 'skill'),
      metadata: fc.constant({ name: 'Test Skill', description: 'Test', allowed_tools: [] as string[] }),
      linked_agents: fc.constant(linkedAgents),
      symlinked_agents: fc.subarray(linkedAgents), // symlinked is a subset of linked
    })
  );

/**
 * Generator for arrays of skills with linked agents.
 */
const skillsArrayArbitrary = (agentIds: string[]) =>
  fc.integer({ min: 0, max: 5 }).chain((count) =>
    fc.array(skillArbitrary(agentIds), { minLength: count, maxLength: count })
  );

/**
 * Generator for currentView state.
 */
const currentViewArbitrary = fc.constantFrom<'global-skills' | 'agent-detail'>(
  'global-skills',
  'agent-detail'
);

/**
 * Generator for navigation state including agents, currentView, and selectedAgentId.
 * When currentView is 'agent-detail', selectedAgentId is one of the agent IDs.
 * When currentView is 'global-skills', selectedAgentId is null.
 */
type NavigationState = {
  agents: Agent[];
  currentView: 'global-skills' | 'agent-detail';
  selectedAgentId: string | null;
};

const navigationStateArbitrary: fc.Arbitrary<NavigationState> = agentsArrayArbitrary.chain((agents) =>
  fc
    .record({
      agents: fc.constant(agents),
      currentView: currentViewArbitrary,
    })
    .chain(({ agents, currentView }): fc.Arbitrary<NavigationState> => {
      if (currentView === 'global-skills') {
        return fc.constant({
          agents,
          currentView: 'global-skills' as const,
          selectedAgentId: null,
        });
      } else {
        // For agent-detail view, select one of the agent IDs
        return fc.constantFrom(...agents.map((a) => a.id)).map((selectedAgentId) => ({
          agents,
          currentView: 'agent-detail' as const,
          selectedAgentId,
        }));
      }
    })
);

/**
 * Helper function to render Sidebar with default props.
 */
function renderSidebar(props: Partial<SidebarProps> & { agents: Agent[] }) {
  const defaultProps: SidebarProps = {
    agents: props.agents,
    skills: props.skills ?? [],
    currentView: props.currentView ?? 'global-skills',
    selectedAgentId: props.selectedAgentId ?? null,
    onNavigateGlobalSkills: vi.fn(),
    onSelectAgent: vi.fn(),
    onRefresh: vi.fn(),
    loading: false,
  };

  return render(<Sidebar {...defaultProps} {...props} />);
}

/**
 * Helper to find an agent button by agent name (since sorting changes order).
 * Uses exact match to avoid partial name collisions.
 */
function findAgentButtonByName(container: HTMLElement, agentName: string): HTMLElement | null {
  const buttons = container.querySelectorAll('.space-y-0\\.5 button');
  for (const button of buttons) {
    // Get the span with the agent name (it has class "truncate")
    const nameSpan = button.querySelector('span.truncate');
    if (nameSpan && nameSpan.textContent === agentName) {
      return button as HTMLElement;
    }
  }
  return null;
}

/**
 * Helper to count linked skills for an agent.
 */
function countLinkedSkills(agentId: string, skills: Skill[]): number {
  return skills.filter(s => s.linked_agents.includes(agentId)).length;
}

describe('Feature: skills-manager-enhancement', () => {
  describe('Property 8: Sidebar Linked Skills Indicator Matches Skills State', () => {
    /**
     * **Property 8: Sidebar Linked Skills Indicator**
     *
     * For any agent displayed in the sidebar, the visual indicator
     * SHALL be visible if and only if the agent has linked skills.
     *
     * **Validates: Requirements 4.4**
     */
    it('linked skills indicator is visible if and only if agent has linked skills', () => {
      fc.assert(
        fc.property(
          agentsArrayArbitrary.chain((agents) =>
            skillsArrayArbitrary(agents.map(a => a.id)).map((skills) => ({ agents, skills }))
          ),
          ({ agents, skills }) => {
            const { container } = renderSidebar({ agents, skills });

            // For each agent, find its button by name and verify indicator
            agents.forEach((agent) => {
              const agentButton = findAgentButtonByName(container, agent.name);
              expect(agentButton).toBeInTheDocument();

              // Check for linked skills indicator within the agent button
              const indicator = agentButton?.querySelector('[title="Has linked skills"]');
              const hasLinkedSkills = countLinkedSkills(agent.id, skills) > 0;

              if (hasLinkedSkills) {
                // If agent has linked skills, indicator should be visible
                expect(indicator).toBeInTheDocument();
                expect(indicator).toHaveAttribute('aria-label', 'Has linked skills');
              } else {
                // If agent has no linked skills, indicator should not be present
                expect(indicator).not.toBeInTheDocument();
              }
            });

            cleanup();
          }
        ),
        { numRuns: 100 }
      );
    });

    /**
     * Additional property: Indicator count matches agents with linked skills.
     *
     * The total number of indicators in the sidebar SHALL equal
     * the number of agents with at least one linked skill.
     *
     * **Validates: Requirements 4.4**
     */
    it('total indicators equals count of agents with linked skills', () => {
      fc.assert(
        fc.property(
          agentsArrayArbitrary.chain((agents) =>
            skillsArrayArbitrary(agents.map(a => a.id)).map((skills) => ({ agents, skills }))
          ),
          ({ agents, skills }) => {
            const { container } = renderSidebar({ agents, skills });

            // Count agents with linked skills
            const agentsWithLinkedSkillsCount = agents.filter(
              (a) => countLinkedSkills(a.id, skills) > 0
            ).length;

            // Count indicators in the DOM
            const indicators = container.querySelectorAll('[title="Has linked skills"]');

            expect(indicators.length).toBe(agentsWithLinkedSkillsCount);

            cleanup();
          }
        ),
        { numRuns: 100 }
      );
    });
  });

  describe('Property 9: Selected Navigation Item is Highlighted', () => {
    /**
     * **Property 9: Selected Navigation Item is Highlighted**
     *
     * For any navigation state, exactly one navigation item SHALL have the
     * "selected" visual style, and it SHALL correspond to the current view.
     *
     * **Validates: Requirements 4.6**
     */
    it('exactly one navigation item has aria-current="page"', () => {
      fc.assert(
        fc.property(navigationStateArbitrary, ({ agents, currentView, selectedAgentId }) => {
          const { container } = renderSidebar({ agents, currentView, selectedAgentId });

          // Find all elements with aria-current="page"
          const selectedItems = container.querySelectorAll('[aria-current="page"]');

          // Exactly one item should be selected
          expect(selectedItems.length).toBe(1);

          cleanup();
        }),
        { numRuns: 100 }
      );
    });

    /**
     * When currentView is 'global-skills', the Global Skills button SHALL be highlighted.
     *
     * **Validates: Requirements 4.6**
     */
    it('Global Skills is highlighted when currentView is global-skills', () => {
      fc.assert(
        fc.property(agentsArrayArbitrary, (agents) => {
          const { container } = renderSidebar({
            agents,
            currentView: 'global-skills',
            selectedAgentId: null,
          });

          // Find the Global Skills button within the container to avoid multiple matches
          const globalSkillsButton = container.querySelector('button[aria-current="page"]');

          // It should exist and have aria-current="page"
          expect(globalSkillsButton).toBeInTheDocument();

          // It should have the selected styling class (primary for Global Skills)
          expect(globalSkillsButton).toHaveClass('bg-primary');
          expect(globalSkillsButton).toHaveClass('text-primary-foreground');

          cleanup();
        }),
        { numRuns: 100 }
      );
    });

    /**
     * When currentView is 'agent-detail', the selected agent button SHALL be highlighted.
     *
     * **Validates: Requirements 4.6**
     */
    it('selected agent is highlighted when currentView is agent-detail', () => {
      fc.assert(
        fc.property(
          agentsArrayArbitrary.chain((agents) =>
            fc.constantFrom(...agents.map((a) => a.id)).map((selectedAgentId) => ({
              agents,
              selectedAgentId,
            }))
          ),
          ({ agents, selectedAgentId }) => {
            const { container } = renderSidebar({
              agents,
              currentView: 'agent-detail',
              selectedAgentId,
            });

            // Find the selected agent by ID
            const selectedAgent = agents.find((a) => a.id === selectedAgentId);
            expect(selectedAgent).toBeDefined();

            // Find the agent button by name (since sorting changes order)
            const selectedAgentButton = findAgentButtonByName(container, selectedAgent!.name);
            expect(selectedAgentButton).toBeInTheDocument();

            // It should have aria-current="page"
            expect(selectedAgentButton).toHaveAttribute('aria-current', 'page');

            // It should have the selected styling class
            expect(selectedAgentButton).toHaveClass('bg-accent');
            expect(selectedAgentButton).toHaveClass('text-accent-foreground');

            cleanup();
          }
        ),
        { numRuns: 100 }
      );
    });

    /**
     * When currentView is 'agent-detail', Global Skills button SHALL NOT be highlighted.
     *
     * **Validates: Requirements 4.6**
     */
    it('Global Skills is not highlighted when currentView is agent-detail', () => {
      fc.assert(
        fc.property(
          agentsArrayArbitrary.chain((agents) =>
            fc.constantFrom(...agents.map((a) => a.id)).map((selectedAgentId) => ({
              agents,
              selectedAgentId,
            }))
          ),
          ({ agents, selectedAgentId }) => {
            renderSidebar({
              agents,
              currentView: 'agent-detail',
              selectedAgentId,
            });

            // Find the Global Skills button
            const globalSkillsButton = screen.getByRole('button', { name: /Global Skills/i });

            // It should NOT have aria-current="page"
            expect(globalSkillsButton).not.toHaveAttribute('aria-current', 'page');

            cleanup();
          }
        ),
        { numRuns: 100 }
      );
    });

    /**
     * Non-selected agents SHALL NOT be highlighted.
     *
     * **Validates: Requirements 4.6**
     */
    it('non-selected agents are not highlighted', () => {
      fc.assert(
        fc.property(
          agentsArrayArbitrary.filter((agents) => agents.length > 1).chain((agents) =>
            fc.constantFrom(...agents.map((a) => a.id)).map((selectedAgentId) => ({
              agents,
              selectedAgentId,
            }))
          ),
          ({ agents, selectedAgentId }) => {
            const { container } = renderSidebar({
              agents,
              currentView: 'agent-detail',
              selectedAgentId,
            });

            // Check all non-selected agents by finding them by name
            agents.forEach((agent) => {
              if (agent.id !== selectedAgentId) {
                const agentButton = findAgentButtonByName(container, agent.name);
                expect(agentButton).toBeInTheDocument();
                // Non-selected agents should NOT have aria-current="page"
                expect(agentButton).not.toHaveAttribute('aria-current', 'page');
              }
            });

            cleanup();
          }
        ),
        { numRuns: 100 }
      );
    });
  });
});
