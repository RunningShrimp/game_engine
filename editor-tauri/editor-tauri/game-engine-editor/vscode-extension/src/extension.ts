import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

/**
 * Activate the extension
 */
export async function activate(context: vscode.ExtensionContext) {
    console.log('Game Engine extension is now active!');

    // Check if LSP is enabled
    const config = vscode.workspace.getConfiguration('gameEngine');
    const enabled = config.get<boolean>('lsp.enabled', true);

    if (!enabled) {
        console.log('Game Engine LSP is disabled in settings');
        return;
    }

    // Register commands
    const restartCommand = vscode.commands.registerCommand(
        'gameEngine.restartLSP',
        restartLSP
    );
    context.subscriptions.push(restartCommand);

    const showDocsCommand = vscode.commands.registerCommand(
        'gameEngine.showDocumentation',
        showDocumentation
    );
    context.subscriptions.push(showDocsCommand);

    const openPlaygroundCommand = vscode.commands.registerCommand(
        'gameEngine.openPlayground',
        openPlayground
    );
    context.subscriptions.push(openPlaygroundCommand);

    const runDiagnosticsCommand = vscode.commands.registerCommand(
        'gameEngine.runDiagnostics',
        runDiagnostics
    );
    context.subscriptions.push(runDiagnosticsCommand);

    const showPerformanceCommand = vscode.commands.registerCommand(
        'gameEngine.showPerformance',
        showPerformance
    );
    context.subscriptions.push(showPerformanceCommand);

    // Start the LSP client
    await startLSP(context);
}

/**
 * Start the LSP client
 */
async function startLSP(context: vscode.ExtensionContext) {
    // If client is already running, stop it first
    if (client) {
        await stopLSP();
    }

    const config = vscode.workspace.getConfiguration('gameEngine');

    // Determine the LSP server path
    let lspPath = config.get<string>('lsp.path');
    if (!lspPath) {
        // Try to find game-engine-lsp in PATH
        lspPath = 'game-engine-lsp';
    }

    const lspArgs = config.get<string[]>('lsp.args', []);

    // Server options
    const serverOptions: ServerOptions = {
        command: lspPath,
        args: lspArgs,
        options: {
            env: {
                ...process.env,
                RUST_LOG: 'info',
            },
        },
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'rust' },
            { scheme: 'file', language: 'toml' }, // For Cargo.toml
        ],
        diagnosticCollectionName: 'gameEngine',
        progressOnInitialization: true,
        outputChannelName: 'Game Engine LSP',
        initializationOptions: {
            maxNumberOfProblems: config.get<number>('lsp.maxNumberOfProblems', 100),
        },
    };

    // Create and start the client
    client = new LanguageClient(
        'gameEngine',
        'Game Engine Language Server',
        serverOptions,
        clientOptions
    );

    // Register the disposable
    context.subscriptions.push(client);

    try {
        console.log(`Starting Game Engine LSP: ${lspPath}`);
        await client.start();
        console.log('Game Engine LSP started successfully!');

        vscode.window.showInformationMessage('Game Engine Language Server started 🚀');
    } catch (error) {
        console.error('Failed to start Game Engine LSP:', error);
        vscode.window.showErrorMessage(
            `Failed to start Game Engine Language Server: ${error}`
        );
    }
}

/**
 * Stop the LSP client
 */
async function stopLSP() {
    if (client) {
        try {
            await client.stop();
            console.log('Game Engine LSP stopped');
        } catch (error) {
            console.error('Error stopping LSP client:', error);
        }
        client = undefined;
    }
}

/**
 * Restart the LSP server
 */
async function restartLSP() {
    if (!client) {
        vscode.window.showWarningMessage('LSP server is not running');
        return;
    }

    vscode.window.showInformationMessage('Restarting Game Engine Language Server...');

    try {
        await stopLSP();
        await startLSP(vscode.extensions.getExtension('game-engine.game-engine-vscode')!.activationTimes);
        vscode.window.showInformationMessage('Game Engine Language Server restarted ✅');
    } catch (error) {
        vscode.window.showErrorMessage(`Failed to restart LSP server: ${error}`);
    }
}

/**
 * Show documentation
 */
async function showDocumentation() {
    const docUrl = 'https://docs.game-engine.dev';
    vscode.env.openExternal(vscode.Uri.parse(docUrl));
}

/**
 * Open playground
 */
async function openPlayground() {
    // Create a new untitled file with game engine template
    const doc = await vscode.workspace.openTextDocument({
        language: 'rust',
        content: `//! Game Engine Playground
//!
//! This is a scratchpad for testing game engine features

use game_engine::prelude::*;

fn main() {
    // Your game code here

    // Create a new entity
    let entity = Entity::new();

    // Add transform component
    entity.add_component(Transform::default());

    println!("Game Engine Playground 🎮");
}
`,
    });
    await vscode.window.showTextDocument(doc);
}

/**
 * Run diagnostics
 */
async function runDiagnostics() {
    if (!client) {
        vscode.window.showWarningMessage('LSP server is not running');
        return;
    }

    vscode.window.showInformationMessage('Running diagnostics...');
    // Diagnostics run automatically on file changes
    // This command just informs the user
}

/**
 * Show performance statistics
 */
async function showPerformance() {
    if (!client) {
        vscode.window.showWarningMessage('LSP server is not running');
        return;
    }

    // Request performance stats from the LSP server
    const stats = await client.sendRequest<any>('gameEngine/performance', {});

    const message = `
Game Engine LSP Performance:
- Request Count: ${stats.requestCount || 'N/A'}
- Average Response Time: ${stats.avgResponseTime || 'N/A'}ms
- Memory Usage: ${stats.memoryUsage || 'N/A'}MB
    `.trim();

    vscode.window.showInformationMessage(message);
}

/**
 * Deactivate the extension
 */
export async function deactivate() {
    console.log('Deactivating Game Engine extension');
    await stopLSP();
}
