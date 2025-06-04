//@ts-check

import * as utils from "./utils.js"

async function main() {

    if (false == await utils.auth()) {
        console.log("not authorized")
        window.location.href = "/login.html"
        return
    }

    window.location.href = "/api/search" + window.location.search
}

await main()