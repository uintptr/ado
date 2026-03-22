//@ts-check

/**
 * WebSocket client for the ado headless backend via ttyd.
 *
 * ttyd protocol:
 *   - byte 0 of each frame is the message type (0 = stdin/stdout)
 *   - remaining bytes are the payload
 *
 * ado headless protocol:
 *   - input:  one line of text per command (newline-terminated)
 *   - output: either a JSON AdoData blob (pretty-printed, multi-line)
 *             or plain text (from /commands like /models)
 */
export class AdoClient {
    constructor() {
        /** @type {WebSocket|null} */
        this.ws = null;
        /** @type {string} */
        this.buffer = "";
        /** @type {((data: any) => void)|null} */
        this.pendingResolve = null;
        /** @type {((data: any) => void)|null} */
        this.onResponse = null;
        /** @type {(() => void)|null} */
        this.onThinkingStart = null;
        /** @type {(() => void)|null} */
        this.onThinkingStop = null;
        /** @type {number|null} */
        this.flushTimer = null;
    }

    /**
     * Open a WebSocket connection to the ttyd backend.
     * @returns {Promise<void>}
     */
    connect() {
        return new Promise((resolve, reject) => {
            const protocol =
                window.location.protocol === "https:" ? "wss:" : "ws:";
            const wsUrl = `${protocol}//${window.location.host}/ado/ws`;

            this.ws = new WebSocket(wsUrl);
            this.ws.binaryType = "arraybuffer";

            this.ws.onopen = () => resolve();
            this.ws.onerror = (e) => reject(e);

            this.ws.onmessage = (event) => {
                const data = new Uint8Array(event.data);
                if (data[0] === 0) {
                    const text = new TextDecoder().decode(data.slice(1));
                    this._handleOutput(text);
                }
            };
        });
    }

    /**
     * @param {string} text
     */
    _handleOutput(text) {
        this.buffer += text;

        if (this.flushTimer) {
            clearTimeout(this.flushTimer);
        }

        // Try to flush a complete JSON object immediately
        if (this._tryFlushJson()) {
            return;
        }

        // Otherwise debounce for plain-text responses
        this.flushTimer = setTimeout(() => this._flush(), 200);
    }

    /**
     * @returns {boolean}
     */
    _tryFlushJson() {
        const trimmed = this.buffer.trim();
        if (!trimmed.startsWith("{")) return false;

        try {
            const data = JSON.parse(trimmed);
            this.buffer = "";
            this._deliverResponse(data);
            return true;
        } catch {
            return false;
        }
    }

    _flush() {
        const text = this.buffer.trim();
        this.buffer = "";
        if (!text) return;

        try {
            const data = JSON.parse(text);
            this._deliverResponse(data);
            return;
        } catch {
            // not JSON — deliver as plain text
        }

        this._deliverResponse(text);
    }

    /**
     * @param {any} data
     */
    _deliverResponse(data) {
        if (this.onThinkingStop) this.onThinkingStop();

        if (this.pendingResolve) {
            const resolve = this.pendingResolve;
            this.pendingResolve = null;
            resolve(data);
        } else if (this.onResponse) {
            this.onResponse(data);
        }
    }

    /**
     * Send a command (fire-and-forget). The response arrives via onResponse.
     * @param {string} command
     */
    send(command) {
        if (this.onThinkingStart) this.onThinkingStart();
        this._sendRaw(command);
    }

    /**
     * Send a command and return a Promise that resolves with the response.
     * @param {string} command
     * @returns {Promise<any>}
     */
    query(command) {
        return new Promise((resolve) => {
            this.pendingResolve = resolve;
            if (this.onThinkingStart) this.onThinkingStart();
            this._sendRaw(command);
        });
    }

    /**
     * @param {string} command
     */
    _sendRaw(command) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            const encoded = new TextEncoder().encode(command + "\n");
            const payload = new Uint8Array(1 + encoded.length);
            payload[0] = 0; // ttyd stdin type
            payload.set(encoded, 1);
            this.ws.send(payload);
        }
    }
}
