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