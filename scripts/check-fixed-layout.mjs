import fs from "node:fs";

const css = fs.readFileSync(new URL("../src/styles.css", import.meta.url), "utf8");
const mainTs = fs.readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
const indexHtml = fs.readFileSync(new URL("../index.html", import.meta.url), "utf8");
const readme = fs.readFileSync(new URL("../README.md", import.meta.url), "utf8");
const macosBundleFix = fs.readFileSync(
  new URL("../scripts/fix-macos-bundle.mjs", import.meta.url),
  "utf8",
);
const tauriLib = fs.readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
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
hasRule(".side-panel > .panel", "flex", "1 1 0");
hasRule(".job-list", "overflow-y", "auto");
hasRule(".history-list", "overflow-y", "auto");

if (css.includes(".secondary-button") || mainTs.includes("secondary-button")) {
  throw new Error("Expected the 更多设置 secondary button to be removed");
}

if (
  mainTs.includes("load_history") ||
  tauriLib.includes("HistoryStore") ||
  tauriLib.includes("history.json")
) {
  throw new Error("Expected recent completion history to stay session-only");
}

if (mainTs.includes("webPToGif") || indexHtml.includes("webPToGif") || readme.includes("webPToGif")) {
  throw new Error("Expected user-facing app name to be WebP 转 GIF");
}

if (tauriConfig.productName !== "WebP 转 GIF") {
  throw new Error("Expected Tauri productName to be WebP 转 GIF");
}

const [mainWindow] = tauriConfig.app.windows;
if (!mainWindow) {
  throw new Error("Missing main Tauri window config");
}

if (mainWindow.title !== "WebP 转 GIF") {
  throw new Error("Expected main window title to be WebP 转 GIF");
}

if (!macosBundleFix.includes('const appName = "WebP 转 GIF"') || !macosBundleFix.includes("`${appName}.app`")) {
  throw new Error("Expected macOS bundle fix script to use the new app name");
}

if (mainWindow.resizable !== false) {
  throw new Error("Expected main window to be fixed size with resizable: false");
}

if (mainWindow.minWidth !== mainWindow.width || mainWindow.minHeight !== mainWindow.height) {
  throw new Error("Expected min window size to match the fixed window size");
}
