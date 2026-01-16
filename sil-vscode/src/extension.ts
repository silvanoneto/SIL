import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let silClient: LanguageClient | undefined;
let lisClient: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('SIL & LIS Language Support is now active!');

    // Register commands
    registerCommands(context);

    // Start SIL language client
    const silConfig = vscode.workspace.getConfiguration('sil');
    if (silConfig.get<boolean>('lsp.enabled', true)) {
        startSilLanguageClient(context);
    }

    // Start LIS language client
    const lisConfig = vscode.workspace.getConfiguration('lis');
    if (lisConfig.get<boolean>('lsp.enabled', true)) {
        startLisLanguageClient(context);
    }

    // Register status bar
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        100
    );
    statusBarItem.text = '$(circuit-board) SIL';
    statusBarItem.tooltip = 'SIL Language Support';
    statusBarItem.command = 'sil.showInfo';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Watch for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('sil.lsp.enabled')) {
                const enabled = vscode.workspace
                    .getConfiguration('sil')
                    .get<boolean>('lsp.enabled', true);
                if (enabled && !silClient) {
                    startSilLanguageClient(context);
                } else if (!enabled && silClient) {
                    stopSilLanguageClient();
                }
            }
            if (e.affectsConfiguration('lis.lsp.enabled')) {
                const enabled = vscode.workspace
                    .getConfiguration('lis')
                    .get<boolean>('lsp.enabled', true);
                if (enabled && !lisClient) {
                    startLisLanguageClient(context);
                } else if (!enabled && lisClient) {
                    stopLisLanguageClient();
                }
            }
        })
    );
}

export function deactivate(): Thenable<void> | undefined {
    return stopLanguageClient();
}

