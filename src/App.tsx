import { useEffect, useState, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "@/components/Sidebar";
import { GlobalSkillsPage } from "@/components/GlobalSkillsPage";
import { AgentDetailPage } from "@/components/AgentDetailPage";
import { ToastContainer, useToast } from "@/components/ui/toast";
import type { AppData, AgentDetailData } from "@/lib/types";

const MIN_LOADING_DURATION = 800;

function App() {
  const [data, setData] = useState<AppData>({ agents: [], skills: [] });
  const [agentDetail, setAgentDetail] = useState<AgentDetailData | null>(null);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [currentView, setCurrentView] = useState<'global-skills' | 'agent-detail'>('global-skills');
  const [loading, setLoading] = useState(true);
  const [sidebarWidth, setSidebarWidth] = useState(256);
  const { toasts, dismissToast, showError, showSuccess } = useToast();
  const loadingStartTime = useRef<number>(0);

  const setLoadingWithMinDuration = useCallback((isLoading: boolean) => {
    if (isLoading) {
      loadingStartTime.current = Date.now();
      setLoading(true);
    } else {
      const elapsed = Date.now() - loadingStartTime.current;
      const remaining = MIN_LOADING_DURATION - elapsed;
      if (remaining > 0) {
        setTimeout(() => setLoading(false), remaining);
      } else {
        setLoading(false);
      }
    }
  }, []);

  const fetchData = async () => {
    try {
      setLoadingWithMinDuration(true);
      const result = await invoke<AppData>("get_app_data");
      setData(result);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to load skills", errorMessage);
      console.error("Failed to fetch data:", error);
    } finally {
      setLoadingWithMinDuration(false);
    }
  };

  const fetchAgentDetail = async (agentId: string) => {
    try {
      setLoadingWithMinDuration(true);
      const result = await invoke<AgentDetailData>("get_agent_detail", { agentId });
      setAgentDetail(result);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to load agent details", errorMessage);
      console.error("Failed to fetch agent detail:", error);
    } finally {
      setLoadingWithMinDuration(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  useEffect(() => {
    if (selectedAgentId && currentView === 'agent-detail') {
      fetchAgentDetail(selectedAgentId);
    }
  }, [selectedAgentId, currentView]);

  const handleNavigateGlobalSkills = () => {
    setCurrentView('global-skills');
    setSelectedAgentId(null);
    setAgentDetail(null);
  };

  const handleSelectAgent = (agentId: string) => {
    setCurrentView('agent-detail');
    setSelectedAgentId(agentId);
  };

  const handleLinkSkill = async (skillName: string) => {
    if (!selectedAgentId) return;
    try {
      await invoke("toggle_skill", { agentId: selectedAgentId, skillName, enable: true });
      await fetchAgentDetail(selectedAgentId);
      await fetchData();
      showSuccess("Skill linked", `Linked ${skillName}`);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to link skill", errorMessage);
    }
  };

  const handleUnlinkSkill = async (skillName: string) => {
    if (!selectedAgentId) return;
    try {
      await invoke("toggle_skill", { agentId: selectedAgentId, skillName, enable: false });
      await fetchAgentDetail(selectedAgentId);
      await fetchData();
      showSuccess("Skill unlinked", `Unlinked ${skillName}`);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to unlink skill", errorMessage);
    }
  };

  const handleDeleteSkill = async (skillName: string) => {
    if (!selectedAgentId) return;
    try {
      await invoke("delete_local_skill", { agentId: selectedAgentId, skillName });
      await fetchAgentDetail(selectedAgentId);
      await fetchData();
      showSuccess("Skill deleted", `Deleted ${skillName}`);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to delete skill", errorMessage);
    }
  };

  const handleUploadToGlobal = async (skillName: string) => {
    if (!selectedAgentId) return;
    try {
      await invoke("upload_to_global", { agentId: selectedAgentId, skillName });
      await fetchAgentDetail(selectedAgentId);
      await fetchData();
      showSuccess("Skill uploaded", `Uploaded ${skillName} to global skills`);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to upload skill", errorMessage);
    }
  };

  const handleLinkSkillToAgents = async (skillName: string, agentIds: string[]) => {
    try {
      setLoadingWithMinDuration(true);
      let successCount = 0;
      let failCount = 0;
      
      for (const agentId of agentIds) {
        try {
          await invoke("toggle_skill", { agentId, skillName, enable: true });
          successCount++;
        } catch {
          failCount++;
        }
      }
      
      await fetchData();
      if (selectedAgentId) await fetchAgentDetail(selectedAgentId);
      
      if (failCount > 0) {
        showError("Some links failed", `${successCount} succeeded, ${failCount} failed`);
      } else {
        showSuccess("Skill linked", `Linked to ${successCount} agent(s)`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to link skill", errorMessage);
    } finally {
      setLoadingWithMinDuration(false);
    }
  };

  const handleUnlinkSkillFromAgents = async (skillName: string, agentIds: string[]) => {
    try {
      setLoadingWithMinDuration(true);
      let successCount = 0;
      let failCount = 0;
      
      for (const agentId of agentIds) {
        try {
          await invoke("toggle_skill", { agentId, skillName, enable: false });
          successCount++;
        } catch {
          failCount++;
        }
      }
      
      await fetchData();
      if (selectedAgentId) await fetchAgentDetail(selectedAgentId);
      
      if (failCount > 0) {
        showError("Some unlinks failed", `${successCount} succeeded, ${failCount} failed`);
      } else {
        showSuccess("Skill unlinked", `Unlinked from ${successCount} agent(s)`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showError("Failed to unlink skill", errorMessage);
    } finally {
      setLoadingWithMinDuration(false);
    }
  };

  const handleRefresh = async () => {
    await fetchData();
    if (selectedAgentId && currentView === 'agent-detail') {
      await fetchAgentDetail(selectedAgentId);
    }
  };

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden font-sans">
      <ToastContainer toasts={toasts} onDismiss={dismissToast} />

      <Sidebar
        agents={data.agents}
        skills={data.skills}
        currentView={currentView}
        selectedAgentId={selectedAgentId}
        onNavigateGlobalSkills={handleNavigateGlobalSkills}
        onSelectAgent={handleSelectAgent}
        onRefresh={handleRefresh}
        loading={loading}
        width={sidebarWidth}
        onWidthChange={setSidebarWidth}
      />

      {currentView === 'global-skills' ? (
        <GlobalSkillsPage
          skills={data.skills}
          agents={data.agents}
          onLinkSkill={handleLinkSkillToAgents}
          onUnlinkSkill={handleUnlinkSkillFromAgents}
          onRefresh={handleRefresh}
          loading={loading}
        />
      ) : agentDetail ? (
        <AgentDetailPage
          agent={agentDetail.agent}
          skills={agentDetail.skills}
          onLinkSkill={handleLinkSkill}
          onUnlinkSkill={handleUnlinkSkill}
          onDeleteSkill={handleDeleteSkill}
          onUploadToGlobal={handleUploadToGlobal}
          onRefresh={handleRefresh}
          loading={loading}
        />
      ) : null}
    </div>
  );
}

export default App;
