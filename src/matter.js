import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { openPath } from '@tauri-apps/plugin-opener';

import { cdate } from 'cdate';

import { escapeHTML, pad, enqueueNotification, openMessageDialog } from './util.js';

export function initClocks(ctx, cfg, clocks) {
  let clocksHtml = '';

  for (const [index, clock] of clocks.getAllClocks().entries()) {
    clock.id = `mclk-${index}`;
    clocksHtml += renderClockHTML(ctx, clock);
    if (!clock.countdown) {
      clock.fn = cdate().locale(cfg.locale).tz(clock.timezone).cdateFn();
    }
  }

  ctx.mainElement().innerHTML = `<ul>${clocksHtml}</ul>`;

  for (const clock of clocks.getAllClocks()) {
    clock.el = document.getElementById(clock.id);
    clock.el.style.paddingRight = cfg.margin;

    if (clock.countdown) {
      if (!ctx.disableHover()) {
        clock.el.title = clock.timerName ?? `Until ${cdate(clock.target).tz(clock.timezone).format("YYYY-MM-DD HH:mm:ssZ")}`;
      }
    } else if (clock.isEpoch) {
      if (!ctx.disableHover()) {
        clock.el.title = "elapsed since 1970-01-01T00:00:00Z";
      }
      clock.el.parentElement.hidden = !ctx.displayEpoch();
      clock.el.parentElement.style.display = ctx.displayEpoch() ? "inline" : "none";
    } else {
      if (!ctx.disableHover()) {
        clock.el.title = `${clock.timezone} ${clock.fn().format("Z")}`;
      }
    }
  }

  function renderClockHTML(ctx, clock) {
    if (clock.countdown) {
      return `<li><span id='${clock.id}'>${escapeHTML(buildCountdown(ctx, clock))}</span></li>`;
    } else {
      return `<li>${escapeHTML(clock.name)} <span id='${clock.id}'></span></li>`;
    }
  }
}

export async function adjustWindowSize(ctx, clocks) {
  let w = 0;

  for (const clock of clocks.getAllClocks()) {
    tock(ctx, clock);
    w += clock.el.parentElement.offsetWidth;
  }

  try {
    const currentWindow = getCurrentWindow();
    await currentWindow.setSize(new LogicalSize(w + 16, ctx.mainElement().offsetHeight + 16));
  } catch (e) {
    // Fallback for testing environment where Tauri APIs are not available
    console.warn('Could not adjust window size (testing environment?):', e);
    // Don't throw error in testing environment, just log it
  }
}

export function startClocks(ctx, clocks) {
  for (const clock of clocks.getAllClocks()) {
    tick(ctx, clock);
  }
}

export function buildCountdown(ctx, clock) {
  let diffSec = 0, diffMin = 0, diffHour = 0, diffDay = 0;

  if (!clock.isFinishCountDown) {
    let diffMS;
    if (clock.timerName) {
      // clock.pauseStart is null when not paused, so null is treated as current datetime
      diffMS = ctx.cdateUTC(clock.target).t - ctx.cdateUTC(clock.pauseStart).t;
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
      if (!ctx.withoutNotification()) {
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

function tick(ctx, clock) {
  tock(ctx, clock);
  clock.timeoutId = setTimeout(() => { tick(ctx, clock); }, 1000 - Date.now() % 1000);
}

function tock(ctx, clock) {
  if (clock.countdown) {
    clock.el.innerHTML = escapeHTML(buildCountdown(ctx, clock));
  } else {
    if (clock.isEpoch) {
      clock.el.innerHTML = `${Math.trunc(clock.fn().t / 1000)}`;
    } else {
      clock.el.innerHTML = escapeHTML(clock.fn().format(ctx.format()));
    }
  }
}

export function switchFormat(ctx, cfg, clocks) {
  ctx.setFormat(ctx.format() === cfg.format && cfg.format2 ? cfg.format2 : cfg.format);
  adjustWindowSize(ctx, clocks);
}

export async function openToEditConfigFile(ctx) {
  try {
    await openMessageDialog("Please restart mclocks after editing the config.json");
    const config_path = await invoke("get_config_path");
    await openPath(config_path);
  } catch (e) {
    ctx.mainElement().textContent = `Err: ${e}`;
    throw new Error(e, { cause: e });
  }
}

export function toggleEpochTime(ctx, clocks) {
  ctx.setDisplayEpoch(!ctx.displayEpoch());
  const epochClock = clocks.getClocks().at(-1);
  epochClock.el.parentElement.hidden = !ctx.displayEpoch();
  epochClock.el.parentElement.style.display = ctx.displayEpoch() ? "inline" : "none";
  adjustWindowSize(ctx, clocks);
}

export function addTimerClock(ctx, cfg, clocks, timerInSec) {
  clocks.pushTimerClock({
    countdown: `${ctx.timerIcon()}%M:%s`, // The timer clock is just an alternative countdown timer
    target: ctx.cdateUTC().add(timerInSec, "s").text(),
    timezone: "UTC",
    id: `mclk-${clocks.getAllClocks().length - 1}`,
    timerName: `${timerInSec / 60}-minute timer`,
    pauseStart: null,
  });
  initClocks(ctx, cfg, clocks);
  adjustWindowSize(ctx, clocks);
  startClocks(ctx, clocks);
}
