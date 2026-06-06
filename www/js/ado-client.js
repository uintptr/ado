//@ts-check

/**
 * Toggle verbose client logging. Set `window.ADO_DEBUG = false` in the console
 * (or before load) to silence it.
 */
const DEBUG = () =>
    typeof window === "undefined" || window["ADO_DEBUG"] !== false;

/**
 * @param {string} msg
 * @param {...any} args
 */
function log(msg, ...args) {
    if (DEBUG()) console.log("%c[ado]", "color:#0c9", msg, ...args);
}

/**
 * @param {string} msg
 * @param {...any} args
 */
function warn(msg, ...args) {
    console.warn("[ado]", msg, ...args);
}

/**
 * ttyd wire protocol: byte 0 of every frame is an ASCII command char, the rest
 * is the payload. (Confirmed against ttyd 1.7.x.)
 */
const Command = {
    // client -> server
    INPUT: "0".charCodeAt(0), // 0x30
    RESIZE: "1".charCodeAt(0),
    // server -> client
    OUTPUT: "0".charCodeAt(0), // 0x30
    SET_WINDOW_TITLE: "1".charCodeAt(0),
    SET_PREFERENCES: "2".charCodeAt(0),
};

/**
 * WebSocket client for the ado headless backend via ttyd.
 *
 * ttyd handshake: on open the client MUST send an init JSON message
 * (`{AuthToken, columns, rows}`) — ttyd spawns/feeds the child only after it.
 * Input is then sent as `[Command.INPUT, ...utf8]`, and stdout arrives as
 * `[Command.OUTPUT, ...utf8]`.
 *
 * ado headless protocol:
 *   - input:  one line of text per command (newline-terminated)
 *   - output: newline-delimited JSON (NDJSON) — one compact JSON object per
 *             line, each tagged with a `type` field:
 *               { "type": "data",     "data": { ...AdoData... } }
 *               { "type": "markdown", "text": "..." }
 *               { "type": "error",    "message": "..." }
 *             Lines that aren't valid JSON (e.g. pty echo) are ignored.
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
    }

    /**
     * Resolve the ttyd WebSocket URL. Priority:
     *   1. window.ADO_WS_URL  (injected by the dev server)
     *   2. ?ado_ws=...        (query param override)
     *   3. same-origin /ado/ws (production, behind nginx)
     * @returns {string}
     */
    _resolveWsUrl() {
        if (window["ADO_WS_URL"]) return window["ADO_WS_URL"];
        const param = new URLSearchParams(window.location.search).get("ado_ws");
        if (param) return param;
        const protocol =
            window.location.protocol === "https:" ? "wss:" : "ws:";
        return `${protocol}//${window.location.host}/ado/ws`;
    }

    /**
     * Open a WebSocket connection to the ttyd backend.
     * @returns {Promise<void>}
     */
    connect() {
        return new Promise((resolve, reject) => {
            const wsUrl = this._resolveWsUrl();

            log("connecting to", wsUrl);
            // ttyd only treats the connection as a terminal (and spawns the
            // child process) if the client negotiates the "tty" subprotocol.
            this.ws = new WebSocket(wsUrl, ["tty"]);
            this.ws.binaryType = "arraybuffer";

            this.ws.onopen = () => {
                log("connection open — sending ttyd init handshake");
                // ttyd won't feed the child process until it gets this.
                const init = JSON.stringify({
                    AuthToken: "",
                    columns: 80,
                    rows: 24,
                });
                this.ws.send(new TextEncoder().encode(init));
                resolve();
            };

            this.ws.onerror = (e) => {
                warn("websocket error", e);
                reject(e);
            };

            this.ws.onclose = (e) => {
                warn(
                    `connection closed (code=${e.code} reason="${e.reason}" clean=${e.wasClean}). ` +
                        "If this happened right after connecting, the ado backend process likely exited.",
                );
            };

            this.ws.onmessage = (event) => {
                const data = new Uint8Array(event.data);
                const cmd = data[0];
                if (cmd === Command.OUTPUT) {
                    const text = new TextDecoder().decode(data.slice(1));
                    log(`recv ${text.length} chars of stdout`);
                    this._handleOutput(text);
                } else if (cmd === Command.SET_WINDOW_TITLE) {
                    log("recv window title", new TextDecoder().decode(data.slice(1)));
                } else if (cmd === Command.SET_PREFERENCES) {
                    log("recv preferences");
                } else {
                    log(`recv unknown ttyd frame, first byte=${cmd}`);
                }
            };
        });
    }

    /**
     * Accumulate stdout and dispatch one message per complete line.
     * @param {string} text
     */
    _handleOutput(text) {
        this.buffer += text;

        let idx;
        while ((idx = this.buffer.indexOf("\n")) >= 0) {
            const line = this.buffer.slice(0, idx).trim();
            this.buffer = this.buffer.slice(idx + 1);
            if (!line) continue;

            let msg;
            try {
                msg = JSON.parse(line);
            } catch {
                // Not a protocol message (e.g. pty echo or stray output) — skip.
                log("skip non-JSON line:", JSON.stringify(line));
                continue;
            }
            log("message:", msg);
            this._deliverResponse(msg);
        }
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
        if (!this.ws) {
            warn("cannot send: not connected", command);
            return;
        }

        const states = ["CONNECTING", "OPEN", "CLOSING", "CLOSED"];
        if (this.ws.readyState !== WebSocket.OPEN) {
            warn(
                `cannot send: socket is ${states[this.ws.readyState]}, dropping`,
                command,
            );
            return;
        }

        log("send:", command);
        const encoded = new TextEncoder().encode(command + "\n");
        const payload = new Uint8Array(1 + encoded.length);
        payload[0] = Command.INPUT; // ttyd input command byte ('0')
        payload.set(encoded, 1);
        this.ws.send(payload);
    }
}
