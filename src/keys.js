import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener'

import { cdate } from 'cdate';

import { adjustWindowSize, addTimerClock } from './matter.js';
import { trim, uniqueTimezones, writeClipboardText, readClipboardText, openAskDialog, openMessageDialog } from './util.js';

export function operationKeysHandler(e, ctx, cfg, clocks) {
  if (e.metaKey) {
    if (e.key === "d") {
      e.preventDefault(); // ignore "Windows + D" to keep displaying mclocks
      return;
    }
  }

  if (e.ctrlKey) {
    withCtrl(e, ctx, cfg, clocks);
  }
}

// operations to be pressed together with Ctrl key
async function withCtrl(e, ctx, cfg, clocks) {
  // switch date-time format if format2 would be defined
  if (e.key === "f") {
    e.preventDefault();
    ctx.setFormat(ctx.format() === cfg.format && cfg.format2 ? cfg.format2 : cfg.format);
    adjustWindowSize(ctx, clocks);
    return;
  }

  if (e.key === "o") {
    e.preventDefault();
    try {
      await openMessageDialog("Please restart mclocks after editing the config.json");
      const config_path = await invoke("get_config_path");
      await openPath(config_path);
    } catch (e) {
      ctx.mainElement().textContent = "Err: " + e;
      throw new Error(e);
    }
    return;
  }

  // toggle to display Epoch time
  if (e.key === "e" || e.key === "u") {
    e.preventDefault();
    ctx.setDisplayEpoch(!ctx.displayEpoch());
    const epochClock = clocks.getClocks().at(-1);
    epochClock.el.parentElement.hidden = !ctx.displayEpoch();
    epochClock.el.parentElement.style.display = ctx.displayEpoch() ? "inline" : "none";
    adjustWindowSize(ctx, clocks);
    return;
  }

  if (e.key) {
    const input = Number(e.key);
    // add timer clock
    if (input >= 1 && input <= 9) {
      e.preventDefault();
      // Ctrl + 1       --> start 1 min timer
      // Ctrl + Alt + 1 --> start 10 mins timer
      const coef = e.altKey ? 600 : 60;
      if (clocks.getTimerClocks().length < ctx.maxTimerClockNumber()) {
        addTimerClock(ctx, cfg, clocks, input * coef);
      }
      return;
    }

    // remove timer-clock
    if (input === 0 && clocks.getTimerClocks().length > 0) {
      e.preventDefault();
      if (e.altKey) {
        // Ctrl + Alt + 0 --> remove newest timer (remove the clock on the far right)
        clocks.removeTimerRight();
      } else {
        // Ctrl + 0 --> remove oldest timer (remove the clock on the far left)
        clocks.removeTimerLeft();
      }
      adjustWindowSize(ctx, clocks);
      return;
    }
  }

  // pause and re-start timer clocks (Not able to control each timer clock)
  if (e.key === "p") {
    e.preventDefault();
    if (clocks.getTimerClocks().length > 0) {
      if (ctx.lockKeyP()) {
        return
      }
      ctx.setLockKeyP(true);
      ctx.setPauseTimer(!ctx.pauseTimer());
      clocks.getTimerClocks().map((clock) => {
        if (ctx.pauseTimer()) {
          // pause
          if (!clock.pauseStart) {
            clock.pauseStart = ctx.cdateUTC().text();
          }
        } else {
          // re-start
          const pauseDiffMS = ctx.cdateUTC().t - ctx.cdateUTC(clock.pauseStart).t
          clock.target = ctx.cdateUTC(clock.target).add(pauseDiffMS, "ms").text();
          clock.pauseStart = null;
        }
      });
      ctx.setLockKeyP(false);
      return;
    }
  }

  // send current clocks to clipboard
  if (e.key === "c") {
    e.preventDefault();
    let cls = [];
    clocks.getAllClocks().map((clock) => {
      if (clock.isEpoch && !ctx.displayEpoch()) {
        return;
      }
      cls.push(clock.el.parentElement.innerText);
    })
    writeClipboardText(cls.join("  "));
    return;
  }

  // convert between epoch time and date-time to paste from the clipboard
  if (e.key === "v") {
    e.preventDefault();
    conversionHandler(e, ctx, clocks);
  }
}

async function conversionHandler(e, ctx, clocks) {
  let origClipboardText = trim(await readClipboardText());
  let src = origClipboardText;
  let isDateTimeText = true;
  if (src.match(/^[0-9.]+$/)) {
    // Ctrl + v       --> convert epoch in sec
    // Ctrl + Alt + v --> convert epoch in ms
    src = e.altKey ? Number(src) : Number(src) * 1000;
    isDateTimeText = false;
  }
  let unit = " in " + (e.altKey ? "milliseconds" : "seconds");

  try {
    cdate(src).tz("UTC").get("year");
  } catch (e) {
    await openMessageDialog("Could not convert the clipboard text.\n\n" + e, "mclocks Error", "error");
    return;
  }

  let results = [];
  uniqueTimezones(clocks).map((tz) => {
    results.push(cdate(src).tz(tz).text() + " in " + tz);
  });
  if (isDateTimeText) {
    results.push((cdate(src).t / 1000) + " Epoch in seconds" )
    results.push(cdate(src).t + " Epoch in milliseconds")
  }

  const body = origClipboardText + (isDateTimeText ? "" : unit) + "\n\n" + results.join("\n") + "\n";
  if (await openAskDialog(body + "\nPress [Y] to copy the result.", "mclocks converter")) {
    writeClipboardText(body);
  }

  return;
}
