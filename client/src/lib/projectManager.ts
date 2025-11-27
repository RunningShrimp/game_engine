/**
 * 项目管理系统
 * 处理项目的创建、打开、保存和配置
 */

export interface ProjectConfig {
  id: string;
  name: string;
  description: string;
  version: string;
  createdAt: number;
  updatedAt: number;
  settings: {
    rendering: {
      api: 'webgpu' | 'webgl';
      quality: 'low' | 'medium' | 'high' | 'ultra';
      vsync: boolean;
      fsr: boolean;
    };
    physics: {
      gravity: [number, number, number];
      timestep: number;
    };
    editor: {
      theme: 'dark' | 'light';
      autoSave: boolean;
      autoSaveInterval: number;
    };
  };
}

export interface RecentProject {
  id: string;
  name: string;
  path: string;
  lastOpened: number;
}

const STORAGE_KEY_CURRENT = 'game_engine_current_project';
const STORAGE_KEY_RECENT = 'game_engine_recent_projects';
const MAX_RECENT_PROJECTS = 10;

export class ProjectManager {
  private currentProject: ProjectConfig | null = null;
  private listeners: Set<(project: ProjectConfig | null) => void> = new Set();

  /**
   * 创建新项目
   */
  createProject(name: string, description: string = ''): ProjectConfig {
    const project: ProjectConfig = {
      id: this.generateId(),
      name,
      description,
      version: '1.0.0',
      createdAt: Date.now(),
      updatedAt: Date.now(),
      settings: this.getDefaultSettings(),
    };

    this.setCurrentProject(project);
    this.addToRecentProjects(project);
    return project;
  }

  /**
   * 打开项目
   */
  openProject(project: ProjectConfig): void {
    this.setCurrentProject(project);
    this.addToRecentProjects(project);
  }

  /**
   * 保存当前项目
   */
  saveProject(): boolean {
    if (!this.currentProject) {
      return false;
    }

    this.currentProject.updatedAt = Date.now();
    localStorage.setItem(
      STORAGE_KEY_CURRENT,
      JSON.stringify(this.currentProject)
    );
    this.addToRecentProjects(this.currentProject);
    return true;
  }

  /**
   * 关闭当前项目
   */
  closeProject(): void {
    this.setCurrentProject(null);
  }

  /**
   * 获取当前项目
   */
  getCurrentProject(): ProjectConfig | null {
    return this.currentProject;
  }

  /**
   * 加载保存的项目
   */
  loadSavedProject(): ProjectConfig | null {
    try {
      const saved = localStorage.getItem(STORAGE_KEY_CURRENT);
      if (saved) {
        const project = JSON.parse(saved) as ProjectConfig;
        this.currentProject = project;
        return project;
      }
    } catch (error) {
      console.error('Failed to load saved project:', error);
    }
    return null;
  }

  /**
   * 更新项目配置
   */
  updateProject(updates: Partial<ProjectConfig>): void {
    if (!this.currentProject) {
      return;
    }

    this.currentProject = {
      ...this.currentProject,
      ...updates,
      updatedAt: Date.now(),
    };

    this.notifyListeners();
    this.saveProject();
  }

  /**
   * 更新项目设置
   */
  updateSettings(settings: Partial<ProjectConfig['settings']>): void {
    if (!this.currentProject) {
      return;
    }

    this.currentProject.settings = {
      ...this.currentProject.settings,
      ...settings,
    };

    this.notifyListeners();
    this.saveProject();
  }

  /**
   * 获取最近项目列表
   */
  getRecentProjects(): RecentProject[] {
    try {
      const saved = localStorage.getItem(STORAGE_KEY_RECENT);
      if (saved) {
        return JSON.parse(saved) as RecentProject[];
      }
    } catch (error) {
      console.error('Failed to load recent projects:', error);
    }
    return [];
  }

  /**
   * 订阅项目变化
   */
  subscribe(listener: (project: ProjectConfig | null) => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * 设置当前项目
   */
  private setCurrentProject(project: ProjectConfig | null): void {
    this.currentProject = project;
    this.notifyListeners();
    
    if (project) {
      localStorage.setItem(STORAGE_KEY_CURRENT, JSON.stringify(project));
    } else {
      localStorage.removeItem(STORAGE_KEY_CURRENT);
    }
  }

  /**
   * 添加到最近项目
   */
  private addToRecentProjects(project: ProjectConfig): void {
    const recent = this.getRecentProjects();
    
    // 移除已存在的同名项目
    const filtered = recent.filter((p) => p.id !== project.id);
    
    // 添加到列表开头
    filtered.unshift({
      id: project.id,
      name: project.name,
      path: `/projects/${project.id}`,
      lastOpened: Date.now(),
    });

    // 限制列表大小
    const limited = filtered.slice(0, MAX_RECENT_PROJECTS);

    localStorage.setItem(STORAGE_KEY_RECENT, JSON.stringify(limited));
  }

  /**
   * 通知所有监听器
   */
  private notifyListeners(): void {
    this.listeners.forEach((listener) => listener(this.currentProject));
  }

  /**
   * 生成唯一ID
   */
  private generateId(): string {
    return `project_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 获取默认设置
   */
  private getDefaultSettings(): ProjectConfig['settings'] {
    return {
      rendering: {
        api: 'webgpu',
        quality: 'high',
        vsync: true,
        fsr: false,
      },
      physics: {
        gravity: [0, -9.81, 0],
        timestep: 1 / 60,
      },
      editor: {
        theme: 'dark',
        autoSave: true,
        autoSaveInterval: 300, // 5分钟
      },
    };
  }
}

// 导出单例实例
export const projectManager = new ProjectManager();
