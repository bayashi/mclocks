import { openUrl } from '@tauri-apps/plugin-opener';

import { adjustWindowSize, switchFormat, openToEditConfigFile, toggleEpochTime, addTimerClock } from './clock_matter.js';
import { writeClipboardText, isMacOS, isWindowsOS, openMessageDialog } from './util.js';
import { conversionHandler } from './conversion.js';
import { quoteAndAppendCommaClipboardHandler } from './clipboard.js';
import { createSticky } from './sticky/sticky_manager.js';

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

export async function operationKeysHandler(e, pressedKeys, clockCtx, cfg, clocks) {
  if (isWindowsOS()) {
    if (e.metaKey && e.key === "d") {
      e.preventDefault(); // ignore "Windows + D" to keep displaying mclocks
      return;
    }

    if (e.key === "F1") {
      e.preventDefault();
      await openHelpPage();
      return;
    }
  }

  if (isMacOS()) {
    if (e.metaKey && e.shiftKey && e.key === "/") {
      e.preventDefault();
      await openHelpPage();
      return;
    }
  }

  if (pressingBaseKey(e)) {
    await withBaseKey(e, pressedKeys, clockCtx, cfg, clocks);
  }
}

// operations to be pressed together with Ctrl key
async function withBaseKey(e, pressedKeys, clockCtx, cfg, clocks) {
  // switch date-time format if format2 would be defined
  if (e.key === "f") {
    e.preventDefault();
    switchFormat(clockCtx, cfg, clocks);
    return;
  }

  if (e.key === "o") {
    e.preventDefault();
    openToEditConfigFile(clockCtx);
    return;
  }

  // Ctrl + s: Create a sticky note from clipboard text
  if (e.key === "s" || e.key === "S") {
    e.preventDefault();
    await createSticky();
    return;
  }

  // toggle to display Epoch time
  if (e.key === "e" || e.key === "u") {
    e.preventDefault();
    toggleEpochTime(clockCtx, clocks);
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
      if (clocks.getTimerClocks().length < clockCtx.maxTimerClockNumber()) {
        addTimerClock(clockCtx, cfg, clocks, input * coef);
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
      adjustWindowSize(clockCtx, clocks);
      return;
    }
  }

  // pause and re-start timer clocks (Not able to control each timer clock)
  if (e.key === "p") {
    e.preventDefault();
    if (clocks.getTimerClocks().length > 0) {
      if (clockCtx.lockKeyP()) {
        return;
      }
      clockCtx.setLockKeyP(true);
      clockCtx.setPauseTimer(!clockCtx.pauseTimer());

      for (const clock of clocks.getTimerClocks()) {
        if (clockCtx.pauseTimer()) {
          // pause
          if (!clock.pauseStart) {
            clock.pauseStart = clockCtx.cdateUTC().text();
          }
        } else {
          // re-start
          const pauseDiffMS = clockCtx.cdateUTC().t - clockCtx.cdateUTC(clock.pauseStart).t;
          clock.target = clockCtx.cdateUTC(clock.target).add(pauseDiffMS, "ms").text();
          clock.pauseStart = null;
        }
      }

      clockCtx.setLockKeyP(false);
      return;
    }
  }

  // send current clocks to clipboard
  if (e.key === "c") {
    e.preventDefault();
    const cls = [];

    for (const clock of clocks.getAllClocks()) {
      if (clock.isEpoch && !clockCtx.displayEpoch()) {
        continue;
      }
      cls.push(clock.el.parentElement.innerText);
    }

    await writeClipboardText(cls.join("  "));
    return;
  }

  // Ctrl + i: Quote clipboard text with double quotes and append comma to the end of each line and open in editor
  // Ctrl + Shift + i: Append comma to the end of each line (no quotes) for INT list IN condition and open in editor
  if (e.key === "i" || e.key === "I") {
    e.preventDefault();
    if (e.shiftKey) {
      await quoteAndAppendCommaClipboardHandler("");
    } else {
      await quoteAndAppendCommaClipboardHandler('"');
    }
    return;
  }

  // convert between epoch time and date-time to paste from the clipboard
  if (e.key === "v" || e.key === "V") {
    e.preventDefault();
    await conversionHandler(e, pressedKeys, clocks, clockCtx.useTZ(), clockCtx.convTZ());
  }
}
