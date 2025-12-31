// VS Code扩展主文件
// Game Engine Debugger 扩展入口

import * as vscode from 'vscode';
import * as net from 'net';

/**
 * 调试适配器会话类型
 */
type Session = {
    socket: net.Socket;
    disposable: vscode.Disposable;
};

/**
 * 扩展上下文
 */
let extensionContext: vscode.ExtensionContext;

/**
 * 当前调试会话
 */
let currentSession: Session | undefined;

/**
 * 激活扩展
 */
export function activate(context: vscode.ExtensionContext) {
    extensionContext = context;

    // 注册调试适配器描述符工厂
    const factory = new GameEngineDebugAdapterDescriptorFactory();
    context.subscriptions.push(
        vscode.debug.registerDebugAdapterDescriptorFactory('game-engine', factory)
    );

    // 注册命令
    context.subscriptions.push(
        vscode.commands.registerCommand('gameEngine.toggleBreakpoint', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                const line = editor.selection.active.line;
                vscode.debug.toggleBreakpoint(editor.document.uri, line);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('gameEngine.addWatch', async () => {
            const expression = await vscode.window.showInputBox({
                prompt: 'Enter watch expression',
                placeHolder: 'e.g., myVariable'
            });

            if (expression) {
                vscode.debug.addWatchExpression(expression);
            }
        })
    );

    console.log('Game Engine Debugger extension activated');
}

/**
 * 停用扩展
 */
export function deactivate() {
    if (currentSession) {
        currentSession.socket.destroy();
        currentSession.disposable.dispose();
    }
}

/**
 * Game Engine调试适配器描述符工厂
 */
class GameEngineDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(
        session: vscode.DebugSession,
        executable: vscode.DebugAdapterExecutable | undefined
    ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        // 使用服务器模式
        return new vscode.DebugAdapterServerImplementation(4711, '127.0.0.1');
    }
}

/**
 * DAP消息接口
 */
interface DapMessage {
    seq: number;
    type: string;
    request_seq?: number;
    success?: boolean;
    command?: string;
    arguments?: any;
    message?: string;
    body?: any;
}

/**
 * Game Engine调试适配器实现（可选：直接实现而非使用服务器）
 */
class GameEngineDebugAdapter extends vscode.DebugAdapter {
    private readonly _onRequest = new vscode.EventEmitter<DapMessage>();
    private readonly _onResponse = new vscode.EventEmitter<DapMessage>();
    private readonly _onEvent = new vscode.EventEmitter<DapMessage>();

    readonly onRequest = this._onRequest.event;
    readonly onResponse = this._onResponse.event;
    readonly onEvent = this._onEvent.event;

    private _sequence = 1;
    private _breakpoints = new Map<string, vscode.DebugBreakpoint[]>();
    private _variableHandles = 0;
    private _variables = new Map<number, any>();

    /**
     * 处理连接
     */
    connect(): void {
        // 连接到游戏引擎DAP服务器
    }

    /**
     * 断开连接
     */
    disconnect(): void {
        this.shutdown();
    }

    /**
     * 发送消息
     */
    send(message: DapMessage): void {
        this._onResponse.fire(message);
    }

    /**
     * 接收消息
     */
    receive(message: DapMessage): void {
        this._onRequest.fire(message);
    }

    /**
     * 处理请求
     */
    protected async handleRequest(request: DapMessage): Promise<DapMessage> {
        const response: DapMessage = {
            seq: this._sequence++,
            type: 'response',
            request_seq: request.seq,
            success: true,
            command: request.command,
            body: undefined
        };

        switch (request.command) {
            case 'initialize':
                response.body = {
                    capabilities: {
                        supportsConfigurationDoneRequest: true,
                        supportsFunctionBreakpoints: true,
                        supportsConditionalBreakpoints: true,
                        supportsEvaluateForHovers: true,
                        supportsSetVariable: true,
                        supportsCompletionsRequest: true,
                        supportsLogPoints: true,
                    }
                };
                break;

            case 'setBreakpoints':
                const args = request.arguments as {
                    source: { path: string };
                    breakpoints: { line: number; condition?: string }[];
                };
                response.body = await this.setBreakpoints(args.source.path, args.breakpoints);
                break;

            case 'setFunctionBreakpoints':
                // TODO: 实现函数断点
                response.body = { breakpoints: [] };
                break;

            case 'setExceptionBreakpoints':
                // TODO: 实现异常断点
                response.body = { breakpoints: [] };
                break;

            case 'configurationDone':
                // 配置完成，可以开始调试
                break;

            case 'launch':
                // 启动调试会话
                break;

            case 'attach':
                // 附加到运行中的进程
                break;

            case 'continue':
            case 'next':
            case 'stepIn':
            case 'stepOut':
                response.body = { allThreadsContinued: true };
                this._onEvent.fire({
                    seq: this._sequence++,
                    type: 'event',
                    event: 'continued',
                    body: { threadId: 1, allThreadsContinued: true }
                });
                break;

            case 'pause':
                this._onEvent.fire({
                    seq: this._sequence++,
                    type: 'event',
                    event: 'stopped',
                    body: {
                        reason: 'pause',
                        threadId: 1,
                        allThreadsStopped: true
                    }
                });
                break;

            case 'stackTrace':
                response.body = await this.getStackTrace();
                break;

            case 'scopes':
                response.body = await this.getScopes();
                break;

            case 'variables':
                const varsArgs = request.arguments as { variablesReference: number };
                response.body = await this.getVariables(varsArgs.variablesReference);
                break;

            case 'setVariable':
                // TODO: 实现变量设置
                break;

            case 'evaluate':
                const evalArgs = request.arguments as { expression: string };
                response.body = await this.evaluate(evalArgs.expression);
                break;

            case 'threads':
                response.body = { threads: [{ id: 1, name: 'Main Thread' }] };
                break;

            case 'terminate':
            case 'disconnect':
                this.shutdown();
                break;

            default:
                response.success = false;
                response.message = `Unknown command: ${request.command}`;
        }

        return response;
    }

