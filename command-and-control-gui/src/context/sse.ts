import { GlobalLogs } from "@/context/globals/logs";
import {
    EventSourceMessage,
    fetchEventSource,
} from "@microsoft/fetch-event-source";

export enum SseStatus {
    not_connected,
    connected,
    closed,
}

export class SSE {
    private _abort_controller: AbortController;

    public constructor(private _host: string, private _bearer: string) {
        this._abort_controller = new AbortController();
        this._abort_controller.abort = this._abort_controller.abort.bind(this._abort_controller);

        // Bind the functions to the class to avoid issues with `this`
        this.onClose = this.onClose.bind(this);
        this.onError = this.onError.bind(this);
        this.onMessage = this.onMessage.bind(this);
        this.connect = this.connect.bind(this);
        this.abort = this.abort.bind(this);
    }

    private _status = SseStatus.not_connected;

    public get status() {
        return this._status;
    }

    public get host() {
        return this._host;
    }

    public get bearer() {
        return this._bearer;
    }

    public set bearer(bearer: string) {
        this._bearer = bearer;
    }

    public async connect() {
        if (this._status === SseStatus.connected) {
            throw new Error("SSE is already connected");
        }

        if (!this._abort_controller) {
            this._abort_controller = new AbortController();
            this._abort_controller.abort = this._abort_controller.abort.bind(this._abort_controller);
        }

        try {
            await fetchEventSource(`http://${ this._host }/sse`, {
                headers:   {
                    Authorization: `Bearer ${ this._bearer }`,
                },
                signal:    this._abort_controller.signal,
                onmessage: this.onMessage,
                onclose:   this.onClose,
                onerror:   this.onError,
            });

            this._status = SseStatus.connected;
        }
        catch (error) {
            this._status = SseStatus.closed;
            throw new Error(`Failed to connect to SSE: ${ error }`);
        }
    }

    public abort() {
        if (this._status !== SseStatus.connected) {
            throw new Error("SSE is not connected, cannot abort");
        }

        this._abort_controller.abort("SSE aborted");
        this._abort_controller = new AbortController();
        this._abort_controller.abort = this._abort_controller.abort.bind(this._abort_controller);
        this._status = SseStatus.closed;
    }

    private onMessage(event: EventSourceMessage) {
        switch (event.event) {
            case "log":
                GlobalLogs.data.push(JSON.parse(event.data));
                break;
            default:
                console.error(`Unknown event(${ event.event }):`, event);
                break;
        }
    }

    private onClose() {
        console.log("Connection closed");
        this._status = SseStatus.closed;
    }

    private onError(err: any) {
        this._status = SseStatus.closed;
        throw new Error(`SSE Error ${ err }`);
    }
}