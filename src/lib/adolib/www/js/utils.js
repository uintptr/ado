//@ts-check

/**
 * @param {HTMLElement} element
 */
export function hide_element(element) {
    element.style.display = "none";
}

/**
 * @param {HTMLElement} element
 */
export function show_element(element) {
    element.style.display = "block";
}

/**
 * @param {string} url
 * @returns {Promise<string | null>}
 */
export async function fetch_as_string(url) {
    try {
        let resp = await fetch(url);

        if (resp.status == 200) {
            return await resp.text();
        } else {
            console.log(url + " returned " + resp.status);
        }
    } catch (e) {
        console.log(e);
    }

    return null;
}

/**
 * @param {string} url
 * @returns {Promise<object | null>}
 */
export async function fetch_as_dict(url) {

    try {
        const response = await fetch(url);
        const data = await response.json();
        return data
    } catch (e) {
        console.log(e);
    }
    return null
}

/**
 * @param {HTMLElement} element
 */
export function remove_all_children(element) {
    while (element.firstChild) {
        element.removeChild(element.firstChild);
    }
}

/**
 * @param {string} id
 * @param {string} inner_tag
 * @returns {HTMLElement|null}
 */
export function new_template(id, inner_tag = "div") {
    const entry_template = document.getElementById(id);

    if (
        entry_template != null &&
        entry_template instanceof HTMLTemplateElement
    ) {
        let new_content = entry_template.content.cloneNode(true);

        if (new_content != null && new_content instanceof DocumentFragment) {
            let item = new_content.querySelector(inner_tag);

            if (item != null && item instanceof HTMLElement) {
                return item;
            }
        }
    }

    return null;
}

/**
 * @param {HTMLElement} component
 * @param {string} selector
 * @param {string} inner_html
 */
export function set_selector_text(component, selector, inner_html) {
    let element = component.querySelector(selector);

    if (element != null && element instanceof HTMLElement) {
        element.innerHTML = inner_html;
    }
}

export function isMobile() {
    const ua = navigator.userAgent;
    return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
        ua,
    );
}

/**
 * @param {Object} object
 * @returns {string}
 */
export function object_to_b64(object) {
    const object_string = JSON.stringify(object);
    const object_array = new TextEncoder().encode(object_string);
    const item_encoded = btoa(String.fromCharCode(...object_array));
    return item_encoded;
}

/**
 * @returns {Promise<String|null>}
 */
async function get_token_id() {
    //let server = await utils.fetch_as_json("/api/key/server")
    //console.log("server: " + server)
    let uri = "https://keys.pi/token";

    let token_id = await fetch_as_string(uri);

    return token_id;
}

/**
 * Checks if a cookie with the given name is already set
 * @param {string} cookieName - The name of the cookie to check
 * @returns {boolean} - True if the cookie is set, false otherwise
 */
function cookie_exists(cookieName) {
    return document.cookie.split(";").some((cookie) => {
        return cookie.trim().startsWith(cookieName + "=");
    });
}

/**
 * @returns {Promise<Boolean>}
 */
export async function auth() {
    if (true == cookie_exists("token_id")) {
        // already authorized
        return true;
    }

    const token_id_string = await get_token_id();

    if (null == token_id_string) {
        console.log("null token_id");
        return false;
    }

    if (36 != token_id_string.length) {
        console.log("length != 36");
        return false;
    }

    document.cookie = "token_id=" + token_id_string + "; Secure;";

    return true;
}
