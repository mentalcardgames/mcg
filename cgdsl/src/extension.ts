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
    const serverBinary = process.platform === 'win32' ? 'lsp_server.exe' : 'lsp_server';
    const serverPath = path.join(context.extensionPath, 'bin', serverBinary);


    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    // 2. Options to control the language client
    const clientOptions: LanguageClientOptions = {
        // Register the server for your specific file extension
        documentSelector: [{ scheme: 'file', language: 'cgdsl' }],
        synchronize: {
            // Notify the server about file changes to '.cgdsl' files contained in the workspace
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.cgdsl')
        }
    };

    // 3. Create and start the client
    client = new LanguageClient(
        'cgdslServer',
        'CGDSL Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
    console.log('CGDSL Language Server is starting...');

     if (!vscode.workspace.workspaceFolders) {
        vscode.window.showErrorMessage('No workspace folder open');
        return;
    }

    const workspaceRoot = vscode.workspace.workspaceFolders[0].uri.fsPath;
    const outDir = path.join(workspaceRoot, 'cgdsl-output');

    if (!fs.existsSync(outDir)) {
        fs.mkdirSync(outDir);
    }

    // 1. Register the command that the user triggers (e.g., from a button or Command Palette)
    const runCommand = vscode.commands.registerCommand('cgdsl.runFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) return;

        // Define your output directory (e.g., a 'generated' folder in the workspace)
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0].uri.fsPath;
        const outDir = path.join(workspaceFolder || '', 'out');
        
        // Ensure outDir exists
        if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });

        const dotOutputPath = path.join(outDir, 'graph.dot');
        const jsonOutputPath = path.join(outDir, 'graph.json');

        const dotPath = path.join(outDir, 'graph.dot');
    const jsonPath = path.join(outDir, 'graph.json');
    const pngPath = path.join(outDir, 'graph.png');

    try {
        // 1. Request Rust to write the DOT file and return JSON data
        const graphData = await vscode.commands.executeCommand(
            'cgdsl.generateGraph', 
            dotPath 
        );

        if (graphData) {
            // 2. Save the JSON file locally
            fs.writeFileSync(jsonPath, JSON.stringify(graphData, null, 2));
            
            // 3. Open JSON in a split view for immediate inspection
            const doc = await vscode.workspace.openTextDocument(jsonPath);
            await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);

            // 4. Trigger Graphviz to compile the DOT into a PNG
            // We use double quotes around paths to handle spaces in folder names
            cp.exec(`dot -Tpng "${dotPath}" -o "${pngPath}"`, (error, stdout, stderr) => {
                if (error) {
                    vscode.window.showErrorMessage(`Graphviz Error: ${error.message}`);
                    return;
                }
                
                vscode.window.showInformationMessage(`Success! Saved JSON, DOT, and PNG to /out`);
                
                // Optional: Automatically open the generated image
                vscode.commands.executeCommand('vscode.open', vscode.Uri.file(pngPath));
            });
        }
    } catch (err) {
            vscode.window.showErrorMessage(`Failed: ${err}`);
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