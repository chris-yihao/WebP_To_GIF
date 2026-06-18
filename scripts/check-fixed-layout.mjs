import fs from "node:fs";

const css = fs.readFileSync(new URL("../src/styles.css", import.meta.url), "utf8");
const tauriConfig = JSON.parse(
  fs.readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
);

function block(selector) {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = css.match(new RegExp(`${escaped}\\s*\\{([^}]*)\\}`, "m"));
  if (!match) {
    throw new Error(`Missing CSS selector: ${selector}`);
  }
  return match[1];
}

function hasRule(selector, property, value) {
  const rules = block(selector);
  const pattern = new RegExp(`${property}\\s*:\\s*${value}\\s*;`);
  if (!pattern.test(rules)) {
    throw new Error(`Expected ${selector} to include ${property}: ${value};`);
  }
}

hasRule("html", "height", "100%");
hasRule("body", "height", "100%");
hasRule("body", "overflow", "hidden");
hasRule("#app", "height", "100%");
hasRule(".shell", "height", "100dvh");
hasRule(".shell", "overflow", "hidden");
hasRule(".workspace", "overflow", "hidden");
hasRule(".side-panel", "overflow", "hidden");
hasRule(".panel", "min-height", "0");
hasRule(".job-list", "overflow-y", "auto");
hasRule(".history-list", "overflow-y", "auto");

const [mainWindow] = tauriConfig.app.windows;
if (!mainWindow) {
  throw new Error("Missing main Tauri window config");
}

if (mainWindow.resizable !== false) {
  throw new Error("Expected main window to be fixed size with resizable: false");
}

if (mainWindow.minWidth !== mainWindow.width || mainWindow.minHeight !== mainWindow.height) {
  throw new Error("Expected min window size to match the fixed window size");
}
