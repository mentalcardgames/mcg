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

    // 1. Register the command that the user triggers (e.g., from a button or Command Palette)
    const runCommand = vscode.commands.registerCommand('cgdsl.runFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor found.');
            return;
        }

        // 1. Get the directory of the currently open file
        const filePath = editor.document.uri.fsPath;
        const fileDir = path.dirname(filePath);

        // 2. Define the output directory relative to the file
        // This creates a folder named 'cgdsl-output' right next to your .cgdsl file
        const outDir = path.join(fileDir, 'cgdsl-output');
        
        try {
            // Ensure outDir exists (recursive: true handles parent folders if needed)
            if (!fs.existsSync(outDir)) {
                fs.mkdirSync(outDir, { recursive: true });
            }

            const dotPath = path.join(outDir, 'graph.dot');
            const jsonPath = path.join(outDir, 'graph.json');
            const pngPath = path.join(outDir, 'graph.png');

            // 3. Request your LSP/Server to generate the data
            const graphData = await vscode.commands.executeCommand('cgdsl.generateGraph', dotPath);

            if (graphData) {
                fs.writeFileSync(jsonPath, JSON.stringify(graphData, null, 2));
                
                const doc = await vscode.workspace.openTextDocument(jsonPath);
                await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);

                // 4. Execute Graphviz
                cp.exec(`dot -Tpng "${dotPath}" -o "${pngPath}"`, (error) => {
                    if (error) {
                        vscode.window.showErrorMessage(`Graphviz Error: ${error.message}`);
                        return;
                    }
                    vscode.window.showInformationMessage(`Generated files in: ${outDir}`);
                    vscode.commands.executeCommand('vscode.open', vscode.Uri.file(pngPath));
                });
            }
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