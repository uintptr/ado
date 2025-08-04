//@ts-check

import * as utils from "./utils.js";
import init, { AdoWasm } from "./pkg/adolib.js";
import {
    navigateWithLoading,
    configureLoadingScreen,
} from "./loading-screen.js";

const marked = window["marked"];

class UserConfig {
    constructor(user_id, storage_url, config_file) {
        this.user_id = user_id
        this.storage_url = storage_url
        this.config_file = config_file
    }
}

// Configure loading screen with minimal delays
configureLoadingScreen({
    enableDelay: false, // Disable artificial delays
    minAnimationTime: 200, // Set minimal animation time
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

export { };

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
                link = link.replace(/www/, "old");
            }

            title.setAttribute("href", link);
        }

        let url = new URL(link);

        // decomposed URL
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
        console.log("result not found", container);
        return;
    }

    utils.show_element(container);

    const result = utils.new_template("command_result");

    if (null != result) {
        // to rebuild the context
        if (null != chat_source) {
            result.setAttribute("chat-source", chat_source);
            result.setAttribute("chat-data", response);
        }

        const text_container = result.querySelector("#command_text");

        if (text_container != null && text_container instanceof HTMLElement) {
            if (true == markdown) {
                text_container.innerHTML = marked.parse(response);
                // Apply syntax highlighting to code blocks
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
        } else {
            console.error("unable to find text container");
        }
    } else {
        console.error("couldn't find template");
    }
}

/**
 * @param {AdoWasm} wctx
 * @param {string} q
 */
async function search_issue_query(wctx, q) {
    const container = document.getElementById("results");

    if (container != null && container instanceof HTMLElement) {
        let results_str = await wctx.search(q);

        if (results_str != null) {
            display_search_results(results_str);
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
 * @param {object} response
 */
function response_handler(response) {
    if (response.hasOwnProperty("UsageString")) {
        let usage = "```\n" + response.UsageString + "\n```";
        display_string(usage);
    } else if (response.hasOwnProperty("String")) {
        display_string(response.String);
    } else if (response.hasOwnProperty("SearchData")) {
        let json_data = response.SearchData;
        display_search_results(json_data);
    } else if (response == "Reset") {
        display_reset();
    } else {
        console.warn(response);
    }
}

/**
 * @param {AdoWasm} wctx
 */
function init_cmd_line(wctx) {
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
                //
                // hide the keyboard
                //
                if (true == utils.isMobile()) {
                    cmd_input.blur();
                }

                const cmd_line = cmd_input.value;
                cmd_input.value = "";

                if (cmd_line.length > 0) {
                    display_string(cmd_line, false);

                    try {
                        let ret = await wctx.query(cmd_line);
                        response_handler(ret);
                    } catch (error) {
                        let err_msg = "error: " + error;
                        display_string("`" + err_msg + "`");
                    }
                }
            } else if (e.key == "Escape") {
                cmd_input.value = "";
            }
        });
    } else {
        console.error("couldn't find the search input");
    }
}

/**
 * @param {AdoWasm} wctx
 * @param {string} search
 */

async function search_handler(wctx, search) {
    const urlParams = new URLSearchParams(search);

    const q = urlParams.get("q");

    if (q != null) {
        let q_plus_two = q.slice(2);

        if (q.startsWith("s ")) {
            //
            // assume this is a search
            //
            search_issue_query(wctx, q_plus_two);
        } else if (q.startsWith("a ")) {
            //
            // amazon search
            //
            let amazon_url = "https://www.amazon.ca/s?k=" + q_plus_two;
            await navigateWithLoading(amazon_url);
        } else if (q.startsWith("c ")) {
            //
            // assume this is a chat request
            //
            let res = await wctx.query(q_plus_two);
            response_handler(res);
        } else if (q.startsWith("g ")) {
            //
            // google search
            //
            let google_url = "https://google.com/search?q=" + q_plus_two;
            await navigateWithLoading(google_url);
        } else if (q.startsWith("l ")) {
            //
            // I'm feeling lucky google search
            //
            let lucky_url = await wctx.lucky(q_plus_two);
            await navigateWithLoading(lucky_url);
        } else if (q.startsWith("r ")) {
            //
            // Find the associated subreddit
            //
            let sub = await wctx.find_sub_reddit(q_plus_two);
            let reddit_url = "https://old.reddit.com" + sub + "/";
            await navigateWithLoading(reddit_url);
        } else if (q.startsWith("t ")) {
            //
            // ticker
            //
            let yfi_url = "https://finance.yahoo.com/quote/" + q_plus_two + "/";
            await navigateWithLoading(yfi_url);
        } else if (q.startsWith("w ")) {
            //
            // wikipedia
            //
            let wikipedia_url = await wctx.lucky("wikipedia " + q_plus_two);
            await navigateWithLoading(wikipedia_url);
        } else {
            //
            // detect if this is a question
            //
            if (true == wctx.is_question(q)) {
                let res = await wctx.query(q);
                response_handler(res);
            } else {
                //
                // fallback to google I'm feeling lucky url. In most
                // cases this is better than a search result
                //
                let lucky_url = await wctx.lucky(q);

                if (lucky_url.includes("www.reddit.com")) {
                    lucky_url = lucky_url.replace(/www/, "old");
                }

                await navigateWithLoading(lucky_url);
            }
        }
    }
}


/**
 * @param {string} user_id
 * @param {string} config_server
 * @returns {Promise<string | null>}
 */

async function get_config_file(user_id, config_server) {

    let config_file = null
    const webdis_url = config_server + "/GET/" + user_id
    let config = await utils.fetch_as_dict(webdis_url);

    if (null != config) {
        config_file = config.GET
    }

    return config_file
}

/**
 * @returns {Promise<string|null>}
 */
async function get_user() {

    let config = localStorage.getItem("user_config")

    if (null == config) {
        config = await utils.fetch_as_string("https://keys.pi/user.json");
    }

    if (null != config) {
        localStorage.setItem("user_config", config)
    }

    return config
}

/**
 * @returns {Promise<UserConfig|null>}
 */
async function get_config() {

    let user_json = await get_user()

    if (null != user_json) {
        let user = JSON.parse(user_json)

        const config_file = await get_config_file(user.user_id, user.storage_url)

        if (config_file != null) {
            return new UserConfig(user.user_id, user.storage_url, config_file)
        }
    }

    return null
}

async function main() {
    // loading the wasm bits
    await init();

    // get the config.toml file
    let config = await get_config()

    if (config != null) {

        let wctx = new AdoWasm(config.user_id, config.storage_url, config.config_file)

        init_cmd_line(wctx);

        const search = window.location.search;

        // from the URL bar
        if (search != null && search.length > 0) {
            search_handler(wctx, search);
        }

    } else {
        console.log("not authorized");
        await navigateWithLoading("/login.html");
    }
}

await main();
