import { cpSync, existsSync, rmSync, symlinkSync } from "node:fs";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join } from "node:path";

const root = process.cwd();
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const version = packageJson.version;
const appName = "WebP 转 GIF";
const appPath = join(
  root,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "macos",
  `${appName}.app`,
);
const plistPath = join(appPath, "Contents", "Info.plist");
const dmgPath = join(
  root,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "dmg",
  `${appName}_${version}_aarch64.dmg`,
);
const dmgSourcePath = join(
  root,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "dmg",
  `${appName}-dmg-source`,
);

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    ...options,
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

if (!existsSync(appPath)) {
  console.log("macOS app bundle not found; skipping macOS bundle fix.");
  process.exit(0);
}

spawnSync("plutil", ["-remove", "LSRequiresCarbon", plistPath], {
  stdio: "ignore",
});
run("codesign", ["--force", "--deep", "--sign", "-", appPath]);
run("codesign", ["--verify", "--deep", "--strict", "--verbose=4", appPath]);

if (existsSync(dmgPath)) {
  rmSync(dmgPath);
}

if (existsSync(dmgSourcePath)) {
  rmSync(dmgSourcePath, { recursive: true, force: true });
}

cpSync(appPath, join(dmgSourcePath, `${appName}.app`), { recursive: true });
symlinkSync("/Applications", join(dmgSourcePath, "Applications"));

run("hdiutil", [
  "create",
  "-volname",
  appName,
  "-srcfolder",
  dmgSourcePath,
  "-ov",
  "-format",
  "UDZO",
  dmgPath,
]);

rmSync(dmgSourcePath, { recursive: true, force: true });
