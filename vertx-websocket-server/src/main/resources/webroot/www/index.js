// If you only use `npm` you can simply
// import { Chart } from "wasm-demo" and remove `setup` call from `bootstrap.js`.
class Chart {}

const canvas = document.getElementById("canvas");
const status = document.getElementById("status");

let chart = null;

/** Main entry point */
export function main() {
    setupUI();
    setupCanvas();
}

/** This function is used in `bootstrap.js` to setup imports. */
export function setup(WasmChart) {
    Chart = WasmChart;
}

/** Add event listeners. */
function setupUI() {
    status.innerText = "WebAssembly loaded!";
    window.addEventListener("resize", setupCanvas);
}

/** Setup canvas to properly handle high DPI and redraw current plot. */
function setupCanvas() {
    const dpr = window.devicePixelRatio || 1;
    const aspectRatio = canvas.width / canvas.height;
    const size = Math.min(canvas.width, canvas.parentNode.offsetWidth);
    canvas.style.width = size + "px";
    canvas.style.height = size / aspectRatio + "px";
    canvas.width = size * dpr;
    canvas.height = size / aspectRatio * dpr;
    canvas.getContext("2d").scale(dpr, dpr);
    Chart.websocket_values("ws://127.0.0.1:8666");
}
