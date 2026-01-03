//! # TypeScript Plugin SDK
//!
//! Tools and utilities for developing TypeScript/JavaScript plugins for the editor.

/// TypeScript plugin interface definition
pub const TYPESCRIPT_PLUGIN_TEMPLATE: &str = r#"
interface PluginContext {
    engineApi: EngineApi;
    resourceManager: ResourceManager;
    config: PluginConfig;
}

interface PluginEvent {
    type: string;
    data?: any;
}

interface Plugin {
    name: string;
    version: string;
    apiVersion: string;
    description?: string;
    author?: string;
    dependencies?: string[];

    onLoad?(context: PluginContext): Promise<void> | void;
    onUnload?(context: PluginContext): Promise<void> | void;
    onUpdate?(context: PluginContext, deltaTime: number): void;
    onEvent?(event: PluginEvent): void;
}

// Global plugin registration
declare function registerPlugin(plugin: Plugin): void;
"#;

/// TypeScript plugin template
pub const TYPESCRIPT_MINIMAL_TEMPLATE: &str = r#"
const plugin = {
    name: "my-plugin",
    version: "0.1.0",
    apiVersion: "0.1.0",

    async onLoad(context) {
        console.log("Plugin loaded!");
        console.log("Config:", context.config);
    },

    onUpdate(context, deltaTime) {
        // Update logic here
    },

    onUnload(context) {
        console.log("Plugin unloaded!");
    }
};

registerPlugin(plugin);
"#;

/// TypeScript advanced plugin template
pub const TYPESCRIPT_ADVANCED_TEMPLATE: &str = r#"
interface MyPluginState {
    counter: number;
    lastUpdate: number;
}

const plugin: Plugin = {
    name: "my-advanced-plugin",
    version: "0.1.0",
    apiVersion: "0.1.0",
    description: "An advanced TypeScript plugin",
    author: "Your Name",
    dependencies: [],

    state: {
        counter: 0,
        lastUpdate: Date.now()
    } as MyPluginState,

    async onLoad(context) {
        console.log("Advanced plugin loaded!");

        // Subscribe to events
        context.engineApi.addEventListener("scene.load", (event: PluginEvent) => {
            console.log("Scene loaded:", event.data);
        });

        // Access resources
        const assets = await context.resourceManager.listAssets();
        console.log("Available assets:", assets.length);
    },

    onUpdate(context, deltaTime) {
        this.state.counter++;

        if (this.state.counter % 60 === 0) {
            console.log(`Plugin updated ${this.state.counter} times`);
            this.state.lastUpdate = Date.now();
        }

        // Example: Modify scene
        // const scene = context.engineApi.getActiveScene();
        // scene.traverse((node) => {
        //     // Process nodes
        // });
    },

    onEvent(event: PluginEvent) {
        console.log("Received event:", event.type, event.data);
    },

    onUnload(context) {
        console.log("Advanced plugin unloaded!");
        console.log(`Total updates: ${this.state.counter}`);
    }
};

registerPlugin(plugin);
"#;

/// TypeScript plugin manifest
pub const TYPESCRIPT_MANIFEST_TEMPLATE: &str = r#`
{
  "name": "my-plugin",
  "version": "0.1.0",
  "description": "My TypeScript plugin",
  "main": "dist/plugin.js",
  "scripts": {
    "build": "tsc",
    "watch": "tsc --watch"
  },
  "devDependencies": {
    "@types/game-engine-editor": "^0.1.0",
    "typescript": "^5.0.0"
  }
}
"#;

/// TypeScript type definitions for plugin API
pub const TYPESCRIPT_DEFINITIONS: &str = r#"
declare module "game-engine-editor" {
    export interface EngineApi {
        getVersion(): string;
        getActiveScene(): Scene;
        registerComponent(component: ComponentDefinition): void;
        addEventListener(event: string, handler: (event: PluginEvent) => void): void;
        removeEventListener(event: string, handler: (event: PluginEvent) => void): void;
    }

    export interface ResourceManager {
        loadAsset(path: string): Promise<Asset>;
        saveAsset(path: string, data: any): Promise<void>;
        listAssets(): Promise<Asset[]>;
        getAssetInfo(path: string): Promise<AssetInfo>;
    }

    export interface PluginConfig {
        settings: Record<string, any>;
        enabled: boolean;
        autoLoad: boolean;
        hotReload: boolean;
    }

    export interface PluginEvent {
        type: string;
        data?: any;
        timestamp?: number;
    }

    export interface Scene {
        readonly name: string;
        readonly path: string;
        getRootNodes(): Node[];
        findNodeByName(name: string): Node | null;
        traverse(callback: (node: Node) => void): void;
        createNode(name: string): Node;
        addNode(node: Node): void;
        removeNode(node: Node): void;
    }

    export interface Node {
        readonly id: string;
        readonly name: string;
        getTransform(): Transform;
        setTransform(transform: Transform): void;
        getComponents(): Component[];
        addComponent(component: Component): void;
        removeComponent(component: Component): void;
    }

    export interface Component {
        readonly type: string;
        getProperties(): Record<string, any>;
        setProperty(name: string, value: any): void;
    }

    export interface ComponentDefinition {
        type: string;
        properties: Record<string, PropertyDefinition>;
        onCreate?(component: Component): void;
        onUpdate?(component: Component, deltaTime: number): void;
        onDestroy?(component: Component): void;
    }

    export interface PropertyDefinition {
        type: "number" | "string" | "boolean" | "vector3" | "color";
        default?: any;
        min?: number;
        max?: number;
        readonly?: boolean;
    }

    export interface Asset {
        readonly path: string;
        readonly type: string;
        readonly size: number;
        data: any;
    }

    export interface AssetInfo {
        path: string;
        type: string;
        size: number;
        modifiedAt: Date;
    }

    export interface Transform {
        position: [number, number, number];
        rotation: [number, number, number, number];
        scale: [number, number, number];
    }

    export function registerPlugin(plugin: Plugin): void;
}
"#;

/// Generate package.json for TypeScript plugin
pub fn generate_typescript_package_json(name: &str, version: &str) -> String {
    format!(
        r#"
{{
  "name": "{}",
  "version": "{}",
  "description": "TypeScript plugin for Game Engine Editor",
  "main": "dist/plugin.js",
  "types": "dist/plugin.d.ts",
  "scripts": {{
    "build": "tsc",
    "watch": "tsc --watch",
    "test": "jest"
  }},
  "devDependencies": {{
    "@types/game-engine-editor": "^0.1.0",
    "typescript": "^5.0.0"
  }},
  "peerDependencies": {{
    "game-engine-editor": "^0.1.0"
  }}
}}
"#,
        name, version
    )
}

/// Generate tsconfig.json for TypeScript plugin
pub fn generate_tsconfig_json() -> String {
    r#"
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "declaration": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "moduleResolution": "node",
    "types": ["game-engine-editor"]
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_package_json() {
        let json = generate_typescript_package_json("my-plugin", "0.1.0");
        assert!(json.contains("my-plugin"));
        assert!(json.contains("0.1.0"));
        assert!(json.contains("typescript"));
    }

    #[test]
    fn test_generate_tsconfig() {
        let config = generate_tsconfig_json();
        assert!(config.contains("compilerOptions"));
        assert!(config.contains("outDir"));
    }
}