function registerCommands(context: vscode.ExtensionContext) {
    // New Program command
    context.subscriptions.push(
        vscode.commands.registerCommand('sil.newProgram', async () => {
            const doc = await vscode.workspace.openTextDocument({
                language: 'sil',
                content: `; New SIL Program
.mode SIL-128

.code
main:
    NOP
    HLT
`
            });
            await vscode.window.showTextDocument(doc);
        })
    );

    // Assemble command
    context.subscriptions.push(
        vscode.commands.registerCommand('sil.assemble', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'sil') {
                vscode.window.showWarningMessage('No SIL file is open');
                return;
            }

            const filePath = editor.document.fileName;
            const outputPath = filePath.replace(/\.sil$/, '.silc');

            // Run silasm (external tool)
            const terminal = vscode.window.createTerminal('SIL Assembler');
            terminal.show();
            terminal.sendText(`silasm "${filePath}" -o "${outputPath}"`);
        })
    );

    // Run command
    context.subscriptions.push(
        vscode.commands.registerCommand('sil.run', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'sil') {
                vscode.window.showWarningMessage('No SIL file is open');
                return;
            }

            const filePath = editor.document.fileName;

            // Run vsp (external tool)
            const terminal = vscode.window.createTerminal('SIL Runner');
            terminal.show();
            terminal.sendText(`vsp run "${filePath}"`);
        })
    );

    // Debug command
    context.subscriptions.push(
        vscode.commands.registerCommand('sil.debug', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'sil') {
                vscode.window.showWarningMessage('No SIL file is open');
                return;
            }

            // Start debug session
            vscode.debug.startDebugging(undefined, {
                type: 'sil',
                request: 'launch',
                name: 'Debug SIL',
                program: editor.document.fileName,
                stopOnEntry: vscode.workspace
                    .getConfiguration('sil')
                    .get<boolean>('debug.stopOnEntry', false)
            });
        })
    );

    // REPL command
    context.subscriptions.push(
        vscode.commands.registerCommand('sil.repl', async () => {
            const terminal = vscode.window.createTerminal({
                name: 'SIL REPL',
                shellPath: 'vsp',
                shellArgs: ['repl']
            });
            terminal.show();
        })
    );

    // Disassemble command
    context.subscriptions.push(
        vscode.commands.registerCommand('sil.disassemble', async () => {
            const files = await vscode.window.showOpenDialog({
                canSelectMany: false,
                filters: {
                    'SIL Bytecode': ['silc']
                }
            });

            if (files && files.length > 0) {
                const terminal = vscode.window.createTerminal('SIL Disassembler');
                terminal.show();
                terminal.sendText(`silasm -d "${files[0].fsPath}"`);
            }
        })
    );

    // Show info command
    context.subscriptions.push(
        vscode.commands.registerCommand('sil.showInfo', async () => {
            const config = vscode.workspace.getConfiguration('sil');
            const mode = config.get<string>('mode', 'SIL-128');
            const lspEnabled = config.get<boolean>('lsp.enabled', true);

            vscode.window.showInformationMessage(
                `SIL Mode: ${mode} | LSP: ${lspEnabled ? 'enabled' : 'disabled'}`
            );
        })
    );

    // ========== LIS COMMANDS ==========

    // LIS: New Program
    context.subscriptions.push(
        vscode.commands.registerCommand('lis.newProgram', async () => {
            const doc = await vscode.workspace.openTextDocument({
                language: 'lis',
                content: `// New LIS Program

fn main() {
    // Your code here
}
`
            });
            await vscode.window.showTextDocument(doc);
        })
    );

    // LIS: Compile to SIL
    context.subscriptions.push(
        vscode.commands.registerCommand('lis.compile', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'lis') {
                vscode.window.showWarningMessage('No LIS file is open');
                return;
            }

            const filePath = editor.document.fileName;
            const outputPath = filePath.replace(/\.lis$/, '.sil');

            const config = vscode.workspace.getConfiguration('lis');
            const compilerPath = config.get<string>('compiler.path', 'lis');
            const silMode = config.get<string>('silMode', 'SIL-128');

            const terminal = vscode.window.createTerminal('LIS Compiler');
            terminal.show();
            terminal.sendText(`${compilerPath} compile "${filePath}" -o "${outputPath}" --target ${silMode}`);
        })
    );

    // LIS: Build to Bytecode
    context.subscriptions.push(
        vscode.commands.registerCommand('lis.build', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'lis') {
                vscode.window.showWarningMessage('No LIS file is open');
                return;
            }

            const filePath = editor.document.fileName;
            const outputPath = filePath.replace(/\.lis$/, '.silc');

            const config = vscode.workspace.getConfiguration('lis');
            const compilerPath = config.get<string>('compiler.path', 'lis');
            const silMode = config.get<string>('silMode', 'SIL-128');
            const optLevel = config.get<string>('optimizationLevel', 'O2');

            const terminal = vscode.window.createTerminal('LIS Builder');
            terminal.show();
            terminal.sendText(`${compilerPath} build "${filePath}" -o "${outputPath}" --target ${silMode} -${optLevel}`);
        })
    );

    // LIS: Run
    context.subscriptions.push(
        vscode.commands.registerCommand('lis.run', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'lis') {
                vscode.window.showWarningMessage('No LIS file is open');
                return;
            }

            const filePath = editor.document.fileName;

            const config = vscode.workspace.getConfiguration('lis');
            const compilerPath = config.get<string>('compiler.path', 'lis');

            const terminal = vscode.window.createTerminal('LIS Runner');
            terminal.show();
            terminal.sendText(`${compilerPath} run "${filePath}"`);
        })
    );

    // Register completion providers
    context.subscriptions.push(
        vscode.languages.registerCompletionItemProvider('sil', new SilCompletionProvider())
    );
    context.subscriptions.push(
        vscode.languages.registerCompletionItemProvider('lis', new LisCompletionProvider())
    );

    // Register formatters
    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider('lis', new LisFormattingProvider())
    );
    context.subscriptions.push(
        vscode.languages.registerDocumentRangeFormattingEditProvider('lis', new LisRangeFormattingProvider())
    );
}

