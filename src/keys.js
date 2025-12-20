import { cdate } from 'cdate';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';

import { adjustWindowSize, switchFormat, openToEditConfigFile, toggleEpochTime, addTimerClock } from './matter.js';
import { trim, uniqueTimezones, writeClipboardText, readClipboardText, openMessageDialog, isMacOS, isWindowsOS } from './util.js';

// Win   ---> Ctrl
// macOS ---> Command
const pressingBaseKey = (e) => isMacOS() ? e.metaKey : e.ctrlKey;

// Win   ---> Alt
// macOS ---> Control
const pressingAltKey = (e) => isMacOS() ? e.ctrlKey : e.altKey;

/**
 * Opens the help page in the default browser
 */
async function openHelpPage() {
  try {
    await openUrl("https://github.com/bayashi/mclocks?tab=readme-ov-file#keyboard-shortcuts");
  } catch (error) {
    await openMessageDialog(`Failed to open help page: ${error}`, "mclocks Error", "error");
  }
}

export function operationKeysHandler(e, pressedKeys, ctx, cfg, clocks) {
  if (isWindowsOS()) {
    if (e.metaKey && e.key === "d") {
      e.preventDefault(); // ignore "Windows + D" to keep displaying mclocks
      return;
    }

    if (e.key === "F1") {
      e.preventDefault();
      openHelpPage();
      return;
    }
  }

  if (isMacOS()) {
    if (e.metaKey && e.shiftKey && e.key === "/") {
      e.preventDefault();
      openHelpPage();
      return;
    }
  }

  if (pressingBaseKey(e)) {
    withBaseKey(e, pressedKeys, ctx, cfg, clocks);
  }
}

// operations to be pressed together with Ctrl key
async function withBaseKey(e, pressedKeys, ctx, cfg, clocks) {
  // switch date-time format if format2 would be defined
  if (e.key === "f") {
    e.preventDefault();
    switchFormat(ctx, cfg, clocks);
    return;
  }

  if (e.key === "o") {
    e.preventDefault();
    openToEditConfigFile(ctx);
    return;
  }

  // toggle to display Epoch time
  if (e.key === "e" || e.key === "u") {
    e.preventDefault();
    toggleEpochTime(ctx, clocks);
    return;
  }

  if (e.key) {
    const input = Number(e.key);
    // add timer clock
    if (input >= 1 && input <= 9) {
      e.preventDefault();
      // Ctrl + 1       --> start 1 min timer
      // Ctrl + Alt + 1 --> start 10 mins timer
      const coef = pressingAltKey(e) ? 600 : 60;
      if (clocks.getTimerClocks().length < ctx.maxTimerClockNumber()) {
        addTimerClock(ctx, cfg, clocks, input * coef);
      }
      return;
    }

    // remove timer-clock
    if (input === 0 && clocks.getTimerClocks().length > 0) {
      e.preventDefault();
      if (pressingAltKey(e)) {
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
        return;
      }
      ctx.setLockKeyP(true);
      ctx.setPauseTimer(!ctx.pauseTimer());

      for (const clock of clocks.getTimerClocks()) {
        if (ctx.pauseTimer()) {
          // pause
          if (!clock.pauseStart) {
            clock.pauseStart = ctx.cdateUTC().text();
          }
        } else {
          // re-start
          const pauseDiffMS = ctx.cdateUTC().t - ctx.cdateUTC(clock.pauseStart).t;
          clock.target = ctx.cdateUTC(clock.target).add(pauseDiffMS, "ms").text();
          clock.pauseStart = null;
        }
      }

      ctx.setLockKeyP(false);
      return;
    }
  }

  // send current clocks to clipboard
  if (e.key === "c") {
    e.preventDefault();
    const cls = [];

    for (const clock of clocks.getAllClocks()) {
      if (clock.isEpoch && !ctx.displayEpoch()) {
        continue;
      }
      cls.push(clock.el.parentElement.innerText);
    }

    writeClipboardText(cls.join("  "));
    return;
  }

  // Ctrl + i: Quote clipboard text with double quotes and append comma to the end of each line and open in editor
  // Ctrl + Shift + i: Quote clipboard text with single quotes and append comma to the end of each line and open in editor
  if (e.key === "i" || e.key === "I") {
    e.preventDefault();
    if (e.shiftKey) {
      quoteAndAppendCommaClipboardHandler("'");
    } else {
      quoteAndAppendCommaClipboardHandler('"');
    }
    return;
  }

  // convert between epoch time and date-time to paste from the clipboard
  if (e.key === "v" || e.key === "V") {
    e.preventDefault();
    conversionHandler(e, pressedKeys, clocks, ctx.useTZ(), ctx.convTZ());
  }
}

/**
 * Determines the epoch time unit and multiplier based on keyboard modifiers
 * @param {KeyboardEvent} e - The keyboard event
 * @param {Object} pressedKeys - Object containing pressed key states
 * @returns {{name: string, multiplier: number}} Object with unit name and multiplier to convert to milliseconds
 */
