import {
    DebugSession,
    InitializedEvent,
    StoppedEvent,
    OutputEvent,
    TerminatedEvent,
    Thread,
    StackFrame,
    Scope,
    Source,
    Variable,
    Handles
} from '@vscode/debugadapter';
import { DebugProtocol } from '@vscode/debugprotocol';
import * as path from 'path';
import { spawn, ChildProcess } from 'child_process';

interface SilLaunchRequestArguments extends DebugProtocol.LaunchRequestArguments {
    program: string;
    stopOnEntry?: boolean;
    mode?: string;
    trace?: boolean;
}

interface SilBreakpoint {
    id: number;
    line: number;
    verified: boolean;
}

interface SilStackFrame {
    index: number;
    name: string;
    file: string;
    line: number;
}

/**
 * Debug Adapter for SIL
 */
export class SilDebugSession extends DebugSession {
    private static THREAD_ID = 1;

    private variableHandles = new Handles<string>();
    private breakpoints = new Map<string, SilBreakpoint[]>();
    private breakpointId = 1;

    private vspProcess: ChildProcess | null = null;
    private currentLine = 0;
    private currentFile = '';
    private isPaused = false;

    // Register values cache
    private registers: number[] = new Array(16).fill(0);
    private flags = { zero: false, negative: false, carry: false, overflow: false };

    public constructor() {
        super();

        this.setDebuggerLinesStartAt1(true);
        this.setDebuggerColumnsStartAt1(true);
    }

    protected initializeRequest(
        response: DebugProtocol.InitializeResponse,
        args: DebugProtocol.InitializeRequestArguments
    ): void {
        response.body = response.body || {};

        // Capabilities
        response.body.supportsConfigurationDoneRequest = true;
        response.body.supportsEvaluateForHovers = true;
        response.body.supportsStepBack = false;
        response.body.supportsDataBreakpoints = false;
        response.body.supportsCompletionsRequest = false;
        response.body.supportsCancelRequest = false;
        response.body.supportsBreakpointLocationsRequest = true;
        response.body.supportsStepInTargetsRequest = false;
        response.body.supportsExceptionFilterOptions = false;
        response.body.supportsValueFormattingOptions = true;
        response.body.supportsRestartRequest = true;

        this.sendResponse(response);
        this.sendEvent(new InitializedEvent());
    }

    protected configurationDoneRequest(
        response: DebugProtocol.ConfigurationDoneResponse,
        args: DebugProtocol.ConfigurationDoneArguments
    ): void {
        super.configurationDoneRequest(response, args);
    }

    protected async launchRequest(
        response: DebugProtocol.LaunchResponse,
        args: SilLaunchRequestArguments
    ): Promise<void> {
        this.currentFile = args.program;
        this.currentLine = 0;

        // Start VSP in debug mode
        const vspArgs = ['debug', args.program];
        if (args.mode) {
            vspArgs.push('--mode', args.mode);
        }

        this.vspProcess = spawn('vsp', vspArgs, {
            stdio: ['pipe', 'pipe', 'pipe']
        });

        this.vspProcess.stdout?.on('data', (data) => {
            this.handleVspOutput(data.toString());
        });

        this.vspProcess.stderr?.on('data', (data) => {
            this.sendEvent(new OutputEvent(data.toString(), 'stderr'));
        });

        this.vspProcess.on('exit', (code) => {
            this.sendEvent(new TerminatedEvent());
        });

        // Set initial breakpoints
        for (const [file, bps] of this.breakpoints) {
            for (const bp of bps) {
                this.sendToVsp(`break ${bp.line}`);
            }
        }

        // Start or stop on entry
        if (args.stopOnEntry) {
            this.isPaused = true;
            this.sendEvent(new StoppedEvent('entry', SilDebugSession.THREAD_ID));
        } else {
            this.sendToVsp('continue');
        }

        this.sendResponse(response);
    }

    protected setBreakPointsRequest(
        response: DebugProtocol.SetBreakpointsResponse,
        args: DebugProtocol.SetBreakpointsArguments
    ): void {
        const path = args.source.path || '';
        const clientLines = args.lines || [];

        // Clear old breakpoints for this file
        this.breakpoints.delete(path);

        const silBreakpoints: SilBreakpoint[] = [];
        const responseBreakpoints: DebugProtocol.Breakpoint[] = [];

        for (const line of clientLines) {
            const bp: SilBreakpoint = {
                id: this.breakpointId++,
                line: line,
                verified: true
            };
            silBreakpoints.push(bp);

            const responseBp: DebugProtocol.Breakpoint = {
                id: bp.id,
                verified: bp.verified,
                line: bp.line,
                source: args.source
            };
            responseBreakpoints.push(responseBp);

            // Send to VSP
            if (this.vspProcess) {
                this.sendToVsp(`break ${line}`);
            }
        }

        this.breakpoints.set(path, silBreakpoints);
        response.body = { breakpoints: responseBreakpoints };
        this.sendResponse(response);
    }

    protected threadsRequest(response: DebugProtocol.ThreadsResponse): void {
        response.body = {
            threads: [
                new Thread(SilDebugSession.THREAD_ID, 'Main Thread')
            ]
        };
        this.sendResponse(response);
    }

    protected stackTraceRequest(
        response: DebugProtocol.StackTraceResponse,
        args: DebugProtocol.StackTraceArguments
    ): void {
        const frames: StackFrame[] = [];

        // Current frame
        frames.push(new StackFrame(
            0,
            `line ${this.currentLine}`,
            new Source(path.basename(this.currentFile), this.currentFile),
            this.currentLine
        ));

        response.body = {
            stackFrames: frames,
            totalFrames: frames.length
        };
        this.sendResponse(response);
    }

