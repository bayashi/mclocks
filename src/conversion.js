import { cdate } from 'cdate';
import { trim, uniqueTimezones, openMessageDialog, isMacOS, readClipboardText } from './util.js';
import { openTextInEditor } from './editor.js';

// Win   ---> Alt
// macOS ---> Control
const pressingAltKey = (e) => isMacOS() ? e.ctrlKey : e.altKey;

/**
 * Converts a date-time value to a specific timezone
 * @param {Function} cdt - The cdate function instance
 * @param {string|number} src - The source date-time value
 * @param {string} tz - The target timezone
 * @param {boolean} usetz - Whether to use strict timezone conversion
 * @returns {string} The converted date-time string in the format "result in tz" or "error in tz"
 */
function convertToTimezone(cdt, src, tz, usetz) {
  try {
    let result;
    if (usetz) {
      // Use strict timezone conversion by `usetz:true` option in config.
      // For example, before 1888/1/1 00:00:00 in JST, its utcOffset is 09:18, historically.
      result = cdt(src).tz(tz).text();
    } else {
      // Use utcOffset for any date-time.
      const offset = cdt().tz(tz).utcOffset();
      result = cdt(src).utcOffset(offset).text();
    }
    return `${result} in ${tz}`;
  } catch (error) {
    return `${error} in ${tz}`;
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

  if (pressingAltKey(e) && e.shiftKey && pressedKeys.has("N")) {
    return { name: "nanoseconds", multiplier: 1 / 1000 / 1000 };
  } else if (pressingAltKey(e) && e.shiftKey) {
    return { name: "microseconds", multiplier: 1 / 1000 };
  } else if (pressingAltKey(e)) {
    return { name: "milliseconds", multiplier: 1 };
  } else {
    return { name: "seconds", multiplier: 1000 };
  }
}

// Some datetime strings that represent common use cases may fail to parse in certain environments,
// so they need to be converted to generally parseable datetime strings.
function normalizeDT(src) {
  // BQ datetime format
  const m = src.match(/^(\d\d\d\d-\d\d-\d\d \d\d:\d\d:\d\d(?:\.\d+)?) UTC$/);
  if (m) {
    return m[1] + "Z";
  }

  return src;
}

/**
 * Handles conversion between epoch time and date-time from clipboard
 * @param {KeyboardEvent} e - The keyboard event
 * @param {Object} pressedKeys - Object containing pressed key states
 * @param {Object} clocks - The clocks object
 * @param {boolean} usetz - Whether to use strict timezone conversion
 * @param {string} convtz - The timezone for conversion
 */
export async function conversionHandler(e, pressedKeys, clocks, usetz, convtz) {
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
    const result = convertToTimezone(cdt, src, tz, usetz);
    results.push(result);
  }

  if (isDateTimeText) {
    results.push(`${cdt(src).t / 1000} Epoch in seconds`);
    results.push(`${cdt(src).t} Epoch in milliseconds`);
  }

  const body = `${origClipboardText}${isDateTimeText ? "" : unit}\n\n${results.join("\n")}\n`;

  await openTextInEditor(body, "open editor");
}

