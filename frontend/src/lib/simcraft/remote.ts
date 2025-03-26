import type { SimcraftAdapter, Process, Connection, SimulationState } from './base';

export class RemoteAdapter implements SimcraftAdapter {
    private ws: WebSocket | null = null;
    private messageQueue: ((value: SimulationState[]) => void)[] = [];
    private errorHandlers: ((error: Error) => void)[] = [];

    constructor(private url: string) {
        this.setupWebSocket();
    }

    private setupWebSocket() {
        this.ws = new WebSocket(this.url);
        
        this.ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (this.messageQueue.length > 0) {
                const resolver = this.messageQueue.shift();
                resolver?.(data);
            }
        };

        this.ws.onerror = (event) => {
            const error = new Error('WebSocket error occurred');
            if (this.errorHandlers.length > 0) {
                const handler = this.errorHandlers.shift();
                handler?.(error);
            }
        };
    }

    private async sendMessage(type: string, payload: any): Promise<SimulationState[]> {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('WebSocket not connected');
        }

        return new Promise((resolve, reject) => {
            this.messageQueue.push(resolve);
            this.errorHandlers.push(reject);
            this.ws?.send(JSON.stringify({ type, payload }));
        });
    }

    async initialise(processes: Process[], connections: Connection[]): Promise<void> {
        await this.sendMessage('init', { processes, connections });
    }

    async step(): Promise<SimulationState[]> {
        return this.sendMessage('step', {});
    }

    async step_until(until: number): Promise<SimulationState[]> {
        return this.sendMessage('step_until', { until });
    }

    async step_n(n: number): Promise<SimulationState[]> {
        return this.sendMessage('step_n', { n });
    }

    async destroy(): Promise<void> {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('WebSocket not connected');
        }
    }
}