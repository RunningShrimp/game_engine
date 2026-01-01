/**
 * Game Engine Debug Adapter Session
 *
 * 实现DAP（Debug Adapter Protocol）会话
 */

import * as vscode from 'vscode';
import {
    DebugSession,
    InitializedEvent,
    OutputEvent,
    TerminatedEvent,
    StoppedEvent,
    ContinuedEvent,
    ThreadEvent,
    BreakpointEvent,
    ModuleEvent,
    LoadedSourceEvent
} from 'vscode-debugadapter';
import { DebugProtocol } from '@vscode/debugprotocol';

/**
 * 游戏引擎调试会话
 */
export class GameEngineDebugAdapterSession extends DebugSession {
    private static THREAD_ID = 1;

    private _breakpoints = new Map<string, DebugProtocol.SourceBreakpoint[]>();
    private _variableHandles = new Map<number, string>();
    private _variableHandleCounter = 1000;

    /**
     * 构造函数
     */
    public constructor() {
        super();
    }

    /**
     * 会话初始化
     */
    protected initializeRequest(
        response: DebugProtocol.InitializeResponse,
        args: DebugProtocol.InitializeRequestArguments
    ): void {
        // 声明支持的DAP功能
        response.body = response.body || {};

        response.body.supportsConfigurationDoneRequest = true;
        response.body.supportsConditionalBreakpoints = true;
        response.body.supportsHitConditionalBreakpoints = false;
        response.body.supportsEvaluateForHovers = true;
        response.body.supportsStepBack = false;
        response.body.supportsSetVariable = true;
        response.body.supportsRestartRequest = false;
        response.body.supportsGotoTargetsRequest = false;
        response.body.supportsStepInTargetsRequest = false;
        response.body.supportsCompletionsRequest = true;
        response.body.supportsModulesRequest = false;
        response.body.supportsReadMemoryRequest = false;
        response.body.supportsDisassembleRequest = false;
        response.body.supportsCancelRequest = false;
        response.body.supportsBreakpointLocationsRequest = false;
        response.body.supportsClipboardContext = false;
        response.body.supportsSteppingGranularity = false;
        response.body.supportsInstructionBreakpoints = false;

        this.sendEvent(new InitializedEvent());
    }

    /**
     * 配置完成请求
     */
    protected configurationDoneRequest(
        response: DebugProtocol.ConfigurationDoneResponse,
        args: DebugProtocol.ConfigurationDoneArguments
    ): void {
        // 配置完成，可以开始调试
        response.success = true;
    }

    /**
     * 设置断点请求
     */
    protected setBreakPointsRequest(
        response: DebugProtocol.SetBreakpointsResponse,
        args: DebugProtocol.SetBreakpointsArguments
    ): void {
        const path = args.source.path as string;
        const lines = args.lines || [];

        // 存储断点
        const actualBreakpoints: DebugProtocol.SourceBreakpoint[] = lines.map((line, index) => ({
            id: index + 1,
            verified: true,
            line: line,
            column: 1
        }));

        this._breakpoints.set(path, actualBreakpoints);

        response.body = {
            breakpoints: actualBreakpoints
        };

        // 发送断点事件
        actualBreakpoints.forEach(bp => {
            this.sendEvent(new BreakpointEvent('changed', bp));
        });

        response.success = true;
    }

    /**
     * 启动/附加请求
     */
    protected launchOrAttachRequest(
        response: DebugProtocol.LaunchResponse | DebugProtocol.AttachResponse,
        args: any
    ): void {
        // 在实际实现中，这里会启动游戏引擎或附加到运行中的引擎
        response.success = true;
    }

    protected launchRequest(response: DebugProtocol.LaunchResponse, args: any): void {
        this.launchOrAttachRequest(response, args);
    }

    protected attachRequest(response: DebugProtocol.AttachResponse, args: any): void {
        this.launchOrAttachRequest(response, args);
    }

    /**
     * 断开连接请求
     */
    protected disconnectRequest(
        response: DebugProtocol.DisconnectResponse,
        args: DebugProtocol.DisconnectArguments
    ): void {
        response.success = true;
        this.sendEvent(new TerminatedEvent());
    }

    /**
     * 继续执行请求
     */
    protected continueRequest(
        response: DebugProtocol.ContinueResponse,
        args: DebugProtocol.ContinueArguments
    ): void {
        response.body = {
            allThreadsContinued: true
        };

        // 在实际实现中，这里会通知引擎继续执行
        this.sendEvent(new ContinuedEvent(GameEngineDebugAdapterSession.THREAD_ID, false));
    }

    /**
     * 暂停请求
     */
    protected pauseRequest(
        response: DebugProtocol.PauseResponse,
        args: DebugProtocol.PauseArguments
    ): void {
        // 在实际实现中，这里会暂停引擎执行
        // 然后发送停止事件
        this.sendEvent(new StoppedEvent('pause', GameEngineDebugAdapterSession.THREAD_ID));
    }

    /**
     * 单步执行请求
     */
    protected stepRequest(
        response: DebugProtocol.StepResponse,
        args: DebugProtocol.StepArguments
    ): void {
        const granularity = args.granularity || 'statement';

        // 根据step类型发送不同的停止事件
        let stopReason: string;
        switch (args.stepKind) {
            case 'next':
                stopReason = 'step';
                break;
            case 'in':
                stopReason = 'step';
                break;
            case 'out':
                stopReason = 'step';
                break;
            default:
                stopReason = 'step';
        }

        this.sendEvent(new StoppedEvent(stopReason, GameEngineDebugAdapterSession.THREAD_ID));
    }

