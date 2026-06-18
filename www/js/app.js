//@ts-check

import * as utils from "./utils.js";
import { AdoClient } from "./ado-client.js";

const marked = window["marked"];

// @ts-ignore
if (window.hljs) {
    marked.setOptions({
        highlight: function (code, lang) {
            // @ts-ignore
            if (lang && window.hljs.getLanguage(lang)) {
                // @ts-ignore
                return window.hljs.highlight(code, { language: lang }).value;
            }
            // @ts-ignore
            return window.hljs.highlightAuto(code).value;
        },
    });
}

export {};

/** @type {HTMLElement|null} */
let thinking_el = null;

function show_thinking() {
    const container = document.getElementById("results");
    if (container instanceof HTMLElement) {
        utils.show_element(container);
        thinking_el = document.createElement("div");
        thinking_el.className = "result-item thinking-card";
        thinking_el.innerHTML =
            '<div class="thinking-dots"><span></span><span></span><span></span></div>';
        container.appendChild(thinking_el);
        thinking_el.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
}

function hide_thinking() {
    if (thinking_el) {
        thinking_el.remove();
        thinking_el = null;
    }
}

function scroll_to_latest() {
    const container = document.getElementById("results");
    if (container instanceof HTMLElement && container.lastElementChild) {
        container.lastElementChild.scrollIntoView({
            behavior: "smooth",
            block: "nearest",
        });
    }
}

/**
 * @param {HTMLElement} container
 */
function add_copy_buttons(container) {
    container.querySelectorAll("pre").forEach((pre) => {
        if (pre.querySelector(".copy-btn")) return;
        const btn = document.createElement("button");
        btn.className = "copy-btn";
        btn.textContent = "copy";
        btn.addEventListener("click", () => {
            const code = pre.querySelector("code");
            const text = code ? code.innerText : pre.innerText;
            navigator.clipboard.writeText(text).then(() => {
                btn.textContent = "copied!";
                setTimeout(() => {
                    btn.textContent = "copy";
                }, 2000);
            });
        });
        pre.appendChild(btn);
    });
}

/**
 * @param {string} status
 */
function set_ready_status(status) {
    const ready_container = document.querySelector("#ready-status");

    if (null != ready_container && ready_container instanceof HTMLElement) {
        ready_container.textContent = status;
    }
}

/**
 * @param {string} response
 * @param {boolean} markdown
 * @param {string | null} chat_source
 * @param {string | null} extra_class
 */
function display_string(
    response,
    markdown = true,
    chat_source = null,
    extra_class = null,
) {
    const container = document.getElementById("results");

    if (container == null || !(container instanceof HTMLElement)) {
        console.warn("result not found", container);
        return;
    }

    utils.show_element(container);

    const result = utils.new_template("command_result");

    if (null != result) {
        if (null != chat_source) {
            result.setAttribute("chat-source", chat_source);
            result.setAttribute("chat-data", response);
        }

        if (extra_class) {
            result.classList.add(extra_class);
        }

        const text_container = result.querySelector("#command_text");

        if (text_container != null && text_container instanceof HTMLElement) {
            if (true == markdown && marked) {
                text_container.innerHTML = marked.parse(response);
                // @ts-ignore
                if (window.hljs) {
                    result.querySelectorAll("pre code").forEach((block) => {
                        // @ts-ignore
                        window.hljs.highlightElement(block);
                    });
                }
                add_copy_buttons(result);
            } else {
                // Plain text, or markdown lib unavailable (CDN blocked/offline).
                if (true == markdown && !marked) {
                    console.warn("[ado] marked.js not loaded; rendering as text");
                }
                text_container.innerText = response;
            }

            container.appendChild(result);
            scroll_to_latest();
        }
    }
}

/**
 * @param {object} artifact
 */
function display_artifact(artifact) {
    switch (artifact.type) {
        case "code": {
            const lang = artifact.language || "";
            display_string("```" + lang + "\n" + artifact.content + "\n```");
            break;
        }
        case "diff":
            display_string("```diff\n" + artifact.content + "\n```");
            break;
        case "command":
            display_string("`" + artifact.content + "`");
            break;
        default:
            display_string(artifact.content);
            break;
    }
}

/**
 * Render a structured AdoData response.
 * @param {any} data
 */
function display_ado_data(data) {
    if (data.error) {
        display_string("`Error: " + data.error.message + "`");
        return;
    }

    if (data.response) {
        if (data.response.message) {
            display_string(data.response.message);
        }

        if (data.response.artifacts) {
            for (const artifact of data.response.artifacts) {
                display_artifact(artifact);
            }
        }
    }
}

/**
 * Handle a message from the ado backend (NDJSON envelope, see ado-client.js).
 * @param {any} msg
 */
function display_response(msg) {
    if (typeof msg === "string") {
        display_string(msg);
        return;
    }

    switch (msg.type) {
        case "data":
            display_ado_data(msg.data);
            return;
        case "markdown":
            display_string(msg.text);
            return;
        case "action":
            // Agentic progress note (running a command / writing a file).
            display_string("`» " + msg.text + "`", true, null, "action-note");
            return;
        case "error":
            display_string("`Error: " + msg.message + "`");
            return;
        default:
            console.warn("unknown message type", msg);
    }
}

/**
 * @param {AdoClient} client
 */
function init_cmd_line(client) {
    const cmd_input = document.getElementById("cmd_line");

    if (cmd_input != null && cmd_input instanceof HTMLInputElement) {
        /** @type {string[]} */
        const history = [];
        let hist_idx = -1;
        let hist_draft = "";

        document.addEventListener("keydown", function (event) {
            if (event.ctrlKey) {
                return;
            }

            cmd_input.focus();
        });

        cmd_input.addEventListener("keydown", function (e) {
            if (e.key === "ArrowUp") {
                e.preventDefault();
                if (history.length === 0) return;
                if (hist_idx === -1) hist_draft = cmd_input.value;
                if (hist_idx < history.length - 1) hist_idx++;
                cmd_input.value = history[history.length - 1 - hist_idx];
                cmd_input.setSelectionRange(
                    cmd_input.value.length,
                    cmd_input.value.length,
                );
            } else if (e.key === "ArrowDown") {
                e.preventDefault();
                if (hist_idx <= 0) {
                    hist_idx = -1;
                    cmd_input.value = hist_draft;
                } else {
                    hist_idx--;
                    cmd_input.value = history[history.length - 1 - hist_idx];
                }
                cmd_input.setSelectionRange(
                    cmd_input.value.length,
                    cmd_input.value.length,
                );
            }
        });

        cmd_input.addEventListener("keyup", async function (e) {
            if (e.key == "Enter") {
                if (true == utils.isMobile()) {
                    cmd_input.blur();
                }

                const cmd_line = cmd_input.value;
                cmd_input.value = "";
                hist_idx = -1;
                hist_draft = "";

                if (cmd_line.length > 0) {
                    console.log("[ado] command entered:", cmd_line);
                    history.push(cmd_line);
                    display_string(cmd_line, false, null, "user-cmd");

                    try {
                        client.send(cmd_line);
                    } catch (error) {
                        console.error("[ado] send failed", error);
                        let err_msg = "error: " + error;
                        display_string("`" + err_msg + "`");
                    }
                }
            } else if (e.key == "Escape") {
                cmd_input.value = "";
                hist_idx = -1;
                hist_draft = "";
            }
        });
    }
}

async function main() {
    console.log("[ado] app starting");
    const client = new AdoClient();

    client.onResponse = (data) => {
        console.log("[ado] dispatching response", data);
        display_response(data);
    };
    client.onThinkingStart = () => {
        set_ready_status("THINKING...");
        show_thinking();
    };
    client.onThinkingStop = () => {
        set_ready_status("READY");
        hide_thinking();
    };

    try {
        await client.connect();
    } catch (e) {
        console.error("[ado] failed to connect to ado backend", e);
        set_ready_status("OFFLINE");
        return;
    }

    set_ready_status("READY");

    init_cmd_line(client);
    console.log("[ado] ready — command input wired");
}

await main();
