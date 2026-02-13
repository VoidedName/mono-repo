export function load_file_js(src) {
    return new Promise((resolve, reject) => {
        let request = new XMLHttpRequest();
        request.open('GET', src, true);
        request.responseType = 'arraybuffer';
        request.onload = function () {
            if (request.status !== 200) {
                reject("Failed with status: " + request.status);
            } else {
                resolve(request.response);
            }
        };
        request.onerror = function () {
            reject("Failed with status " + request.status);
        };
        request.send();
    })
}

export function loaded() {
    function rescale_canvas_by_pixel_ratio(width, height, name, ratio) {
        console.log("rescale_canvas_by_pixel_ratio", width, height, name, ratio)

        let canvas = document.getElementById(name)

        canvas.width = width * ratio
        canvas.height = height * ratio

        canvas.style.width = width + 'px'
        canvas.style.height = height + 'px'
    }

    function get_current_pixel_ratio() {
        return window.devicePixelRatio || window.screen.availWidth / document.documentElement.clientWidth;
    }

    rescale_canvas_by_pixel_ratio(
        document.documentElement.clientWidth,
        document.documentElement.clientHeight,
        "canvas",
        get_current_pixel_ratio(),
    )

    window.onresize = () => {
        rescale_canvas_by_pixel_ratio(
            document.documentElement.clientWidth,
            document.documentElement.clientHeight,
            "canvas",
            get_current_pixel_ratio(),
        )
    }
}


export function exit() {
    alert("Application terminated.");
    window.close();
}