import init, { run_app } from "./pkg/webapp_frontend.js";

async function main() {
  await init("/pkg/webapp_frontend_bg.wasm");
  run_app();
}

main();
