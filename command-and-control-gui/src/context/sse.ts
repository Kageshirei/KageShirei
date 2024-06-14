import {EventSourceMessage, fetchEventSource,} from "@microsoft/fetch-event-source";

export enum SseStatus {
    not_connected,
    connected,
    error,
    closed,
}

export class SSE {
    private _abort_controller: AbortController;

    public constructor(private _host: string, private _bearer: string) {
        this._abort_controller = new AbortController();

        // Bind the functions to the class to avoid issues with `this`
        this.onClose = this.onClose.bind(this);
        this.onError = this.onError.bind(this);
        this.onMessage = this.onMessage.bind(this);
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
        await fetchEventSource(`http://${this._host}/sse`, {
            headers: {
                Authorization: `Bearer ${this._bearer}`,
            },
            signal: this._abort_controller.signal,
            onmessage: this.onMessage,
            onclose: this.onClose,
            onerror: this.onError,
        })
    }

    public abort() {
        if (this._status !== SseStatus.connected) {
            throw new Error("SSE is not connected, cannot abort");
        }

        this._abort_controller.abort();
    }

    private onMessage(event: EventSourceMessage) {
        console.log(event);
    }

    private onClose() {

    }

    private onError(err: any) {
        console.error(err);
    }
}