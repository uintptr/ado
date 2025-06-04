//@ts-check

import * as utils from "./utils.js"
import init, { AdoWasm } from "../pkg/adolib.js";

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
function new_search_card(item, name) {

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
 * @param {HTMLElement} container
 * @param {string} q
 */
async function issue_google_query(container, q) {


    let results = []; //await utils.fetch_as_json(url)

    if (results != null) {
        results.items.forEach(item => {
            let card = new_search_card(item, "search_result")

            if (card != null && card instanceof HTMLElement) {
                container.appendChild(card)
                const item_b64 = utils.object_to_b64(item)
                card.setAttribute("chat-data-b64", item_b64)
            }
        });

        utils.show_element(container)
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


async function main() {

    // loading the wasm bit
    await init()

    // get the config.toml file

    let config = await utils.fetch_as_string("http://10.0.0.2/ado.toml")

    if (config != null) {

        let wctx = new AdoWasm(config); // global

        console.log(wctx)

        let reddit = await wctx.find_sub_reddit("all")

        console.log(reddit)

        const search = window.location.search;

        if (false == await utils.auth()) {
            console.log("not authorized")
            window.location.href = "/login.html"
            return
        }

        // from the URL bar
        if (search != null && search.length > 0) {
            const urlParams = new URLSearchParams(search);
            console.log(urlParams)
        }
    }
}

await main()