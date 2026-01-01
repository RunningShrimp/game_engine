/**
 * Game Engine VSCode Extension
 *
 * 提供LSP（Language Server Protocol）和DAP（Debug Adapter Protocol）支持
 */

import * as vscode from 'vscode';
import * as net from 'net';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Trace
} from 'vscode-languageclient/node';
import { GameEngineDebugAdapterDescriptorFactory, GameEngineDebugAdapterSession } from './debugAdapter';

/**
 * 扩展上下文
 */
let extensionContext: vscode.ExtensionContext;
let languageClient: LanguageClient | undefined = undefined;

/**
 * 启动LSP服务器
 */
async function startLanguageServer(): Promise<void> {
    const config = vscode.workspace.getConfiguration('game-engine.lsp');

    // 检查是否启用LSP
    if (!config.get<boolean>('enabled', true)) {
        vscode.window.showInformationMessage('Game Engine LSP is disabled in settings');
        return;
    }

    const lspPath = config.get<string>('path', 'game-engine-lsp');
    const lspArgs: string[] = config.get<string[]>('args', []);
    const traceLevel: Trace = config.get<string>('trace.server', 'off') as Trace;

    // 服务器选项
    const serverOptions: ServerOptions = {
        command: lspPath,
        args: lspArgs,
        options: {
            env: {
                // 设置环境变量
                ...process.env
            }
        }
    };

    // 客户端选项
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'lua' },
            { scheme: 'file', language: 'typescript' },
            { scheme: 'file', language: 'javascript' },
            { scheme: 'file', language: 'python' },
            { scheme: 'file', language: 'rust' }
        ],
        synchronize: {
            configurationSection: 'game-engine.lsp',
            fileEvents: vscode.workspace.createFileSystemWatcher('**/.clientrc')
        },
        traceOutputChannel: traceLevel !== 'off' ? vscode.window.createOutputChannel('Game Engine LSP Trace') : undefined
    };

    // 创建并启动语言客户端
    languageClient = new LanguageClient(
        'game-engine-lsp',
        'Game Engine Language Server',
        serverOptions,
        clientOptions
    );

    await languageClient.start();

    vscode.window.showInformationMessage('Game Engine LSP Server started');
}

/**
 * 停止LSP服务器
 */
async function stopLanguageServer(): Promise<void> {
    if (languageClient) {
        await languageClient.stop();
        languageClient = undefined;
        vscode.window.showInformationMessage('Game Engine LSP Server stopped');
    }
}

/**
 * 重启LSP服务器
 */
async function restartLanguageServer(): Promise<void> {
    await stopLanguageServer();
    await startLanguageServer();
}

/**
 * 检查端口是否可用
 */
function isPortAvailable(port: number): Promise<boolean> {
    return new Promise((resolve) => {
        const server = net.createServer();

        server.once('error', () => {
            resolve(false);
        });

        server.once('listening', () => {
            server.close();
            resolve(true);
        });

        server.listen(port);
    });
}

/**
 * 获取可用的DAP端口
 */
async function getAvailableDapPort(): Promise<number> {
    const config = vscode.workspace.getConfiguration('game-engine.debug');
    let port = config.get<number>('port', 4711);

    // 如果默认端口不可用，尝试其他端口
    while (!(await isPortAvailable(port))) {
        port++;
        if (port > 65535) {
            throw new Error('No available ports found for DAP server');
        }
    }

    return port;
}

/**
 * 激活扩展
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
    extensionContext = context;

    console.log('Game Engine extension is now active');

    // 注册命令
    const startLspCommand = vscode.commands.registerCommand(
        'game-engine.startLSP',
        startLanguageServer
    );

    const stopLspCommand = vscode.commands.registerCommand(
        'game-engine.stopLSP',
        stopLanguageServer
    );

    const restartLspCommand = vscode.commands.registerCommand(
        'game-engine.restartLSP',
        restartLanguageServer
    );

    context.subscriptions.push(startLspCommand, stopLspCommand, restartLspCommand);

    // 注册调试适配器工厂
    const debugAdapterDescriptorFactory = new GameEngineDebugAdapterDescriptorFactory();
    context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory(
        'game-engine',
        debugAdapterDescriptorFactory
    ));

    // 注册调试配置提供者
    const debugConfigProvider = new GameEngineDebugConfigProvider();
    context.subscriptions.push(vscode.debug.registerDebugConfigurationProvider(
        'game-engine',
        debugConfigProvider
    ));

    // 自动启动LSP服务器（如果启用）
    const config = vscode.workspace.getConfiguration('game-engine.lsp');
    if (config.get<boolean>('enabled', true)) {
        await startLanguageServer();
    }

    vscode.window.showInformationMessage('Game Engine extension activated');
}

/**
 * 停用扩展
 */
export async function deactivate(): Promise<void> {
    await stopLanguageServer();
    console.log('Game Engine extension deactivated');
}

/**
 * 游戏引擎调试适配器描述符工厂
 */
class GameEngineDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(
        session: vscode.DebugSession
    ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        // 使用内联调试适配器（不需要外部进程）
        return new vscode.DebugAdapterInlineImplementation(
            new GameEngineDebugAdapterSession()
        );
    }
}

/**
 * 游戏引擎调试配置提供者
 */
class GameEngineDebugConfigProvider implements vscode.DebugConfigurationProvider {
    async resolveDebugConfiguration(
        folder: vscode.WorkspaceFolder | undefined,
        config: vscode.DebugConfiguration,
        token?: vscode.CancellationToken
    ): Promise<vscode.DebugConfiguration | undefined> {
        // 如果没有配置，返回默认配置
        if (!config) {
            return {
                type: 'game-engine',
                name: 'Launch Game Engine Script',
                request: 'launch',
                scriptPath: '${workspaceFolder}/main.lua',
                scriptLanguage: 'lua',
                cwd: '${workspaceFolder}'
            };
        }

        // 验证必需字段
        if (config.request === 'launch') {
            if (!config.scriptPath) {
                vscode.window.showErrorMessage('scriptPath is required for launch configuration');
                return undefined;
            }
        } else if (config.request === 'attach') {
            if (!config.port) {
                config.port = 4711;
            }
        }

        return config;
    }

    async resolveDebugConfigurationWithSubstitutedVariables(
        folder: vscode.WorkspaceFolder | undefined,
        config: vscode.DebugConfiguration,
        token?: vscode.CancellationToken
    ): Promise<vscode.DebugConfiguration | undefined> {
        // 替换变量后的配置处理
        return config;
    }
}