    protected nextRequest(
        response: DebugProtocol.NextResponse,
        args: DebugProtocol.NextArguments
    ): void {
        this.stepRequest(response, { ...args, stepKind: 'next' });
    }

    protected stepInRequest(
        response: DebugProtocol.StepInResponse,
        args: DebugProtocol.StepInArguments
    ): void {
        this.stepRequest(response, { ...args, stepKind: 'in' });
    }

    protected stepOutRequest(
        response: DebugProtocol.StepOutResponse,
        args: DebugProtocol.StepOutArguments
    ): void {
        this.stepRequest(response, { ...args, stepKind: 'out' });
    }

    /**
     * 调用栈请求
     */
    protected stackTraceRequest(
        response: DebugProtocol.StackTraceResponse,
        args: DebugProtocol.StackTraceArguments
    ): void {
        // 在实际实现中，这里会从引擎获取真实的调用栈
        const stackFrames: DebugProtocol.StackFrame[] = [
            {
                id: 1,
                name: 'main',
                source: {
                    path: args.startFrame > 0 ? undefined : '${workspaceFolder}/main.lua',
                    name: 'main.lua'
                },
                line: 1,
                column: 1,
            }
        ];

        response.body = {
            stackFrames: stackFrames,
            totalFrames: stackFrames.length
        };
    }

    /**
     * 作用域请求
     */
    protected scopesRequest(
        response: DebugProtocol.ScopesResponse,
        args: DebugProtocol.ScopesArguments
    ): void {
        // 返回局部变量和全局变量作用域
        const scopes: DebugProtocol.Scope[] = [
            {
                name: 'Locals',
                variablesReference: this._variableHandleCounter++,
                expensive: false
            },
            {
                name: 'Globals',
                variablesReference: this._variableHandleCounter++,
                expensive: false
            }
        ];

        response.body = {
            scopes: scopes
        };
    }

    /**
     * 变量请求
     */
    protected variablesRequest(
        response: DebugProtocol.VariablesResponse,
        args: DebugProtocol.VariablesArguments
    ): void {
        const variablesReference = args.variablesReference;
        const variables: DebugProtocol.Variable[] = [];

        // 在实际实现中，这里会从引擎获取真实的变量
        if (variablesReference === 1000) {
            // 局部变量
            variables.push(
                { name: 'player', value: '{...}', variablesReference: 2001, type: 'table' },
                { name: 'deltaTime', value: '0.016', variablesReference: 0, type: 'number' }
            );
        } else if (variablesReference === 1001) {
            // 全局变量
            variables.push(
                { name: 'game', value: '{...}', variablesReference: 2002, type: 'table' },
                { name: 'DEBUG', value: 'true', variablesReference: 0, type: 'boolean' }
            );
        } else if (variablesReference === 2001) {
            // player对象的属性
            variables.push(
                { name: 'x', value: '100', variablesReference: 0, type: 'number' },
                { name: 'y', value: '200', variablesReference: 0, type: 'number' },
                { name: 'health', value: '100', variablesReference: 0, type: 'number' }
            );
        }

        response.body = {
            variables: variables
        };
    }

    /**
     * 求值请求
     */
    protected evaluateRequest(
        response: DebugProtocol.EvaluateResponse,
        args: DebugProtocol.EvaluateArguments
    ): void {
        const expression = args.expression;
        const context = args.context;

        // 在实际实现中，这里会真正求值表达式
        let result: string;
        let variablesReference = 0;

        if (context === 'hover' || context === 'watch') {
            // 简化的求值
            result = `"${expression}"`;
            variablesReference = this._variableHandleCounter++;
        } else {
            result = `"${expression}"`;
        }

        response.body = {
            result: result,
            variablesReference: variablesReference
        };
    }

    /**
     * 线程请求
     */
    protected threadsRequest(response: DebugProtocol.ThreadsResponse): void {
        response.body = {
            threads: [
                {
                    id: GameEngineDebugAdapterSession.THREAD_ID,
                    name: 'Main Thread'
                }
            ]
        };
    }

    /**
     * 设置变量请求
     */
    protected setVariableRequest(
        response: DebugProtocol.SetVariableResponse,
        args: DebugProtocol.SetVariableArguments
    ): void {
        const name = args.name;
        const value = args.value;

        // 在实际实现中，这里会在引擎中设置变量值
        response.body = {
            value: value,
            variablesReference: 0
        };
    }

    /**
     * 补全请求
     */
    protected completionsRequest(
        response: DebugProtocol.CompletionsResponse,
        args: DebugProtocol.CompletionsArguments
    ): void {
        const text = args.text;
        const column = args.column;

        // 简化的补全实现
        const completions: DebugProtocol.CompletionItem[] = [
            { label: 'local', type: 'keyword' },
            { label: 'function', type: 'keyword' },
            { label: 'if', type: 'keyword' },
            { label: 'then', type: 'keyword' },
            { label: 'end', type: 'keyword' },
            { label: 'true', type: 'value' },
            { label: 'false', type: 'value' },
            { label: 'nil', type: 'value' }
        ];

        response.body = {
            completions: completions
        };
    }
}

/**
 * 游戏引擎调试适配器描述符工厂（导出）
 */
export class GameEngineDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(
        session: vscode.DebugSession
    ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        return new vscode.DebugAdapterInlineImplementation(
            new GameEngineDebugAdapterSession()
        );
    }
}
