import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { initClocks, refreshClocks, adjustWindowSize, startClocks } from './clock_matter.js';
import { ClockCtx } from './clock_ctx.js';
import { Clocks } from './clocks.js';
import { operationKeysHandler } from './keys.js';
import { stickyEntry } from './sticky/sticky.js';
import { cbhistPanelEntry } from './cbhist.js';

// Application entry point
window.addEventListener("DOMContentLoaded", async () => {
  const mainElement = document.querySelector("#mclocks");

  if (await handleStickyWindow(mainElement)) {
    return;
  }

  if (await handleCbhistPanel(mainElement)) {
    return;
  }

  const clockCtx = new ClockCtx(mainElement);

  await clockGlobalInit(clockCtx);
  await clockMain(clockCtx);

  await restoreStickies();
});

/**
 * Handle sticky note window entry
 * @param {HTMLElement} mainElement - Main application element
 * @returns {Promise<boolean>} true if this window is a sticky note window
 */
const handleCbhistPanel = async (mainElement) => {
  let windowLabel = null;
  try {
    windowLabel = getCurrentWindow().label;
  } catch {
    // windowLabel stays null
  }

  if (windowLabel !== 'cbhist') {
    return false;
  }

  document.documentElement.classList.add('cbhist');

  await cbhistPanelEntry(mainElement);

  return true;
};

const handleStickyWindow = async (mainElement) => {
  let windowLabel = null;
  try {
    windowLabel = getCurrentWindow().label;
  } catch {
    // windowLabel remains null
  }

  if (windowLabel?.startsWith('sticky-')) {
    await stickyEntry(mainElement);
    return true;
  }

  return false;
};

/**
 * Restore persisted sticky notes
 */
const restoreStickies = async () => {
  try {
    await invoke('restore_stickies');
  } catch (error) {
    console.warn('[sticky] Failed to restore stickies:', error);
  }
};

const onDropTempWebRoot = async (payload) => {
  if (!payload || payload.type !== 'drop') {
    return;
  }

  const droppedPaths = payload.paths || [];
  if (droppedPaths.length === 0) {
    return;
  }

  try {
    const openedUrl = await invoke('register_temp_web_root', {
      droppedPath: droppedPaths[0]
    });
    console.info(`[web] Temporary root shared: ${openedUrl}`);
  } catch (error) {
    console.warn('[web] Failed to share dropped directory:', error);
  }
};

const isDraggingPathPayload = (payload) => {
  return Array.isArray(payload?.paths) && payload.paths.length > 0;
};

const updateDropHoverState = (mainElement, payload) => {
  if (!mainElement) {
    return;
  }
  if (!payload) {
    mainElement.classList.remove('is-drop-hover');
    return;
  }
  switch (payload.type) {
    case 'enter':
    case 'over':
      if (isDraggingPathPayload(payload)) {
        mainElement.classList.add('is-drop-hover');
      }
      break;
    case 'leave':
    case 'drop':
      mainElement.classList.remove('is-drop-hover');
      break;
    default:
      // Keep current state for unknown payload types.
      break;
  }
};

/**
 * Initialize global event handlers and window behavior
 * @param {ClockCtx} clockCtx - Application context
 */
const clockGlobalInit = async (clockCtx) => {
  // Window move handler with debouncing (save position after idle)
  // Fallback for testing environment where Tauri APIs are not available
  let currentClockWindow;
  try {
    currentClockWindow = getCurrentWindow();
  } catch (error) {
    // Fallback for testing environment
    console.warn('getCurrentWindow not available, skipping window handlers:', error);
    return;
  }

  try {
    await currentClockWindow.onMoved(() => {
      if (clockCtx.ignoreOnMoved()) {
        return;
      }

      clockCtx.setIgnoreOnMoved(true);
      setTimeout(async () => {
        try {
          await invoke('save_window_state_exclusive');
        } catch (error) {
          console.warn('Err:', error);
        } finally {
          clockCtx.setIgnoreOnMoved(false);
        }
      }, 5000);
    });
  } catch (error) {
    // Ignore error in testing environment
    console.warn('Could not set up window move handler:', error);
  }

  // Enable window dragging on macOS
  clockCtx.mainElement().addEventListener("mousedown", async () => {
    if (clockCtx.isMacOS()) {
      try {
        if (currentClockWindow) {
          await currentClockWindow.startDragging();
        }
      } catch (error) {
        console.warn('Err:', error);
      }
    }
  });

  try {
    await currentClockWindow.onDragDropEvent((event) => {
      updateDropHoverState(clockCtx.mainElement(), event.payload);
      onDropTempWebRoot(event.payload);
    });
  } catch (error) {
    // Ignore error in testing environment
    console.warn('Could not set up drag and drop handler:', error);
  }
};