function startSilLanguageClient(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('sil');
    const customPath = config.get<string>('lsp.path', '');

    // Server command - try custom path, then common locations, then PATH
    const serverCommand = customPath || findSilLsp() || 'sil-lsp';

    const serverOptions: ServerOptions = {
        run: {
            command: serverCommand,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverCommand,
            transport: TransportKind.stdio,
            args: ['--debug']
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'sil' }
        ],
        synchronize: {
            configurationSection: 'sil',
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.sil')
        },
        outputChannelName: 'SIL Language Server',
        initializationOptions: {
            mode: config.get<string>('mode', 'SIL-128'),
            format: {
                alignOperands: config.get<boolean>('format.alignOperands', true),
                uppercaseOpcodes: config.get<boolean>('format.uppercaseOpcodes', true)
            }
        }
    };

    silClient = new LanguageClient(
        'sil',
        'SIL Language Server',
        serverOptions,
        clientOptions
    );

    // Handle client errors gracefully
    silClient.onDidChangeState(e => {
        if (e.newState === 1) { // Stopped
            console.log('SIL Language Server stopped');
        }
    });

    // Start the client
    silClient.start().catch(err => {
        console.error('Failed to start SIL Language Server:', err);
        vscode.window.showWarningMessage(
            `SIL Language Server not found. Install sil-lsp or set sil.lsp.path in settings. Error: ${err.message}`
        );
    });

    context.subscriptions.push(silClient);
    console.log('SIL Language Server started with:', serverCommand);
}

function startLisLanguageClient(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('lis');
    const customPath = config.get<string>('lsp.path', '');

    // Server command - try custom path, then common locations, then PATH
    const serverCommand = customPath || findLisLsp() || 'lis-lsp';

    const serverOptions: ServerOptions = {
        run: {
            command: serverCommand,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverCommand,
            transport: TransportKind.stdio,
            args: ['--debug']
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'lis' }
        ],
        synchronize: {
            configurationSection: 'lis',
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lis')
        },
        outputChannelName: 'LIS Language Server',
        initializationOptions: {
            silMode: config.get<string>('silMode', 'SIL-128'),
            optimizationLevel: config.get<string>('optimizationLevel', 'O2')
        }
    };

    lisClient = new LanguageClient(
        'lis',
        'LIS Language Server',
        serverOptions,
        clientOptions
    );

    // Handle client errors gracefully
    lisClient.onDidChangeState(e => {
        if (e.newState === 1) { // Stopped
            console.log('LIS Language Server stopped');
        }
    });

    // Start the client
    lisClient.start().catch(err => {
        console.error('Failed to start LIS Language Server:', err);
        vscode.window.showWarningMessage(
            `LIS Language Server not found. Install lis-lsp or set lis.lsp.path in settings. Error: ${err.message}`
        );
    });

    context.subscriptions.push(lisClient);
    console.log('LIS Language Server started with:', serverCommand);
}

function findSilLsp(): string | undefined {
    const { existsSync } = require('fs');
    const { join } = require('path');
    const home = process.env.HOME || process.env.USERPROFILE || '';

    const candidates = [
        join(home, '.local', 'bin', 'sil-lsp'),
        join(home, '.cargo', 'bin', 'sil-lsp'),
        '/usr/local/bin/sil-lsp',
        '/opt/homebrew/bin/sil-lsp',
    ];

    for (const path of candidates) {
        if (existsSync(path)) {
            return path;
        }
    }

    return undefined;
}

function findLisLsp(): string | undefined {
    const { existsSync } = require('fs');
    const { join } = require('path');
    const home = process.env.HOME || process.env.USERPROFILE || '';

    const candidates = [
        join(home, '.local', 'bin', 'lis-lsp'),
        join(home, '.cargo', 'bin', 'lis-lsp'),
        '/usr/local/bin/lis-lsp',
        '/opt/homebrew/bin/lis-lsp',
    ];

    for (const path of candidates) {
        if (existsSync(path)) {
            return path;
        }
    }

    return undefined;
}

