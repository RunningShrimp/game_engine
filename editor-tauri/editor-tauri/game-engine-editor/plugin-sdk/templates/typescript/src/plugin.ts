/**
 * {{plugin-name}}
 *
 * {{description}}
 */

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

const plugin: Plugin = {
    name: "{{plugin-name}}",
    version: "0.1.0",
    apiVersion: "0.1.0",
    description: "{{description}}",
    author: "{{author}}",

    async onLoad(context) {
        console.log(`✓ Plugin '${this.name}' loaded!`);
        console.log(`  Engine version: ${context.engineApi.getVersion()}`);

        // TODO: Initialize your plugin here
    },

    onUpdate(context, deltaTime) {
        // Called every frame
        // TODO: Implement update logic
    },

    async onUnload(context) {
        console.log(`✓ Plugin '${this.name}' unloaded!`);

        // TODO: Cleanup your plugin here
    }
};

// Register the plugin
declare function registerPlugin(plugin: Plugin): void;
registerPlugin(plugin);
