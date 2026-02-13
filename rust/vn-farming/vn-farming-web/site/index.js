import init, { main_web } from "../pkg/vn_farming_web";

async function run() {
    await init();        // 🔑 REQUIRED
    main_web();
}

run();