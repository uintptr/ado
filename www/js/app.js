//@ts-check

import * as utils from "./utils.js";
import { AdoClient } from "./ado-client.js";
import {
    navigateWithLoading,
    configureLoadingScreen,
} from "./loading-screen.js";

const marked = window["marked"];

configureLoadingScreen({
    enableDelay: false,
    minAnimationTime: 200,
});

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
 * @param {any} item
 * @param {string} name
 * @returns {HTMLElement | null}
 */
function search_new_card(item, name) {
    const result = utils.new_template(name);

    if (result != null) {
        const title = result.querySelector("#title_link");

        let link = item.link;

        if (title != null && title instanceof HTMLElement) {
            title.innerHTML = item.title;

            if (link.includes("www.reddit.com")) {
                link = link.replace("/www/", "old");
            }

            title.setAttribute("href", link);
        }

        let url = new URL(link);

        let components = url.pathname.split("/");
        components[0] = url.hostname;

        for (let i = 0; i < components.length; i++) {
            components[i] = decodeURIComponent(components[i]);
        }

        let parts = components.join(" > ");
        utils.set_selector_text(result, "#url_parts", parts);

        const body = result.querySelector("#result_text");

        if (body != null && body instanceof HTMLElement) {
            body.innerHTML = item.snippet;
        }
    }

    return result;
}

/**
 * @param {string} json_data
 */
async function display_search_results(json_data) {
    const container = document.getElementById("results");

    if (container != null && container instanceof HTMLElement) {
        let results = JSON.parse(json_data);

        results.items.forEach((item) => {
            let card = search_new_card(item, "search_result");

            if (card != null && card instanceof HTMLElement) {
                container.appendChild(card);
                const item_b64 = utils.object_to_b64(item);
                card.setAttribute("chat-data-b64", item_b64);
            }
        });

        utils.show_element(container);
    }
}

/**
 * @param {string} response
 * @param {boolean} markdown
 * @param {string | null} chat_source
 */
function display_string(response, markdown = true, chat_source = null) {
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

        const text_container = result.querySelector("#command_text");

        if (text_container != null && text_container instanceof HTMLElement) {
            if (true == markdown) {
                text_container.innerHTML = marked.parse(response);
                // @ts-ignore
                if (window.hljs) {
                    result.querySelectorAll("pre code").forEach((block) => {
                        // @ts-ignore
                        window.hljs.highlightElement(block);
                    });
                }
            } else {
                text_container.innerText = response;
            }

            container.appendChild(result);
        }
    }
}

function display_reset() {
    const results = document.getElementById("results");

    if (results != null && results instanceof HTMLElement) {
        utils.remove_all_children(results);
        utils.hide_element(results);
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
 * Handle a response from the ado backend.
 * @param {any} data - Either an AdoData JSON object or a plain text string.
 */
function display_response(data) {
    if (typeof data === "string") {
        // Plain text from print_markdown (e.g. /models output)
        display_string(data);
        return;
    }

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
 * @param {AdoClient} client
 */
function init_cmd_line(client) {
    const cmd_input = document.getElementById("cmd_line");

    if (cmd_input != null && cmd_input instanceof HTMLInputElement) {
        document.addEventListener("keydown", function (event) {
            if (event.ctrlKey) {
                return;
            }

            cmd_input.focus();
        });

        cmd_input.addEventListener("keyup", async function (e) {
            if (e.key == "Enter") {
                if (true == utils.isMobile()) {
                    cmd_input.blur();
                }

                const cmd_line = cmd_input.value;
                cmd_input.value = "";

                if (cmd_line.length > 0) {
                    display_string(cmd_line, false);

                    try {
                        client.send(cmd_line);
                    } catch (error) {
                        let err_msg = "error: " + error;
                        display_string("`" + err_msg + "`");
                    }
                }
            } else if (e.key == "Escape") {
                cmd_input.value = "";
            }
        });
    }
}

/**
 * Simple client-side question detection.
 * @param {string} text
 * @returns {boolean}
 */
function is_question(text) {
    const q = text.trim().toLowerCase();
    if (q.endsWith("?")) return true;

    const words = [
        "who",
        "what",
        "when",
        "where",
        "why",
        "how",
        "is",
        "are",
        "can",
        "could",
        "would",
        "should",
        "do",
        "does",
        "did",
        "will",
    ];
    return words.some((w) => q.startsWith(w + " "));
}

/**
 * @param {string} query
 */
async function navigate_to_lucky(query) {
    const url =
        "https://www.google.com/search?btnI&q=" + encodeURIComponent(query);
    await navigateWithLoading(url);
}

/**
 * @param {AdoClient} client
 * @param {string} search
 */
async function search_handler(client, search) {
    const urlParams = new URLSearchParams(search);

    const q = urlParams.get("q");

    if (q != null) {
        let q_plus_two = q.slice(2);

        if (q.startsWith("s ")) {
            let google_url =
                "https://google.com/search?q=" + encodeURIComponent(q_plus_two);
            await navigateWithLoading(google_url);
        } else if (q.startsWith("i ")) {
            let google_image_url =
                "https://www.google.com/search?q=" +
                encodeURIComponent(q_plus_two) +
                "&tbm=isch";
            await navigateWithLoading(google_image_url);
        } else if (q.startsWith("a ")) {
            let amazon_url =
                "https://www.amazon.ca/s?k=" + encodeURIComponent(q_plus_two);
            await navigateWithLoading(amazon_url);
        } else if (q.startsWith("c ")) {
            client.send(q_plus_two);
        } else if (q.startsWith("g ")) {
            let google_url =
                "https://google.com/search?q=" + encodeURIComponent(q_plus_two);
            await navigateWithLoading(google_url);
        } else if (q.startsWith("l ")) {
            await navigate_to_lucky(q_plus_two);
        } else if (q.startsWith("r ")) {
            let reddit_url =
                "https://old.reddit.com/search?q=" +
                encodeURIComponent(q_plus_two);
            await navigateWithLoading(reddit_url);
        } else if (q.startsWith("t ")) {
            let yfi_url =
                "https://finance.yahoo.com/quote/" +
                encodeURIComponent(q_plus_two) +
                "/";
            await navigateWithLoading(yfi_url);
        } else if (q.startsWith("w ")) {
            await navigate_to_lucky("wikipedia " + q_plus_two);
        } else {
            if (is_question(q)) {
                client.send(q);
            } else {
                await navigate_to_lucky(q);
            }
        }
    }
}

async function main() {
    const client = new AdoClient();

    client.onResponse = (data) => display_response(data);
    client.onThinkingStart = () => set_ready_status("THINKING...");
    client.onThinkingStop = () => set_ready_status("READY");

    try {
        await client.connect();
    } catch (e) {
        console.error("Failed to connect to ado backend", e);
        set_ready_status("OFFLINE");
        return;
    }

    set_ready_status("READY");

    init_cmd_line(client);

    const search = window.location.search;

    if (search != null && search.length > 0) {
        search_handler(client, search);
    }
}

await main();