function stopLanguageClient(): Thenable<void> | undefined {
    const promises: Thenable<void>[] = [];

    if (silClient) {
        promises.push(
            silClient.stop().then(() => {
                silClient = undefined;
                console.log('SIL Language Server stopped');
            })
        );
    }

    if (lisClient) {
        promises.push(
            lisClient.stop().then(() => {
                lisClient = undefined;
                console.log('LIS Language Server stopped');
            })
        );
    }

    if (promises.length === 0) {
        return undefined;
    }

    return Promise.all(promises).then(() => {});
}

function stopSilLanguageClient(): Thenable<void> | undefined {
    if (!silClient) {
        return undefined;
    }
    return silClient.stop().then(() => {
        silClient = undefined;
        console.log('SIL Language Server stopped');
    });
}

function stopLisLanguageClient(): Thenable<void> | undefined {
    if (!lisClient) {
        return undefined;
    }
    return lisClient.stop().then(() => {
        lisClient = undefined;
        console.log('LIS Language Server stopped');
    });
}

// Register completion provider for when LSP is not available
class SilCompletionProvider implements vscode.CompletionItemProvider {
    provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position
    ): vscode.CompletionItem[] {
        const items: vscode.CompletionItem[] = [];

        // Opcodes
        const opcodes = [
            'NOP', 'HLT', 'RET', 'YIELD', 'JMP', 'JZ', 'JN', 'JC', 'JO', 'CALL', 'LOOP',
            'MOV', 'MOVI', 'LOAD', 'STORE', 'PUSH', 'POP', 'XCHG', 'LSTATE', 'SSTATE',
            'MUL', 'DIV', 'POW', 'ROOT', 'INV', 'CONJ', 'ADD', 'SUB', 'MAG', 'PHASE', 'SCALE', 'ROTATE',
            'XORL', 'ANDL', 'ORL', 'NOTL', 'SHIFTL', 'ROTATL', 'FOLD', 'SPREAD', 'GATHER',
            'TRANS', 'PIPE', 'LERP', 'SLERP', 'GRAD', 'DESCENT', 'EMERGE', 'COLLAPSE',
            'SETMODE', 'PROMOTE', 'DEMOTE', 'TRUNCATE', 'XORDEM', 'AVGDEM', 'MAXDEM', 'COMPAT',
            'IN', 'OUT', 'SENSE', 'ACT', 'SYNC', 'BROADCAST', 'RECEIVE', 'ENTANGLE',
            'HINT.CPU', 'HINT.GPU', 'HINT.NPU', 'HINT.ANY', 'BATCH', 'UNBATCH', 'PREFETCH', 'FENCE', 'SYSCALL'
        ];

        for (const op of opcodes) {
            const item = new vscode.CompletionItem(op, vscode.CompletionItemKind.Keyword);
            item.detail = 'SIL opcode';
            items.push(item);
        }

        // Registers
        for (let i = 0; i <= 15; i++) {
            const name = i < 10 ? `R${i}` : `R${(i - 10 + 10).toString(16).toUpperCase()}`;
            const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Variable);
            item.detail = `Register ${i}`;
            items.push(item);
        }

        // Directives
        const directives = ['.mode', '.code', '.text', '.data', '.global', '.extern', '.byte', '.state'];
        for (const dir of directives) {
            const item = new vscode.CompletionItem(dir, vscode.CompletionItemKind.Module);
            item.detail = 'SIL directive';
            items.push(item);
        }

        return items;
    }
}

