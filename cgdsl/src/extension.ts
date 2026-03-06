import * as cp from 'child_process';
import path from 'path';
import * as vscode from 'vscode';
import * as fs from 'fs';

// Import the language client classes
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const platform = process.platform; 
    const binaryName = platform === 'win32' ? 'lsp_server.exe' : 'lsp_server';

    // Since Mac is now a single 'universal' file in bin/darwin/
    const serverPath = path.join(context.extensionPath, 'bin', platform, binaryName);

    if (platform !== 'win32' && fs.existsSync(serverPath)) {
        fs.chmodSync(serverPath, '755');
    }

    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'cgdsl' }],
        synchronize: {
            // This watcher still works! It just watches the base filesystem if no workspace is active.
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.cgdsl')
        }
    };

    client = new LanguageClient(
        'cgdslServer',
        'CGDSL Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    const runCommand = vscode.commands.registerCommand('cgdsl.runFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor found.');
            return;
        }

        const filePath = editor.document.uri.fsPath;
        const fileDir = path.dirname(filePath);
        const fileName = path.basename(filePath, path.extname(filePath));
        
        // Define the output directory
        const outDir = path.join(fileDir, 'cgdsl-output');
        
        try {
            if (!fs.existsSync(outDir)) {
                fs.mkdirSync(outDir, { recursive: true });
            }

            // Rust will append .dot and .svg to this.
            // Define the output folder and base filename
            const baseOutputPath = path.join(outDir, fileName); // No extension here!

            const jsonPath = path.join(outDir, 'game.json');

            // Rust will turn baseOutputPath into baseOutputPath.dot and baseOutputPath.svg
            const graphData = await vscode.commands.executeCommand('cgdsl.generateGraph', baseOutputPath);

            if (graphData) {
                fs.writeFileSync(jsonPath, JSON.stringify(graphData, null, 2));
                const jsonDoc = await vscode.workspace.openTextDocument(jsonPath);
                await vscode.window.showTextDocument(jsonDoc, vscode.ViewColumn.One);
            } else {
                vscode.window.showErrorMessage("LSP returned no data. Check the Rust logs.");
            }

            // 2. Now TypeScript just needs to know where to find them to open them
            const svgPath = path.join(outDir, `${fileName}.svg`);
            const svgUri = vscode.Uri.file(svgPath);
            await vscode.commands.executeCommand('vscode.open', svgUri, vscode.ViewColumn.Two);

            const dotPath = path.join(outDir, `${fileName}.dot`);
            const dotUri = vscode.Uri.file(svgPath);
            await vscode.commands.executeCommand('vscode.open', dotUri, vscode.ViewColumn.Two);
        } catch (err) {
            vscode.window.showErrorMessage(`Failed to generate graph: ${err}`);
        }
    });

    context.subscriptions.push(runCommand);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}