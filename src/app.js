import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { cdate } from 'cdate';

import { escapeHTML, pad } from './util.js';

let elClocks = document.querySelector("#mclocks");
let Config;

window.addEventListener("DOMContentLoaded", () => {
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

  tick(); // loop
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
    if (index !== Config.clocks.length - 1) {
      document.getElementById(clock.id).style.paddingRight = Config.margin;
    }
    if (clock.countdown.length > 0) {
      document.getElementById(clock.id).title = "Until " + clock.target;
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

function tick() {
  tock()
  setTimeout(tick, 1000 - Date.now() % 1000);
}

function tock() {
  Config.clocks.map(function (clock) {
    let el = document.getElementById(clock.id)
    if (clock.target) {
      el.innerHTML = escapeHTML(buildCountdown(clock.target, clock.timezone, clock.countdown));
    } else {
      el.innerHTML = escapeHTML(clock.fn().format(Config.format));
    }
  });
}

async function adjustWindowSize() {
  tock()

  let w = 0;
  Config.clocks.map(function (clock) {
    w += document.getElementById(clock.id).parentElement.offsetWidth;
  });

  try {
    await getCurrentWindow().setSize(new LogicalSize(w, elClocks.offsetHeight + 4))
  } catch(e) {
    console.log("err setSize", e);
    elClocks.textContent = "Err: " + e;
  }
}

async function setAlwaysOnTop() {
  try {
    await getCurrentWindow().setAlwaysOnTop(Config.forefront)
  } catch(e) {
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