    protected scopesRequest(
        response: DebugProtocol.ScopesResponse,
        args: DebugProtocol.ScopesArguments
    ): void {
        response.body = {
            scopes: [
                new Scope('Registers', this.variableHandles.create('registers'), false),
                new Scope('Flags', this.variableHandles.create('flags'), false),
                new Scope('Memory', this.variableHandles.create('memory'), true)
            ]
        };
        this.sendResponse(response);
    }

    protected variablesRequest(
        response: DebugProtocol.VariablesResponse,
        args: DebugProtocol.VariablesArguments
    ): void {
        const variables: Variable[] = [];
        const id = this.variableHandles.get(args.variablesReference);

        if (id === 'registers') {
            // Show all 16 registers
            for (let i = 0; i < 16; i++) {
                const name = i < 10 ? `R${i}` : `R${(i - 10 + 10).toString(16).toUpperCase()}`;
                variables.push({
                    name: name,
                    value: `0x${this.registers[i].toString(16).padStart(32, '0')}`,
                    variablesReference: 0
                });
            }
        } else if (id === 'flags') {
            variables.push({ name: 'Zero', value: this.flags.zero.toString(), variablesReference: 0 });
            variables.push({ name: 'Negative', value: this.flags.negative.toString(), variablesReference: 0 });
            variables.push({ name: 'Carry', value: this.flags.carry.toString(), variablesReference: 0 });
            variables.push({ name: 'Overflow', value: this.flags.overflow.toString(), variablesReference: 0 });
        }

        response.body = { variables };
        this.sendResponse(response);
    }

    protected continueRequest(
        response: DebugProtocol.ContinueResponse,
        args: DebugProtocol.ContinueArguments
    ): void {
        this.isPaused = false;
        this.sendToVsp('continue');
        this.sendResponse(response);
    }

    protected nextRequest(
        response: DebugProtocol.NextResponse,
        args: DebugProtocol.NextArguments
    ): void {
        this.sendToVsp('step');
        this.sendResponse(response);
    }

    protected stepInRequest(
        response: DebugProtocol.StepInResponse,
        args: DebugProtocol.StepInArguments
    ): void {
        this.sendToVsp('step');
        this.sendResponse(response);
    }

    protected stepOutRequest(
        response: DebugProtocol.StepOutResponse,
        args: DebugProtocol.StepOutArguments
    ): void {
        this.sendToVsp('stepout');
        this.sendResponse(response);
    }

    protected pauseRequest(
        response: DebugProtocol.PauseResponse,
        args: DebugProtocol.PauseArguments
    ): void {
        this.sendToVsp('pause');
        this.sendResponse(response);
    }

    protected disconnectRequest(
        response: DebugProtocol.DisconnectResponse,
        args: DebugProtocol.DisconnectArguments
    ): void {
        if (this.vspProcess) {
            this.sendToVsp('quit');
            this.vspProcess.kill();
            this.vspProcess = null;
        }
        this.sendResponse(response);
    }

    protected evaluateRequest(
        response: DebugProtocol.EvaluateResponse,
        args: DebugProtocol.EvaluateArguments
    ): void {
        const expr = args.expression.toUpperCase();

        // Check if it's a register
        if (expr.match(/^R[0-9A-F]$/)) {
            const regNum = parseInt(expr[1], 16);
            response.body = {
                result: `0x${this.registers[regNum].toString(16).padStart(32, '0')}`,
                variablesReference: 0
            };
        } else {
            // Send to VSP for evaluation
            this.sendToVsp(`eval ${args.expression}`);
            response.body = {
                result: '<pending>',
                variablesReference: 0
            };
        }

        this.sendResponse(response);
    }

    protected restartRequest(
        response: DebugProtocol.RestartResponse,
        args: DebugProtocol.RestartArguments
    ): void {
        this.sendToVsp('restart');
        this.currentLine = 0;
        this.sendResponse(response);
    }

    private sendToVsp(command: string): void {
        if (this.vspProcess && this.vspProcess.stdin) {
            this.vspProcess.stdin.write(command + '\n');
        }
    }

    private handleVspOutput(output: string): void {
        const lines = output.trim().split('\n');

        for (const line of lines) {
            // Parse VSP debug output
            if (line.startsWith('STOPPED:')) {
                const match = line.match(/STOPPED:\s*(\d+)/);
                if (match) {
                    this.currentLine = parseInt(match[1]);
                    this.isPaused = true;
                    this.sendEvent(new StoppedEvent('breakpoint', SilDebugSession.THREAD_ID));
                }
            } else if (line.startsWith('REG:')) {
                // Parse register update: REG:0=0x...
                const match = line.match(/REG:(\d+)=0x([0-9a-fA-F]+)/);
                if (match) {
                    const regNum = parseInt(match[1]);
                    this.registers[regNum] = parseInt(match[2], 16);
                }
            } else if (line.startsWith('FLAGS:')) {
                // Parse flags: FLAGS:ZNCO
                const flagStr = line.substring(6);
                this.flags.zero = flagStr.includes('Z');
                this.flags.negative = flagStr.includes('N');
                this.flags.carry = flagStr.includes('C');
                this.flags.overflow = flagStr.includes('O');
            } else if (line.startsWith('OUTPUT:')) {
                this.sendEvent(new OutputEvent(line.substring(7) + '\n', 'stdout'));
            } else if (line.startsWith('ERROR:')) {
                this.sendEvent(new OutputEvent(line.substring(6) + '\n', 'stderr'));
            } else if (line === 'TERMINATED') {
                this.sendEvent(new TerminatedEvent());
            }
        }
    }
}

// Start the debug session
SilDebugSession.run(SilDebugSession);
