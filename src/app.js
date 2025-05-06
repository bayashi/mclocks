import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state';

import { initClocks, adjustWindowSize, startClocks } from './matter.js';
import { Ctx } from './ctx.js';
import { Clocks } from './clocks.js';
import { operationKeysHandler } from './keys.js';

window.addEventListener("DOMContentLoaded", async () => {
  let ctx = new Ctx(document.querySelector("#mclocks"));
  globalInit(ctx);
  main(ctx);
});

async function globalInit(ctx) {
  await getCurrentWindow().onMoved(() => {
    // Not support to save window-state onMoved for macOS.
    // Just save window-state on quit mclocks for macOS.
    // Because `saveWindowState(StateFlags.ALL)` doesn't work somehow on macOS (at least Mac OS 15.4.0 arm64 X64)
    if (ctx.ignoreOnMoved() || ctx.isMacOS()) {
      return
    }
    ctx.setIgnoreOnMoved(true);
    setTimeout(async () => {
      await saveWindowState(StateFlags.ALL);
      ctx.setIgnoreOnMoved(false);
    }, 5000);
  });

  ctx.mainElement().addEventListener("mousedown", async () => {
    if (ctx.isMacOS()) {
      await getCurrentWindow().startDragging();
    }
  });
}

async function initConfig(ctx) {
  try {
    const config = await invoke("load_config", {});
    if (config.forefront) {
      await getCurrentWindow().setAlwaysOnTop(true);
    }
    ctx.setFormat(config.format);
    ctx.setTimerIcon(config.timerIcon);
    ctx.setWithoutNotification(config.withoutNotification);
    ctx.setMaxTimerClockNumber(config.maxTimerClockNumber);
    return config;
  } catch (e) {
    ctx.mainElement().textContent = "Err: " + e;
    throw new Error(e);
  }
}

async function main(ctx) {
  const cfg = await initConfig(ctx);
  let clocks = new Clocks(cfg.clocks, cfg.epochClockName);

  initStyles(ctx, cfg);
  initClocks(ctx, cfg, clocks);
  adjustWindowSize(ctx, clocks);

  startClocks(ctx, clocks);

  window.addEventListener('keydown', (e) => {
    operationKeysHandler(e, ctx, cfg, clocks);
  });

  async function initStyles(ctx, cfg) {
    const AppStyle = ctx.mainElement().style;
    AppStyle.fontFamily = cfg.font;
    AppStyle.color = cfg.color;
    const sizeUnit = typeof cfg.size === "number" || cfg.size.match(/^[0-1.]$/) ? "px" : "";
    AppStyle.fontSize = cfg.size + sizeUnit;
  }
}
