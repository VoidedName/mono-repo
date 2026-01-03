import init from "../pkg/index.js";

const width = 800
const height = 600
const canvas_name = "canvas"

function rescale_canvas_by_pixel_ratio(width, height, name, ratio) {
    let canvas = document.getElementById(name)

    canvas.width = width * ratio
    canvas.height = height * ratio

    canvas.style.width = width + 'px'
    canvas.style.height = height + 'px'
}

function get_current_pixel_ratio() {
    return window.devicePixelRatio || window.screen.availWidth / document.documentElement.clientWidth;
}

window.onresize = () => {
    rescale_canvas_by_pixel_ratio(width, height, canvas_name, get_current_pixel_ratio())
}

rescale_canvas_by_pixel_ratio(width, height, canvas_name, get_current_pixel_ratio())

init("index_bg.wasm").catch(console.error);

