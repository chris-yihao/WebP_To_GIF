import { spawnSync } from "node:child_process";
import process from "node:process";

function run(command, args) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: process.platform === "win32",
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

run("tauri", ["build"]);

if (process.platform === "darwin") {
  run("node", ["scripts/fix-macos-bundle.mjs"]);
}
