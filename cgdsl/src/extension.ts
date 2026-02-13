// import * as vscode from 'vscode';
// import * as path from 'path';
// import * as fs from 'fs'; // Add this import
// import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

// let client: LanguageClient;

// export async function activate(context: vscode.ExtensionContext) {
//     console.log('Activating CGDSL extension...');

//     const serverBinary = process.platform === 'win32' ? 'lsp_server.exe' : 'lsp_server';
//     const serverPath = path.join(context.extensionPath, 'bin', serverBinary);

//     console.log(`Checking for binary at: ${serverPath}`);

//     if (!fs.existsSync(serverPath)) {
//         console.error('Binary NOT found!');
//         vscode.window.showErrorMessage(`LSP Server missing at: ${serverPath}`);
//         return;
//     }

//     const serverOptions: ServerOptions = {
//         run: { command: serverPath, transport: TransportKind.stdio },
//         debug: { command: serverPath, transport: TransportKind.stdio }
//     };

//     const clientOptions: LanguageClientOptions = {
//         documentSelector: [{ scheme: 'file', language: 'cgdsl' }],
//         // Ensure this matches your language ID in package.json
//     };

//     client = new LanguageClient('cgdslServer', 'CGDSL Server', serverOptions, clientOptions);
    
//     try {
//         await client.start();
//         console.log('LSP Server started successfully.');
//     } catch (e) {
//         console.error('Failed to start LSP Server:', e);
//     }
// }


import path from 'path';
import * as vscode from 'vscode';
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
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}