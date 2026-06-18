# WebP 转 GIF

一个简单直白的 WebP 转 GIF 桌面工具。拖入或选择 `.webp` 文件后会自动转换为 `.gif`，并尽量保留动态帧和播放时长。

当前版本：`1.0.1`  
作者：`Chris_yihao`

## 功能

- 支持单张 WebP 转 GIF
- 支持批量拖入多个 WebP 文件
- 支持动态 WebP，转换后保留动画效果
- 默认在软件同级位置创建 `GIF` 文件夹并输出到里面
- 支持手动选择新的输出文件夹
- 显示转换进度、成功/失败状态和本次打开期间的最近完成记录
- 文件重名时自动生成 `文件名 (1).gif`，不覆盖已有文件
- 窗口尺寸固定，批量文件较多时列表在模块内滚动

## 改动日志

### 1.0.1

- 软件显示名称统一改为 `WebP 转 GIF`。
- 拖入 PNG、GIF、JPG 等非 WebP 文件时，会在列表中显示“不是 WebP 文件，已跳过”，不再静默忽略。
- 最近完成记录改为只保留当前运行期间，关闭软件后自动清空。
- 删除“更多设置”入口，右侧“正在转换”和“最近完成”改为等高布局。
- 更新 README 中的使用说明、打包产物路径和注意事项。
- Windows 发布版使用 GUI 子系统，直接运行时不再额外弹出控制台窗口。
- 新增 Windows 单文件便携版生成脚本，可把主程序和 WebView2 loader 打包进一个 `.exe`。

### 1.0.0

- 初始版本：支持单个或批量 WebP 转 GIF。
- 支持动态 WebP，转换后保留动画帧和播放时长。
- 默认输出到软件同级目录下的 `GIF` 文件夹，并支持手动选择输出目录。
- 支持转换进度、成功/失败状态和最近完成列表。
- 提供 macOS `.app/.dmg` 打包配置。

## 使用方式

1. 打开 `WebP 转 GIF`。
2. 把 `.webp` 文件拖到主窗口中，或点击 `选择 WebP 文件`。
3. 软件会自动开始转换。
4. 转换完成后，GIF 会出现在底部显示的输出目录里。
5. 点击 `选择文件夹` 可以切换输出位置。
6. 点击 `打开 GIF 文件夹` 可以快速查看转换结果。

## 输出规则

默认输出目录是软件可执行文件同级的 `GIF` 文件夹。

示例：

```text
WebP 转 GIF.exe
GIF/
  demo.gif
  demo (1).gif
```

如果用户选择了其他输出目录，软件会记住该目录，下次打开继续使用。

## 技术栈

- 桌面框架：Tauri v2
- 前端：Vite + vanilla TypeScript + CSS
- 后端：Rust
- WebP 动画解码：`webp-animation`
- GIF 编码：`gif`

选择这套方案的目标是让应用体积尽量小、启动速度快，并避免依赖 ImageMagick 或 FFmpeg 这类较大的外部程序。

## 开发

安装依赖：

```bash
npm install
```

启动前端预览：

```bash
npm run dev
```

启动 Tauri 开发模式：

```bash
npm run tauri:dev
```

## 测试

前端构建检查：

```bash
npm run build
```

布局固定检查：

```bash
node scripts/check-fixed-layout.mjs
```

Rust 测试：

```bash
cd src-tauri
cargo test
```

## 打包

macOS 和 Windows 都使用同一个打包入口：

```bash
npm run tauri:build
```

这个命令会先执行 Tauri 的正常发布构建。macOS 下会在构建完成后自动运行 `scripts/fix-macos-bundle.mjs`，修正 `.app/.dmg` 包；Windows 下会直接生成 Windows 安装包。

macOS 构建产物通常在：

```text
src-tauri/target/release/bundle/macos/WebP 转 GIF.app
src-tauri/target/release/bundle/dmg/WebP 转 GIF_1.0.1_aarch64.dmg
```

打开 DMG 后，把 `WebP 转 GIF.app` 拖到 `Applications` 图标上即可安装到“应用程序”。

Windows 安装包通常在：

```text
src-tauri/target/release/bundle/nsis/WebP 转 GIF_1.0.1_x64-setup.exe
```

Windows 也会生成直接运行版：

```text
src-tauri/target/release/webPToGif.exe
src-tauri/target/release/WebView2Loader.dll
```

如果需要对外只分发一个 `.exe`，在 Windows 上先执行 `npm run tauri:build`，再执行：

```bash
npm run portable:windows
```

单文件便携版会生成到：

```text
src-tauri/target/release/WebP_To_GIF_Portable.exe
```

正式发布 macOS 版本时，还需要按 Apple 要求配置签名和公证。

Windows 单文件便携版运行时会把内置的主程序和 WebView2 loader 解压到系统临时目录后启动。用户只需要双击这一个 `.exe`，界面仍然只有主应用窗口。

## 注意事项

- GIF 格式最多支持 256 色，转换后颜色可能和原 WebP 略有差异。
- 第一版只做 WebP 到 GIF，不包含尺寸调整、压缩质量、帧率修改等高级设置。
- 如果某个文件损坏会显示失败状态；如果拖入的不是 WebP，会显示“不是 WebP 文件，已跳过”，不会影响其他文件继续转换。
- 最近完成记录只保留在当前运行期间，关闭软件后会自动清空。
