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