import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { platform } from '@tauri-apps/plugin-os';
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state';
import { cdate } from 'cdate';

import { escapeHTML, pad } from './util.js';

const currentPlatform = platform();
let elClocks = document.querySelector("#mclocks");
let Config;
let ignoreOnMoved = false
let switchFormat = false

window.addEventListener("DOMContentLoaded", async () => {
  elClocks.addEventListener("mousedown", async () => {
    await getCurrentWindow().startDragging();
  })

  elClocks.addEventListener("click", async () => {
    if (!Config || !Config.format2) {
      return
    }
    switchFormat = !switchFormat
    adjustWindowSize()
  })

  await getCurrentWindow().onMoved(() => {
    // Not support to save window-state onMoved for macOS.
    // Just save window-state on quit mclocks for macOS.
    // Because `saveWindowState(StateFlags.ALL)` doesn't work somehow on macOS (at least Mac OS 15.4.0 arm64 X64)
    if (ignoreOnMoved || currentPlatform === 'macos') {
      return
    }
    ignoreOnMoved = true
    setTimeout(async function () {
      await saveWindowState(StateFlags.ALL);
      ignoreOnMoved = false;
    }, 5000);
  });

  main()
});

async function main() {
  try {
    Config = await invoke("load_config", {});
  } catch (e) {
    console.log("Err load_config", e);
    elClocks.textContent = "Err: " + e;
    return;
  }

  initStyles();
  initClocks();
  adjustWindowSize()
  setAlwaysOnTop()

  Config.clocks.map(function (clock) {
    tick(clock);
  });
}

function initStyles() {
  const AppStyle = elClocks.style;
  AppStyle.fontFamily = Config.font;
  AppStyle.color = Config.color;
  AppStyle.fontSize = Config.size + 'px';
}

function initClocks() {
  let clocksHtml = '';
  Config.clocks.map(function (clock, index) {
    clock.id = "mclk-id-" + index;
    clocksHtml = clocksHtml + renderClockHTML(clock);
    clock.fn = cdate().locale(Config.locale).tz(clock.timezone).cdateFn();
  });
  elClocks.innerHTML = "<ul>" + clocksHtml + "</ul>";
  Config.clocks.map(function (clock, index) {
    clock.el = document.getElementById(clock.id)
    if (index !== Config.clocks.length - 1) {
      clock.el.style.paddingRight = Config.margin;
    }
    if (clock.countdown.length > 0) {
      clock.el.title = "Until " + clock.target;
    }
  });
}

function renderClockHTML(clock) {
  if (clock.countdown.length > 0) {
    return "<li><span id='" + clock.id + "'>" + buildCountdown(clock.target, clock.timezone, clock.countdown) + "</span></li>";
  } else {
    return "<li>" + escapeHTML(clock.name) + " <span id='" + clock.id + "'></span></li>";
  }
}

function tick(clock) {
  tock(clock)
  setTimeout(function(){ tick(clock) }, 1000 - Date.now() % 1000);
}

function tock(clock) {
  if (clock.target) {
    clock.el.innerHTML = escapeHTML(buildCountdown(clock.target, clock.timezone, clock.countdown));
  } else {
    let format = Config.format
    if (switchFormat) {
      format = Config.format2
    }
    clock.el.innerHTML = escapeHTML(clock.fn().format(format));
  }
}

async function adjustWindowSize() {
  let w = 0;
  Config.clocks.map(function (clock) {
    tock(clock)
    w += clock.el.parentElement.offsetWidth;
  });

  try {
    await getCurrentWindow().setSize(new LogicalSize(w, elClocks.offsetHeight + 4))
  } catch (e) {
    console.log("err setSize", e);
    elClocks.textContent = "Err: " + e;
  }
}

async function setAlwaysOnTop() {
  try {
    await getCurrentWindow().setAlwaysOnTop(Config.forefront)
  } catch (e) {
    console.log("err setAlwaysOnTop", e);
    elClocks.textContent = "Err: " + e;
  }
}

window.addEventListener('keydown', (event) => {
  if (event.key === 'd' && event.metaKey) { // Windows + D
    event.preventDefault();
  }
});

function buildCountdown(target, timezone, countdownText) {
  let diffSec = 0, diffMin = 0, diffHour = 0, diffDay = 0

  // diffMS = targetMS - nowMS - offsetMS
  let diffMS = cdate(target).t - cdate().t - (cdate().tz(timezone).utcOffset() * 60 * 1000);
  diffMS = diffMS > 0 ? diffMS : 0;

  if (diffMS > 0) {
    diffSec = Math.floor(diffMS / 1000) + 1;
    diffMin = Math.floor(diffSec / 60);
    diffHour = Math.floor(diffMin / 60);
    diffDay = Math.floor(diffHour / 24);
  }

  return countdownText
    .replace("%TG", target)
    .replace("%D", diffDay)
    .replace("%H", pad(diffHour))
    .replace("%h", pad(diffHour % 24))
    .replace("%M", pad(diffMin))
    .replace("%m", pad(diffMin % 60))
    .replace("%S", pad(diffSec))
    .replace("%s", pad(diffSec % 60))
}
