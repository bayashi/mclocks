import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { openPath } from '@tauri-apps/plugin-opener';

import { cdate } from 'cdate';

import { escapeHTML, pad, enqueueNotification, openMessageDialog } from './util.js';

export function initClocks(clockCtx, cfg, clocks) {
  let clocksHtml = '';

  for (const [index, clock] of clocks.getAllClocks().entries()) {
    clock.id = `mclk-${index}`;
    clocksHtml += renderClockHTML(clockCtx, clock);
    if (!clock.countdown) {
      clock.fn = cdate().locale(cfg.locale).tz(clock.timezone).cdateFn();
    }
  }

  clockCtx.mainElement().innerHTML = `<ul>${clocksHtml}</ul>`;

  for (const clock of clocks.getAllClocks()) {
    clock.el = document.getElementById(clock.id);
    clock.el.style.paddingRight = cfg.margin;

    if (clock.countdown) {
      if (!clockCtx.disableHover()) {
        clock.el.title = clock.timerName ?? `Until ${cdate(clock.target).tz(clock.timezone).format("YYYY-MM-DD HH:mm:ssZ")}`;
      }
    } else if (clock.isEpoch) {
      if (!clockCtx.disableHover()) {
        clock.el.title = "elapsed since 1970-01-01T00:00:00Z";
      }
      clock.el.parentElement.hidden = !clockCtx.displayEpoch();
      clock.el.parentElement.style.display = clockCtx.displayEpoch() ? "inline" : "none";
    } else {
      if (!clockCtx.disableHover()) {
        clock.el.title = `${clock.timezone} ${clock.fn().format("Z")}`;
      }
    }
  }

  function renderClockHTML(clockCtx, clock) {
    if (clock.countdown) {
      return `<li><span id='${clock.id}'>${escapeHTML(buildCountdown(clockCtx, clock))}</span></li>`;
    } else {
      return `<li>${escapeHTML(clock.name)} <span id='${clock.id}'></span></li>`;
    }
  }
}

export async function adjustWindowSize(clockCtx, clocks) {
  let w = 0;

  for (const clock of clocks.getAllClocks()) {
    tock(clockCtx, clock);
    w += clock.el.parentElement.offsetWidth;
  }

  try {
    const currentWindow = getCurrentWindow();
    await currentWindow.setSize(new LogicalSize(w + 16, clockCtx.mainElement().offsetHeight + 16));
  } catch (e) {
    // Fallback for testing environment where Tauri APIs are not available
    console.warn('Could not adjust window size (testing environment?):', e);
    // Don't throw error in testing environment, just log it
  }
}

export function startClocks(clockCtx, clocks) {
  for (const clock of clocks.getAllClocks()) {
    tick(clockCtx, clock);
  }
}

export function buildCountdown(clockCtx, clock) {
  let diffSec = 0, diffMin = 0, diffHour = 0, diffDay = 0;

  if (!clock.isFinishCountDown) {
    let diffMS;
    if (clock.timerName) {
      // clock.pauseStart is null when not paused, so null is treated as current datetime
      diffMS = clockCtx.cdateUTC(clock.target).t - clockCtx.cdateUTC(clock.pauseStart).t;
    } else {
      // diffMS = targetMS - nowMS - offsetMS
      diffMS = cdate(clock.target).t - cdate().t - (cdate().tz(clock.timezone).utcOffset() * 60 * 1000);
    }

    diffMS = diffMS > 0 ? diffMS : 0;

    if (diffMS > 0) {
      diffSec = Math.floor(diffMS / 1000) + 1;
      diffMin = Math.floor(diffSec / 60);
      diffHour = Math.floor(diffMin / 60);
      diffDay = Math.floor(diffHour / 24);
    }

    if (diffMS === 0) {
      clock.isFinishCountDown = true;
      if (!clockCtx.withoutNotification()) {
        enqueueNotification("mclocks", `Beep! ${clock.timerName}`);
      }
    }
  }

  return clock.countdown
    .replace("%TG", cdate(clock.target).format("YYYY-MM-DD HH:mm:ssZ"))
    .replace("%D", diffDay)
    .replace("%H", pad(diffHour))
    .replace("%h", pad(diffHour % 24))
    .replace("%M", pad(diffMin))
    .replace("%m", pad(diffMin % 60))
    .replace("%S", pad(diffSec))
    .replace("%s", pad(diffSec % 60));
}

function tick(clockCtx, clock) {
  tock(clockCtx, clock);
  clock.timeoutId = setTimeout(() => { tick(clockCtx, clock); }, 1000 - Date.now() % 1000);
}

function tock(clockCtx, clock) {
  if (clock.countdown) {
    clock.el.innerHTML = escapeHTML(buildCountdown(clockCtx, clock));
  } else {
    if (clock.isEpoch) {
      clock.el.innerHTML = `${Math.trunc(clock.fn().t / 1000)}`;
    } else {
      clock.el.innerHTML = escapeHTML(clock.fn().format(clockCtx.format()));
    }
  }
}

export function switchFormat(clockCtx, cfg, clocks) {
  clockCtx.setFormat(clockCtx.format() === cfg.format && cfg.format2 ? cfg.format2 : cfg.format);
  adjustWindowSize(clockCtx, clocks);
}

export async function openToEditConfigFile(clockCtx) {
  try {
    await openMessageDialog("Please restart mclocks after editing the config.json");
    const config_path = await invoke("get_config_path");
    await openPath(config_path);
  } catch (e) {
    clockCtx.mainElement().textContent = `Err: ${e}`;
    throw new Error(e, { cause: e });
  }
}

export function toggleEpochTime(clockCtx, clocks) {
  clockCtx.setDisplayEpoch(!clockCtx.displayEpoch());
  const epochClock = clocks.getClocks().at(-1);
  epochClock.el.parentElement.hidden = !clockCtx.displayEpoch();
  epochClock.el.parentElement.style.display = clockCtx.displayEpoch() ? "inline" : "none";
  adjustWindowSize(clockCtx, clocks);
}

export function addTimerClock(clockCtx, cfg, clocks, timerInSec) {
  clocks.pushTimerClock({
    countdown: `${clockCtx.timerIcon()}%M:%s`, // The timer clock is just an alternative countdown timer
    target: clockCtx.cdateUTC().add(timerInSec, "s").text(),
    timezone: "UTC",
    id: `mclk-${clocks.getAllClocks().length - 1}`,
    timerName: `${timerInSec / 60}-minute timer`,
    pauseStart: null,
  });
  initClocks(clockCtx, cfg, clocks);
  adjustWindowSize(clockCtx, clocks);
  startClocks(clockCtx, clocks);
}