/**
 * Initialize application configuration from backend
 * @param {ClockCtx} clockCtx - Application context
 * @returns {Promise<Object>} Configuration object
 * @throws {Error} If configuration loading fails
 */
const initClockConfig = async (clockCtx) => {
  try {
    const config = await invoke("load_config", {});

    if (config.forefront) {
      try {
        await getCurrentWindow().setAlwaysOnTop(true);
      } catch (error) {
        // Ignore error in testing environment
        console.warn('Could not set always on top:', error);
      }
    }

    clockCtx.setFormat(config.format);
    clockCtx.setTimerIcon(config.timerIcon);
    clockCtx.setWithoutNotification(config.withoutNotification);
    clockCtx.setMaxTimerClockNumber(config.maxTimerClockNumber);
    clockCtx.setUseTZ(config.usetz);
    clockCtx.setConvTZ(config.convtz);
    clockCtx.setDisableHover(config.disableHover);

    return config;
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to load config: ${reason}`, { cause: error });
  }
};

/**
 * Initialize application styles based on configuration
 * @param {ClockCtx} clockCtx - Application context
 * @param {Object} cfg - Configuration object
 */
const initClockStyles = (clockCtx, cfg) => {
  const appStyle = clockCtx.mainElement().style;

  appStyle.fontFamily = cfg.font;
  appStyle.color = cfg.color;

  const isNumericSize = typeof cfg.size === "number" || /^[\d.]+$/.test(cfg.size);
  const sizeUnit = isNumericSize ? "px" : "";
  appStyle.fontSize = `${cfg.size}${sizeUnit}`;
};

/**
 * Initialize keyboard event handlers
 * @param {ClockCtx} clockCtx - Application context
 * @param {Object} cfg - Configuration object
 * @param {Clocks} clocks - Clocks instance
 * @returns {Object} Cleanup functions
 */
const initKeyboardHandlers = (clockCtx, cfg, clocks) => {
  const pressedKeys = new Set();

  const keydownHandler = async (event) => {
    pressedKeys.add(event.key);
    await operationKeysHandler(event, pressedKeys, clockCtx, cfg, clocks);
  };

  const keyupHandler = (event) => {
    pressedKeys.delete(event.key);
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) {
      pressedKeys.clear();
    }
  };

  window.addEventListener('keydown', keydownHandler);
  window.addEventListener('keyup', keyupHandler);

  return {
    cleanup: () => {
      window.removeEventListener('keydown', keydownHandler);
      window.removeEventListener('keyup', keyupHandler);
    }
  };
};

/**
 * Main application initialization and startup
 * @param {ClockCtx} clockCtx - Application context
 */
const clockMain = async (clockCtx) => {
  try {
    const cfg = await initClockConfig(clockCtx);
    const clocks = new Clocks(cfg.clocks, cfg.epochClockName);
    initClockStyles(clockCtx, cfg);
    initClocks(clockCtx, cfg, clocks);
    refreshClocks(clockCtx, clocks);
    adjustWindowSize(clockCtx, clocks);

    startClocks(clockCtx, clocks);

    const { cleanup } = initKeyboardHandlers(clockCtx, cfg, clocks);

    window.addEventListener('beforeunload', () => {
      cleanup();
    });
  } catch (error) {
    console.error('Err:', error);
    clockCtx.mainElement().textContent = `Err: ${error.message}`;
  }
};
