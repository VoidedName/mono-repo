use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generates the Web (Wasm) target crate and the associated frontend boilerplate.
pub fn create(root: &Path, name: &str) -> Result<()> {
    let crate_name = format!("{}-web", name);
    let path = root.join(&crate_name);
    fs::create_dir_all(path.join("src"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
{name}-core = {{ path = "../{name}-core" }}
wasm-bindgen = {{ workspace = true }}
web-sys = {{ workspace = true, features = ["Window", "Document", "Element", "HtmlElement"] }}
console_error_panic_hook = {{ workspace = true }}
console_log = {{ workspace = true }}
log = {{ workspace = true }}
wasm-bindgen-futures = {{ workspace = true }}
rfd = {{ workspace = true }}
"#
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)?;
    
    let crate_safe_name = name.replace('-', "_");
    let lib_rs = format!(
        r#"use wasm_bindgen::prelude::*;
use {crate_safe_name}_core::{{Counter, PlatformHooks}};
use std::sync::{{Arc, Mutex}};

struct WebHooks;
impl PlatformHooks for WebHooks {{}}

#[wasm_bindgen]
pub struct WebApp {{
    counter: Arc<Mutex<Counter>>,
}}

#[wasm_bindgen]
impl WebApp {{
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {{
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).unwrap();
        Self {{
            counter: Arc::new(Mutex::new(Counter::new())),
        }}
    }}

    pub fn increment(&self) {{
        self.counter.lock().unwrap().increment(&WebHooks);
    }}

    pub fn decrement(&self) {{
        self.counter.lock().unwrap().decrement(&WebHooks);
    }}

    pub fn reset(&self) {{
        self.counter.lock().unwrap().reset(&WebHooks);
    }}

    pub fn get_count(&self) -> i32 {{
        self.counter.lock().unwrap().count()
    }}

    pub fn get_logs(&self) -> String {{
        let counter = self.counter.lock().unwrap();
        let logs: Vec<String> = counter.logs().iter().rev()
            .map(|l| format!("[{{}}] {{}}", l.timestamp.format("%H:%M:%S"), l.message))
            .collect();
        logs.join("\n")
    }}
}}
"#
    );
    fs::write(path.join("src").join("lib.rs"), lib_rs)?;

    // Web Site setup
    let site_path = path.join("site");
    fs::create_dir_all(&site_path)?;

    let package_json = format!(
        r#"{{
  "scripts": {{
    "prebuild": "wasm-pack build --target web --out-dir",
    "build": "webpack --config webpack.config.js",
    "serve": "npx serve dist"
  }},
  "devDependencies": {{
    "copy-webpack-plugin": "^12.0.2",
    "webpack": "^5.91.0",
    "webpack-cli": "^5.1.4",
    "webpack-dev-server": "^5.0.4"
  }}
}}
"#
    );
    fs::write(site_path.join("package.json"), package_json)?;

    let webpack_config = r#"const CopyPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",
        publicPath: "./",
    },
    mode: "development",
    experiments: {
        asyncWebAssembly: true,
    },
    optimization: {
        splitChunks: false,
        runtimeChunk: false,
    },
    resolve: {
        symlinks: false,
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: "index.html" }
            ],
        }),
    ],
    devServer: {
        client: {
            overlay: false,
        }
    }
};
"#;
    fs::write(site_path.join("webpack.config.js"), webpack_config)?;

    let pkg_import_name = crate_name.replace('-', "_");
    let index_js = format!(
        r#"import init, {{ WebApp }} from '../pkg/{pkg_import_name}.js';

async function run() {{
    await init();
    const app = new WebApp();
    
    const countEl = document.getElementById('count');
    const logsEl = document.getElementById('logs');
    
    const updateUI = () => {{
        countEl.innerText = app.get_count();
        logsEl.innerText = app.get_logs();
    }};

    document.getElementById('inc').onclick = () => {{ app.increment(); updateUI(); }};
    document.getElementById('dec').onclick = () => {{ app.decrement(); updateUI(); }};
    document.getElementById('res').onclick = () => {{ app.reset(); updateUI(); }};

    updateUI();
}}

run();
"#
    );
    fs::write(site_path.join("index.js"), index_js)?;

    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name} Web</title>
    <style>
        body {{
            background-color: #121212;
            color: #e0e0e0;
            margin: 0;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100vh;
            font-family: sans-serif;
        }}
        .controls {{ margin: 20px; }}
        button {{ padding: 10px 20px; margin: 5px; cursor: pointer; }}
        #logs {{ 
            width: 80%; 
            height: 200px; 
            background: #1e1e1e; 
            padding: 10px; 
            overflow-y: auto; 
            white-space: pre-wrap;
            border: 1px solid #333;
        }}
    </style>
</head>
<body>
    <h1>{name} Counter</h1>
    <div style="font-size: 3em;" id="count">0</div>
    <div class="controls">
        <button id="dec">-</button>
        <button id="res">Reset</button>
        <button id="inc">+</button>
    </div>
    <div id="logs"></div>
    <script src="index.js"></script>
</body>
</html>
"#
    );
    fs::write(site_path.join("index.html"), index_html)?;

    // Publish Config
    let publish_json = format!(
        r#"{{
  "name": "{name}",
  "description": "{name} project",
  "workflow": "npm-webpack",
  "distDir": "dist",
  "buildCmd": "npm run build"
}}
"#
    );
    fs::write(site_path.join("publish.json"), publish_json)?;

    Ok(())
}
