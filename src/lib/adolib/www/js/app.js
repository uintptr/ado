//@ts-check

import * as utils from "./utils.js"
import init, { AdoWasm,  } from "./pkg/adolib.js";

const marked = window["marked"]

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
        }
    });
}

export { }


/**
 * @param {any} item
 * @param {string} name
 * @returns {HTMLElement | null}
 */
function search_new_card(item, name) {

    const result = utils.new_template(name)

    if (result != null) {

        const title = result.querySelector("#title_link")

        let link = item.link

        if (title != null && title instanceof HTMLElement) {

            title.innerHTML = item.title

            if (link.includes("www.reddit.com")) {
                link = link.replace(/www/, "old");
            }

            title.setAttribute("href", link)
        }

        let url = new URL(link)

        // decomposed URL
        let components = url.pathname.split("/")
        components[0] = url.hostname

        for (let i = 0; i < components.length; i++) {
            components[i] = decodeURIComponent(components[i])
        }

        let parts = components.join(" > ")
        utils.set_selector_text(result, "#url_parts", parts)

        const body = result.querySelector("#result_text")

        if (body != null && body instanceof HTMLElement) {
            body.innerHTML = item.snippet
        }
    }

    return result
}




/**
 * @param {AdoWasm} wctx
 * @param {string} q
 */
async function search_issue_query(wctx, q) {

    const container = document.getElementById("results")

    if (container != null && container instanceof HTMLElement) {

        let results_str = await wctx.search(q)

        if (results_str != null) {

            let results = JSON.parse(results_str)

            results.items.forEach(item => {
                let card = search_new_card(item, "search_result")

                if (card != null && card instanceof HTMLElement) {
                    container.appendChild(card)
                    const item_b64 = utils.object_to_b64(item)
                    card.setAttribute("chat-data-b64", item_b64)
                }
            });

            utils.show_element(container)
        }
    }
}


/**
 * @param {HTMLElement} container
 * @param {string} cmdline
*/
async function command_reset(container, cmdline) {
    utils.remove_all_children(container)
    utils.hide_element(container)
}

/**
 * @param {string} response
 * @param {boolean} markdown
 * @param {string | null} chat_source
 */
function add_command_response(response, markdown = true, chat_source = null) {

    const container = document.getElementById("results")

    if (container == null || !(container instanceof HTMLElement)) {
        console.log("result not found", container)
        return
    }

    utils.show_element(container)

    const result = utils.new_template("command_result")

    if (null != result) {

        // to rebuild the context
        if (null != chat_source) {
            result.setAttribute("chat-source", chat_source)
            result.setAttribute("chat-data", response)
        }

        const text_container = result.querySelector("#command_text")

        if (text_container != null && text_container instanceof HTMLElement) {

            if (true == markdown) {
                text_container.innerHTML = marked.parse(response)
                // Apply syntax highlighting to code blocks
                if (window.hljs) {
                    result.querySelectorAll('pre code').forEach((block) => {
                        window.hljs.highlightElement(block);
                    });
                }
            }
            else {
                text_container.innerText = response
            }

            container.appendChild(result)
        }
        else {
            console.error("unable to find text container")
        }
    }
    else {
        console.error("couldn't find template")
    }
}


/**
 * @param {AdoWasm} wctx
 */
function init_cmd_line(wctx) {

    const cmd_input = document.getElementById('cmd_line');

    if (cmd_input != null && cmd_input instanceof HTMLInputElement) {

        document.addEventListener('keydown', function (event) {

            if (event.ctrlKey) {
                return
            }

            cmd_input.focus()
        });

        cmd_input.addEventListener("keyup", async function (e) {
            if (e.key == "Enter") {

                //
                // hide the keyboard
                //
                if (true == utils.isMobile()) {
                    cmd_input.blur()
                }

                const cmd_line = cmd_input.value
                cmd_input.value = ""

                if (cmd_line.length > 0) {
                    let ret = await wctx.query(cmd_line)

                    console.log(ret)

                    add_command_response(ret)
                }
            }
            else if (e.key == "Escape") {
                cmd_input.value = ""
            }
        })
    }
    else {
        console.error("couldn't find the search input")
    }
}

async function main() {

    // loading the wasm bit
    await init()

    // get the config.toml file

    let config = await utils.fetch_as_string("https://keys.pi/ado.toml")

    if (config != null) {

        let wctx = new AdoWasm(config); // global

        init_cmd_line(wctx)

        const search = window.location.search;

        // from the URL bar
        if (search != null && search.length > 0) {
            const urlParams = new URLSearchParams(search);

            const q = urlParams.get('q')

            if (q != null) {

                let q_plus_two = q.slice(2)

                if (q.startsWith("s ")) {
                    //
                    // assume this is a search
                    //
                    search_issue_query(wctx, q_plus_two)
                } else if (q.startsWith("a ")) {
                    let amazon_url = "https://www.amazon.ca/s?k=" + q_plus_two
                    window.location.href = amazon_url
                } else if (q.startsWith("c ")) {
                    //
                    // assume this is a chat request
                    //
                    let res = await wctx.query(q_plus_two)
                    add_command_response(res)
                } else if (q.startsWith("g ")) {
                    let google_url = "https://google.com/search?q=" + q_plus_two
                    window.location.href = google_url
                } else if (q.startsWith("l ")) {
                    let lucky_url = await wctx.lucky(q_plus_two)
                    window.location.href = lucky_url
                } else if (q.startsWith("r ")) {
                    let sub = await wctx.find_sub_reddit(q_plus_two)
                    let reddit_url = "https://old.reddit.com" + sub + "/"
                    console.log(reddit_url)
                    window.location.href = reddit_url
                } else if (q.startsWith("t ")) {
                    let yfi_url = "https://finance.yahoo.com/quote/" + q_plus_two + "/"
                    window.location.href = yfi_url
                } else {
                    //
                    // detect if this is a question
                    //
                    if (true == wctx.is_question(q)) {
                        let res = await wctx.query(q)
                        add_command_response(res)

                    } else {
                        //
                        // fallback is lucky url
                        //
                        let lucky_url = await wctx.lucky(q)
                        window.location.href = lucky_url
                    }
                }
            }
        }
    } else {
        console.log("not authorized")
        window.location.href = "/login.html"
    }
}

await main()