// import jsdom from "https://dev.jspm.io/jsdom";
// import jsdom from 'https://cdn.skypack.dev/jsdom';
import jsdom from 'https://esm.sh/jsdom'

const jsdomConfig = {
    url: "http://localhost:8080",
    runScripts: "dangerously",
    resources: "usable",
};
console.log(
    new jsdom.JSDOM("<h1>Hello, World!<h1>", jsdomConfig)
    .window.document.querySelector("h1").textContent
);

// deno run --allow-read index.js
// /* esm.sh - error */
// throw new Error("[esm.sh] " + "Unsupported nodejs builtin module \"tls\" (Imported by \"http-proxy-agent\")");
// export default null;
// https://deno.land/std@0.105.0/node/README.md

// https://github.com/capricorn86/happy-dom/tree/master/packages/server-rendering  - needs vm?