    /**
     * 设置断点
     */
    private async setBreakpoints(
        sourcePath: string,
        breakpoints: { line: number; condition?: string }[]
    ): Promise<{ breakpoints: vscode.DebugBreakpoint[] }> {
        const bps: vscode.DebugBreakpoint[] = [];

        for (const bp of breakpoints) {
            const verified = true; // TODO: 实际验证断点
            bps.push(new vscode.DebugBreakpoint(verified, bp.line, 0));
        }

        this._breakpoints.set(sourcePath, bps);
        return { breakpoints: bps };
    }

    /**
     * 获取调用栈
     */
    private async getStackTrace(): Promise<{ stackFrames: vscode.DebugStackFrame[]; totalFrames: number }> {
        // TODO: 实际获取调用栈
        return {
            stackFrames: [
                new vscode.DebugStackFrame(
                    'main',
                    new vscode.Source('script.lua', sourcePath),
                    1,
                    0
                )
            ],
            totalFrames: 1
        };
    }

    /**
     * 获取作用域
     */
    private async getScopes(): Promise<{ scopes: vscode.DebugScope[] }> {
        const localsRef = this._variableHandles++;
        return {
            scopes: [
                new vscode.DebugScope('Locals', localsRef, false),
                new vscode.DebugScope('Globals', this._variableHandles++, false)
            ]
        };
    }

    /**
     * 获取变量
     */
    private async getVariables(variablesReference: number): Promise<{ variables: vscode.DebugVariable[] }> {
        // TODO: 实际获取变量
        if (variablesReference === 0) {
            return { variables: [] };
        }

        return {
            variables: [
                new vscode.DebugVariable('x', '42', 'number'),
                new vscode.DebugVariable('y', '3.14', 'number'),
                new vscode.DebugVariable('name', '"test"', 'string')
            ]
        };
    }

    /**
     * 求值表达式
     */
    private async evaluate(expression: string): Promise<{ result: string; type?: string; variablesReference: number }> {
        // TODO: 实际表达式求值
        return {
            result: `<${expression}>`,
            type: 'unknown',
            variablesReference: 0
        };
    }

    /**
     * 关闭调试会话
     */
    private shutdown(): void {
        this._breakpoints.clear();
        this._variables.clear();
    }
}

/**
 * 配置提供程序
 */
export class GameEngineConfigurationProvider implements vscode.DebugConfigurationProvider {
    /**
     * 解析调试配置
     */
    resolveDebugConfiguration(
        folder: vscode.WorkspaceFolder | undefined,
        config: vscode.DebugConfiguration,
        token?: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.DebugConfiguration> {
        // 提供默认配置
        if (!config.scriptLanguage) {
            config.scriptLanguage = this.detectScriptLanguage();
        }

        if (!config.script && !config.program) {
            // 如果没有指定脚本，尝试使用当前文件
            const activeEditor = vscode.window.activeTextEditor;
            if (activeEditor) {
                config.script = activeEditor.document.uri.fsPath;
            }
        }

        return config;
    }

    /**
     * 检测脚本语言
     */
    private detectScriptLanguage(): string {
        const activeEditor = vscode.window.activeTextEditor;
        if (activeEditor) {
            const fileName = activeEditor.document.fileName.toLowerCase();
            if (fileName.endsWith('.lua')) {
                return 'lua';
            } else if (fileName.endsWith('.ts')) {
                return 'typescript';
            } else if (fileName.endsWith('.js')) {
                return 'javascript';
            } else if (fileName.endsWith('.py')) {
                return 'python';
            }
        }
        return 'lua'; // 默认
    }
}

/**
 * 提供调试配置初始模板
 */
export function provideDebugConfigurations(): vscode.ProviderResult<vscode.DebugConfiguration[]> {
    return [
        {
            type: 'game-engine',
            request: 'launch',
            name: 'Debug Lua Script',
            script: '${file}',
            scriptLanguage: 'lua'
        },
        {
            type: 'game-engine',
            request: 'launch',
            name: 'Debug TypeScript Script',
            script: '${file}',
            scriptLanguage: 'typescript'
        },
        {
            type: 'game-engine',
            request: 'launch',
            name: 'Debug Python Script',
            script: '${file}',
            scriptLanguage: 'python'
        }
    ];
}
