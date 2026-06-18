/*
 * Copyright (c) 2026 Chris_yihao.
 * Author: Chris_yihao
 * Time: 2026-06-17
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import "./styles.css";

type JobStatus = "queued" | "converting" | "completed" | "failed";

type ConversionJob = {
  filePath: string;
  status: JobStatus;
  progress: number;
  outputPath?: string | null;
  error?: string | null;
};

type RecentItem = {
  filePath: string;
  outputPath?: string | null;
  status: string;
  message: string;
};

const isTauri = "__TAURI_INTERNALS__" in window;

const state = {
  outputDir: "软件所在文件夹 / GIF",
  jobs: new Map<string, ConversionJob>(),
  dragActive: false,
};
let previewJobCount = 0;

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("App root not found");
}

const root = app;

function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

function humanStatus(job: ConversionJob): string {
  if (job.status === "completed") return "已完成";
  if (job.status === "failed") return job.error || "转换失败";
  if (job.status === "queued") return "等待转换";
  return "正在转换";
}

function latestJobs(): ConversionJob[] {
  return Array.from(state.jobs.values()).reverse();
}

function completedHistory(): RecentItem[] {
  return Array.from(state.jobs.values())
    .filter((job) => job.status === "completed" || job.status === "failed")
    .map((job) => ({
      filePath: job.filePath,
      outputPath: job.outputPath,
      status: job.status,
      message: job.status === "completed" ? "转换完成" : job.error || "转换失败",
    }))
    .slice(0, 50);
}

function render(): void {
  const activeJobs = latestJobs();
  const history = completedHistory();

  root.innerHTML = `
    <section class="shell">
      <header class="header">
        <div>
          <h1>WebP 转 GIF</h1>
          <p>拖入文件后自动转换，动图效果会保留</p>
        </div>
      </header>

      <section class="workspace">
        <button class="drop-zone ${state.dragActive ? "is-active" : ""}" id="drop-zone" type="button">
          <span class="plus">+</span>
          <strong>把 WebP 文件拖到这里</strong>
          <span>也可以一次拖入多个文件</span>
          <span class="primary-action">选择 WebP 文件</span>
          <small>转换完成后会生成 GIF 文件</small>
        </button>

        <aside class="side-panel">
          <section class="panel">
            <h2>正在转换</h2>
            <div class="job-list">
              ${
                activeJobs.length
                  ? activeJobs
                      .map(
                        (job) => `
                          <article class="job ${job.status}">
                            <div class="file-icon"></div>
                            <div class="job-body">
                              <div class="job-row">
                                <strong title="${job.filePath}">${fileName(job.filePath)}</strong>
                                <span>${job.progress}%</span>
                              </div>
                              <div class="progress"><span style="width:${job.progress}%"></span></div>
                              <p>${humanStatus(job)}</p>
                            </div>
                          </article>
                        `,
                      )
                      .join("")
                  : `<div class="empty">还没有文件。拖入 WebP 后会自动开始。</div>`
              }
            </div>
          </section>

          <section class="panel history-panel">
            <div class="panel-title-row">
              <h2>最近完成</h2>
              <button class="link-button" id="open-output" type="button">打开 GIF 文件夹</button>
            </div>
            <div class="history-list">
              ${
                history.length
                  ? history
                      .map(
                        (item) => `
                          <article class="history-item ${item.status}">
                            <span title="${item.filePath}">${fileName(item.filePath)}</span>
                            <small>${item.message}</small>
                          </article>
                        `,
                      )
                      .join("")
                  : `<div class="empty">完成的 GIF 会显示在这里。</div>`
              }
            </div>
          </section>
        </aside>
      </section>

      <footer class="output-bar">
        <span>输出位置</span>
        <strong title="${state.outputDir}">${state.outputDir}</strong>
        <button id="choose-output" type="button">选择文件夹</button>
      </footer>
    </section>
  `;

  bindDomEvents();
}

function bindDomEvents(): void {
  const dropZone = document.querySelector<HTMLButtonElement>("#drop-zone");
  const chooseOutput = document.querySelector<HTMLButtonElement>("#choose-output");
  const openOutput = document.querySelector<HTMLButtonElement>("#open-output");

  dropZone?.addEventListener("click", () => chooseFiles());
  chooseOutput?.addEventListener("click", () => chooseOutputDir());
  openOutput?.addEventListener("click", () => openOutputDir());

  dropZone?.addEventListener("dragenter", (event) => {
    event.preventDefault();
    state.dragActive = true;
    render();
  });

  dropZone?.addEventListener("dragover", (event) => {
    event.preventDefault();
  });

  dropZone?.addEventListener("dragleave", (event) => {
    event.preventDefault();
    state.dragActive = false;
    render();
  });

  dropZone?.addEventListener("drop", (event) => {
    event.preventDefault();
    state.dragActive = false;
    const paths = Array.from(event.dataTransfer?.files ?? [])
      .map((file) => (file as File & { path?: string }).path)
      .filter((path): path is string => Boolean(path));
    void startConversion(paths);
  });
}

async function chooseFiles(): Promise<void> {
  if (!isTauri) {
    seedPreviewJob();
    return;
  }

  const selected = await open({
    multiple: true,
    directory: false,
    filters: [{ name: "WebP", extensions: ["webp"] }],
  });

  const paths = Array.isArray(selected) ? selected : selected ? [selected] : [];
  await startConversion(paths);
}

async function chooseOutputDir(): Promise<void> {
  if (!isTauri) {
    state.outputDir = "自选文件夹 / GIF";
    render();
    return;
  }

  const selected = await invoke<string | null>("choose_output_dir");
  if (selected) {
    state.outputDir = selected;
    render();
  }
}

async function openOutputDir(): Promise<void> {
  if (!isTauri) return;
  await invoke("open_output_dir");
}

async function startConversion(paths: string[]): Promise<void> {
  const webpPaths = paths.filter((path) => path.toLowerCase().endsWith(".webp"));
  const skippedPaths = paths.filter((path) => !path.toLowerCase().endsWith(".webp"));

  for (const path of skippedPaths) {
    state.jobs.set(path, {
      filePath: path,
      status: "failed",
      progress: 100,
      error: "不是 WebP 文件，已跳过",
    });
  }

  if (!webpPaths.length && !skippedPaths.length) {
    return;
  }

  for (const path of webpPaths) {
    state.jobs.set(path, {
      filePath: path,
      status: "queued",
      progress: 0,
    });
  }
  render();

  if (!webpPaths.length) {
    return;
  }

  if (!isTauri) {
    seedPreviewJob();
    return;
  }

  await invoke<ConversionJob[]>("convert_files", { paths: webpPaths });
}

function seedPreviewJob(): void {
  previewJobCount += 1;
  const path = `/Users/example/Desktop/banner-${previewJobCount}.webp`;
  state.jobs.set(path, {
    filePath: path,
    status: "converting",
    progress: 54,
  });
  render();
}

async function bootstrap(): Promise<void> {
  render();

  if (!isTauri) {
    return;
  }

  await getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === "over") {
      state.dragActive = true;
      render();
    }

    if (event.payload.type === "leave") {
      state.dragActive = false;
      render();
    }

    if (event.payload.type === "drop") {
      state.dragActive = false;
      void startConversion(event.payload.paths);
    }
  });

  state.outputDir = await invoke<string>("get_output_dir");
  await listen<ConversionJob>("conversion-progress", (event) => {
    state.jobs.set(event.payload.filePath, event.payload);
    render();
  });

  render();
}

void bootstrap();
