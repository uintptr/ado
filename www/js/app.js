//@ts-check

import * as utils from "./utils.js";
import { AdoClient } from "./ado-client.js";
import { marked } from "https://cdn.jsdelivr.net/npm/marked@12.0.2/lib/marked.esm.js";
import hljs from "https://cdn.jsdelivr.net/npm/@highlightjs/cdn-assets@11.9.0/es/highlight.min.js";

export {};

/** @type {HTMLElement|null} */
let thinking_el = null;

/**
 * Passthru mode: when set, the next search result is consumed by feeding its
 * first entry's link to this function and redirecting to the returned URL,
 * instead of rendering result cards. Returning null falls back to rendering.
 * @type {((link: string) => string | null) | null}
 */
let pending_dest = null;

/**
 * Command passthru: when set, the next plaintext response is consumed by passing
 * its raw text to this function and redirecting to the returned URL, instead of
 * rendering it. Returning null falls back to rendering. Used by command bangs
 * (e.g. `/reddit`) that reply with a single value rather than search results.
 * @type {((text: string) => string | null) | null}
 */
let pending_text = null;

function show_thinking() {
    const container = document.getElementById("results");
    if (container instanceof HTMLElement) {
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
 * Update the version tag from the backend (sourced from Cargo.toml).
 * @param {string} version
 */
function set_version(version) {
    const el = document.getElementById("version-tag");
    if (el instanceof HTMLElement) {
        el.textContent = "v" + version;
    }
}

/**
 * Build a card for one web-search result (WebResultEntry).
 * @param {{title:string, link:string, link_display:string, snippet:string}} entry
 * @returns {HTMLElement | null}
 */
function search_new_card(entry) {
    const card = utils.new_template("search_result");
    if (card == null) return null;

    const title = card.querySelector("#title_link");
    if (title instanceof HTMLAnchorElement) {
        // Google returns plain (non-HTML) title/snippet — use textContent.
        title.textContent = entry.title || entry.link;
        title.setAttribute("href", entry.link);
    }

    const parts = card.querySelector("#url_parts");
    if (parts instanceof HTMLElement) {
        parts.textContent = entry.link_display || entry.link;
    }

    const body = card.querySelector("#result_text");
    if (body instanceof HTMLElement) {
        body.textContent = entry.snippet || "";
    }

    return card;
}

/**
 * Render a WebResult (`/search` output) as a list of result cards.
 * @param {{entries: any[]}} result
 */
function display_search_results(result) {
    const container = document.getElementById("results");
    if (!(container instanceof HTMLElement)) return;

    const entries = (result && result.entries) || [];
    if (entries.length === 0) {
        display_string("`No results.`");
        return;
    }

    for (const entry of entries) {
        const card = search_new_card(entry);
        if (card != null) container.appendChild(card);
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
            if (markdown) {
                text_container.innerHTML = marked.parse(response);
                result.querySelectorAll("pre code").forEach((block) => {
                    hljs.highlightElement(block);
                });
                add_copy_buttons(result);
            } else {
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
        case "version":
            set_version(msg.version);
            return;
        case "data":
            display_ado_data(msg.data);
            return;
        case "markdown":
            display_string(msg.text);
            return;
        case "plaintext": {
            // Command bang passthru (e.g. `/reddit`): map the text to a URL.
            if (pending_text) {
                const dest = pending_text;
                pending_text = null;
                const url = dest(msg.text);
                if (url) {
                    console.log("[ado] passthru redirect:", url);
                    window.location.href = url;
                    return;
                }
                // No usable URL — fall back to rendering the text.
            }

            // `/search` emits its WebResult as a JSON blob — render as cards;
            // anything else printed verbatim falls back to text.
            let parsed = null;
            try {
                parsed = JSON.parse(msg.text);
            } catch {
                /* not JSON */
            }
            if (parsed && Array.isArray(parsed.entries)) {
                if (pending_dest) {
                    const dest = pending_dest;
                    pending_dest = null;
                    const first = parsed.entries[0];
                    const url =
                        first && first.link ? dest(first.link) : null;
                    if (url) {
                        console.log("[ado] passthru redirect:", url);
                        window.location.href = url;
                        return;
                    }
                    // No usable result — fall back to rendering cards.
                }
                display_search_results(parsed);
            } else {
                display_string(msg.text);
            }
            return;
        }
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

/**
 * Static "bang" shortcuts: a leading "<bang> " in the query redirects straight
 * to the provider's search URL (`%s` = encoded terms), no ado search involved.
 * @type {Record<string, string>}
 */
const SEARCH_BANGS = {
    a: "https://www.amazon.ca/s?k=%s",
    g: "https://www.google.ca/search?q=%s",
    i: "https://www.google.com/search?tbm=isch&q=%s",
};

/**
 * Command bangs: run an ado command, then map its single-line plaintext response
 * to a redirect URL. `command(terms)` builds the command line; `dest(text)` maps
 * the command's output to the final URL (return null to fall back to rendering).
 * @type {Record<string, {command: (t: string) => string, dest: (text: string) => string | null}>}
 */
const COMMAND_BANGS = {
    // "r dev humor" → `/reddit dev humor` → backend returns "/r/ProgrammerHumor"
    // → send the user to the matching old.reddit.com page.
    r: {
        command: (t) => `/reddit ${t}`,
        dest: (text) => {
            // Backend replies with e.g. "/r/ProgrammerHumor"; tolerate a
            // missing leading slash or "r/" prefix too.
            const sub = text
                .trim()
                .replace(/^\/+/, "")
                .replace(/^r\//i, "");
            if (!sub) return null;
            return `https://old.reddit.com/r/${sub}`;
        },
    },
};

/**
 * Send a `/search` to ado. When `dest` is non-null we're in passthru mode (see
 * {@link pending_dest}); when null the results render as cards.
 * @param {AdoClient} client
 * @param {string} terms
 * @param {((link: string) => string | null) | null} dest
 */
function run_search(client, terms, dest) {
    pending_dest = dest;
    console.log("[ado] search:", terms, dest ? "(passthru)" : "");
    if (!dest) {
        display_string(terms, false, null, "user-cmd");
    }
    try {
        client.send("/search " + terms);
    } catch (error) {
        console.error("[ado] search failed", error);
        pending_dest = null;
    }
}

/**
 * Send an ado command whose plaintext response should redirect the browser (see
 * {@link pending_text}).
 * @param {AdoClient} client
 * @param {string} command
 * @param {(text: string) => string | null} dest
 */
function run_command(client, command, dest) {
    pending_text = dest;
    console.log("[ado] command:", command, "(passthru)");
    try {
        client.send(command);
    } catch (error) {
        console.error("[ado] command failed", error);
        pending_text = null;
    }
}

/**
 * Handle a `/search?q=...` deep link.
 * Behaviour by leading bang:
 *   - `<bang> ` in {@link SEARCH_BANGS} → redirect to that external provider.
 *   - `<bang> ` in {@link COMMAND_BANGS} → run that command, redirect on its reply.
 *   - `s `                             → run ado's search, show results page.
 *   - (no recognised prefix)          → "I'm feeling lucky": redirect to the
 *                                        first ado result once it comes back.
 * @param {AdoClient} client
 * @param {string} search - the location.search string
 */
function search_handler(client, search) {
    const q = new URLSearchParams(search).get("q");
    if (q == null) return;

    const trimmed = q.trim();
    if (trimmed.length === 0) return;

    // Split off a leading "<bang> " token.
    const space = trimmed.indexOf(" ");
    const bang = space === -1 ? "" : trimmed.slice(0, space);
    const rest = space === -1 ? "" : trimmed.slice(space + 1).trim();

    if (rest.length > 0) {
        // Static external bang → redirect straight to the provider.
        if (Object.prototype.hasOwnProperty.call(SEARCH_BANGS, bang)) {
            const url = SEARCH_BANGS[bang].replace(
                "%s",
                encodeURIComponent(rest),
            );
            console.log("[ado] bang redirect:", bang, url);
            window.location.href = url;
            return;
        }

        // `s ` → normal results page.
        if (bang === "s") {
            run_search(client, rest, null);
            return;
        }

        // Command bang → run the ado command, redirect using its text reply.
        if (Object.prototype.hasOwnProperty.call(COMMAND_BANGS, bang)) {
            const { command, dest } = COMMAND_BANGS[bang];
            run_command(client, command(rest), dest);
            return;
        }
    }

    // Default: plain "I'm feeling lucky" on the whole query.
    run_search(client, trimmed, (link) => link);
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

    // Deep link: /search?q=<query> runs a web search and shows result cards.
    const params = window.location.search;
    if (params && params.length > 0) {
        search_handler(client, params);
    }
}

await main();
