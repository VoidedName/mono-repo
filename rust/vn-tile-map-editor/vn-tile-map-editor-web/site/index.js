import init, { main_web } from "../pkg/vn_tile_map_editor_web";

async function run() {
    await init();        // 🔑 REQUIRED
    main_web();
}

run();