// LIS Formatting Provider
class LisFormattingProvider implements vscode.DocumentFormattingEditProvider {
    async provideDocumentFormattingEdits(
        document: vscode.TextDocument,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken
    ): Promise<vscode.TextEdit[]> {
        const config = vscode.workspace.getConfiguration('lis');
        const formatterPath = config.get<string>('formatter.path', 'lis-format');
        const indentSize = options.insertSpaces ? options.tabSize : 4;
        const indentStyle = options.insertSpaces ? 'spaces' : 'tabs';

        return new Promise((resolve) => {
            const { spawn } = require('child_process');

            const args = [
                '--indent', indentStyle,
                '--indent-size', indentSize.toString(),
                '-' // Read from stdin
            ];

            const formatter = spawn(formatterPath, args);
            let stdout = '';
            let stderr = '';

            formatter.stdout.on('data', (data: Buffer) => {
                stdout += data.toString();
            });

            formatter.stderr.on('data', (data: Buffer) => {
                stderr += data.toString();
            });

            formatter.on('close', (code: number) => {
                if (code !== 0) {
                    vscode.window.showErrorMessage(`LIS Formatter exited with code ${code}: ${stderr}`);
                    resolve([]);
                    return;
                }

                const fullRange = new vscode.Range(
                    document.positionAt(0),
                    document.positionAt(document.getText().length)
                );

                resolve([vscode.TextEdit.replace(fullRange, stdout)]);
            });

            formatter.on('error', (error: Error) => {
                vscode.window.showErrorMessage(`LIS Formatter failed: ${error.message}`);
                resolve([]);
            });

            // Write document content to stdin
            formatter.stdin.write(document.getText());
            formatter.stdin.end();
        });
    }
}

// LIS Range Formatting Provider
class LisRangeFormattingProvider implements vscode.DocumentRangeFormattingEditProvider {
    async provideDocumentRangeFormattingEdits(
        document: vscode.TextDocument,
        range: vscode.Range,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken
    ): Promise<vscode.TextEdit[]> {
        // For now, format the entire document
        // Range formatting would require more sophisticated parsing
        const provider = new LisFormattingProvider();
        return provider.provideDocumentFormattingEdits(document, options, token);
    }
}

// LIS Completion Provider
class LisCompletionProvider implements vscode.CompletionItemProvider {
    provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position
    ): vscode.CompletionItem[] {
        const items: vscode.CompletionItem[] = [];

        // Keywords
        const keywords = [
            'fn', 'transform', 'type', 'let', 'return',
            'if', 'else', 'loop', 'break', 'continue',
            'feedback', 'emerge', 'const', 'mut', 'pub', 'priv'
        ];
        for (const kw of keywords) {
            const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
            item.detail = 'LIS keyword';
            items.push(item);
        }

        // Types
        const types = [
            'ByteSil', 'State', 'Layer',
            'Int', 'Float', 'Bool', 'String', 'Void'
        ];
        for (const t of types) {
            const item = new vscode.CompletionItem(t, vscode.CompletionItemKind.Class);
            item.detail = 'LIS type';
            items.push(item);
        }

        // Layer constants (L0-LF)
        for (let i = 0; i <= 15; i++) {
            const name = `L${i.toString(16).toUpperCase()}`;
            const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Constant);
            item.detail = `Layer ${i}`;
            items.push(item);
        }

        // Builtin functions
        const functions = [
            'sense', 'act', 'process', 'normalize',
            'detect_patterns', 'emerge', 'transform',
            'lerp', 'slerp'
        ];
        for (const f of functions) {
            const item = new vscode.CompletionItem(f, vscode.CompletionItemKind.Function);
            item.detail = 'LIS builtin function';
            items.push(item);
        }

        // Annotations
        const annotations = ['@cpu', '@gpu', '@npu', '@simd', '@photonic', '@quantum'];
        for (const ann of annotations) {
            const item = new vscode.CompletionItem(ann, vscode.CompletionItemKind.Property);
            item.detail = 'Hardware hint';
            items.push(item);
        }

        // Operators
        const operators = ['|>', '**', '&&', '||', '==', '!=', '<=', '>='];
        for (const op of operators) {
            const item = new vscode.CompletionItem(op, vscode.CompletionItemKind.Operator);
            item.detail = 'LIS operator';
            items.push(item);
        }

        return items;
    }
}
