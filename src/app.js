import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { cdate } from 'cdate';

import { escapeHTML } from './util.js';

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
  let html = '';
  Config.clocks.map(function (clock) {
    clock.id = "mclk-" + clock.name.toLowerCase().replace(/[^a-zA-Z0-9]/g, '-');
    html = html + renderClockHTML(clock);
    clock.fn = cdate().locale(Config.locale).tz(clock.timezone).cdateFn();
  });
  elClocks.innerHTML = "<ul>" + html + "</ul>";
  Config.clocks.map(function (clock, index) {
    if (index !== Config.clocks.length - 1) {
      document.getElementById(clock.id).style.paddingRight = Config.margin;
    }
  });
}

function renderClockHTML(clock) {
  return "<li>" + escapeHTML(clock.name) + " <span id='" + clock.id + "'></span></li>";
}

function tick() {
  tock()
  setTimeout(tick, 1000 - Date.now() % 1000);
}

function tock() {
  Config.clocks.map(function (clock) {
    document.getElementById(clock.id).innerHTML = escapeHTML(clock.fn().format(Config.format));
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