function determineEpochUnit(e, pressedKeys) {
  // KEYs                           convert in
  // Ctrl + v                   --> sec
  // Ctrl + Alt + v             --> millisec
  // Ctrl + Alt + Shift + V     --> microsec
  // Ctrl + Alt + Shift + N + V --> nanosec

  // sec:      -62167219200
  // millisec: -62167219200000
  // microsec: -62167219200000000
  // nanosec:  -62167219200000000000
  // These are converted into "0000-01-01T00:00:00.000+00:00"

  if (pressingAltKey(e) && e.shiftKey && pressedKeys["N"]) {
    return { name: "nanoseconds", multiplier: 1 / 1000 / 1000 };
  } else if (pressingAltKey(e) && e.shiftKey) {
    return { name: "microseconds", multiplier: 1 / 1000 };
  } else if (pressingAltKey(e)) {
    return { name: "milliseconds", multiplier: 1 };
  } else {
    return { name: "seconds", multiplier: 1000 };
  }
}

async function conversionHandler(e, pressedKeys, clocks, usetz, convtz) {
  const origClipboardText = trim(await readClipboardText());
  let src = origClipboardText;
  let isDateTimeText = true;

  const epochUnit = determineEpochUnit(e, pressedKeys);
  const unit = " in " + epochUnit.name;

  if (src.match(/^-?[0-9]+(\.[0-9]+)?$/)) {
    // normalize as millisec
    src = Number(src) * epochUnit.multiplier;
    isDateTimeText = false;
  }

  if (isDateTimeText) {
    src = normalizeDT(src);
    try {
      new Date(src);
    } catch (error) {
      const msg = `Could not convert the clipboard text ${isDateTimeText ? "" : origClipboardText + " "}${unit}.\n\n${error}`;
      await openMessageDialog(msg, "mclocks Error", "error");
      return;
    }
  }

  let cdt;
  if (isDateTimeText && convtz) {
    cdt = cdate().tz(convtz).cdateFn();
  } else {
    cdt = cdate().cdateFn();
  }

  const results = [];

  for (const tz of uniqueTimezones(clocks)) {
    let result;
    try {
      if (usetz) {
        // Use strict timezon conversion by `usetz:true` option in config.
        // For example, before 1888/1/1 00:00:00 in JST, its utcOffset is 09:18, historically.
        result = cdt(src).tz(tz).text()
      } else {
        // Use utcOffset for any date-time.
        const offset = cdt().tz(tz).utcOffset();
        result = cdt(src).utcOffset(offset).text();
      }
      result = `${result} in ${tz}`;
    } catch (error) {
      result = `${error} in ${tz}`;
    }
    results.push(result);
  }

  if (isDateTimeText) {
    results.push(`${cdt(src).t / 1000} Epoch in seconds`);
    results.push(`${cdt(src).t} Epoch in milliseconds`);
  }

  const body = `${origClipboardText}${isDateTimeText ? "" : unit}\n\n${results.join("\n")}\n`;

  await openTextInEditor(body, "open editor");
}

// Some datetime strings that represent common use cases may fail to parse in certain environments,
// so they need to be converted to generally parseable datetime strings.
function normalizeDT(src) {
  let m;

  // BQ datetime format
  if (m = src.match(/^(\d\d\d\d-\d\d-\d\d \d\d:\d\d:\d\d\.\d+) UTC$/)) {
    return m[1] + "Z";
  }

  return src;
}

/**
 * Opens text in editor with error handling
 * @param {string} text - The text to open in editor
 * @param {string} errorContext - Error context message for error dialog (e.g., "open editor")
 * @returns {Promise<void>}
 */
async function openTextInEditor(text, errorContext = "open editor") {
  try {
    await invoke('open_text_in_editor', { text });
  } catch (error) {
    await openMessageDialog(`Failed to ${errorContext}: ${error}`, "mclocks Error", "error");
  }
}

/**
 * Common helper function to process clipboard text, transform it, and open in editor
 * @param {Function} transformFn - Function that takes clipboard text and returns transformed text
 * @param {string} errorContext - Error context message for error dialog (e.g., "open editor")
 * @returns {Promise<void>}
 */
async function processClipboardAndOpenEditor(transformFn, errorContext = "open editor") {
  try {
    const clipboardText = await readClipboardText();
    if (!clipboardText) {
      await openMessageDialog("Clipboard is empty", "mclocks", "info");
      return;
    }

    const transformedText = transformFn(clipboardText);
    await openTextInEditor(transformedText, errorContext);
  } catch (error) {
    await openMessageDialog(`Failed to ${errorContext}: ${error}`, "mclocks Error", "error");
  }
}

/**
 * Handles Ctrl + i / Ctrl + Shift + i: Quotes each line of clipboard text with quotes, appends comma to the end (except the last line), and opens in editor
 * @param {string} quoteChar - The quote character to use (' or ")
 */
async function quoteAndAppendCommaClipboardHandler(quoteChar) {
  await processClipboardAndOpenEditor((clipboardText) => {
    const lines = clipboardText.split(/\r?\n/);

    // Find the index of the last non-empty line
    let lastNonEmptyIndex = -1;
    for (let i = lines.length - 1; i >= 0; i--) {
      if (lines[i].trim() !== '') {
        lastNonEmptyIndex = i;
        break;
      }
    }

    const transformedLines = lines.map((line, index) => {
      if (line.trim() === '') {
        return '';
      }
      const quoted = `${quoteChar}${line.trimStart()}${quoteChar}`;
      return index === lastNonEmptyIndex ? quoted : `${quoted},`;
    });
    return transformedLines.join('\n');
  }, "open editor");
}