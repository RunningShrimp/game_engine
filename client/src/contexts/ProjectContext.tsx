import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { projectManager, ProjectConfig, RecentProject } from '@/lib/projectManager';

interface ProjectContextType {
  currentProject: ProjectConfig | null;
  recentProjects: RecentProject[];
  createProject: (name: string, description?: string) => ProjectConfig;
  openProject: (project: ProjectConfig) => void;
  saveProject: () => boolean;
  closeProject: () => void;
  updateProject: (updates: Partial<ProjectConfig>) => void;
  updateSettings: (settings: Partial<ProjectConfig['settings']>) => void;
}

const ProjectContext = createContext<ProjectContextType | undefined>(undefined);

export function ProjectProvider({ children }: { children: ReactNode }) {
  const [currentProject, setCurrentProject] = useState<ProjectConfig | null>(null);
  const [recentProjects, setRecentProjects] = useState<RecentProject[]>([]);

  useEffect(() => {
    // 加载保存的项目
    const saved = projectManager.loadSavedProject();
    if (saved) {
      setCurrentProject(saved);
    }

    // 加载最近项目
    setRecentProjects(projectManager.getRecentProjects());

    // 订阅项目变化
    const unsubscribe = projectManager.subscribe((project) => {
      setCurrentProject(project);
      setRecentProjects(projectManager.getRecentProjects());
    });

    return unsubscribe;
  }, []);

  const createProject = (name: string, description?: string) => {
    return projectManager.createProject(name, description);
  };

  const openProject = (project: ProjectConfig) => {
    projectManager.openProject(project);
  };

  const saveProject = () => {
    return projectManager.saveProject();
  };

  const closeProject = () => {
    projectManager.closeProject();
  };

  const updateProject = (updates: Partial<ProjectConfig>) => {
    projectManager.updateProject(updates);
  };

  const updateSettings = (settings: Partial<ProjectConfig['settings']>) => {
    projectManager.updateSettings(settings);
  };

  return (
    <ProjectContext.Provider
      value={{
        currentProject,
        recentProjects,
        createProject,
        openProject,
        saveProject,
        closeProject,
        updateProject,
        updateSettings,
      }}
    >
      {children}
    </ProjectContext.Provider>
  );
}

export function useProject() {
  const context = useContext(ProjectContext);
  if (!context) {
    throw new Error('useProject must be used within ProjectProvider');
  }
  return context;
}
