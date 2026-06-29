//@ts-check

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

export function isMobile() {
    const ua = navigator.userAgent;
    return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
        ua,
    );
}
