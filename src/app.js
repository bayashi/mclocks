import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state';

import { initClocks, adjustWindowSize, startClocks } from './matter.js';
import { Ctx } from './ctx.js';
import { Clocks } from './clocks.js';
import { operationKeysHandler } from './keys.js';
import { getDefaultConfig } from './config.js';

// Application entry point
window.addEventListener("DOMContentLoaded", async () => {
  const mainElement = document.querySelector("#mclocks");
  const ctx = new Ctx(mainElement);

  await globalInit(ctx);
  await main(ctx);
});

/**
 * Initialize global event handlers and window behavior
 * @param {Ctx} ctx - Application context
 */
const globalInit = async (ctx) => {
  // Window move handler with debouncing for non-macOS platforms
  // Fallback for testing environment where Tauri APIs are not available
  let currentWindow;
  try {
    currentWindow = getCurrentWindow();
  } catch (error) {
    // Fallback for testing environment
    console.warn('getCurrentWindow not available, skipping window handlers:', error);
    return;
  }

  try {
    await currentWindow.onMoved(() => {
      // Skip saving window state on macOS due to platform-specific issues
      if (ctx.ignoreOnMoved() || ctx.isMacOS()) {
        return;
      }

      ctx.setIgnoreOnMoved(true);
      setTimeout(async () => {
        try {
          await saveWindowState(StateFlags.ALL);
        } catch (error) {
          console.warn('Err:', error);
        } finally {
          ctx.setIgnoreOnMoved(false);
        }
      }, 5000);
    });
  } catch (error) {
    // Ignore error in testing environment
    console.warn('Could not set up window move handler:', error);
  }

  // Enable window dragging on macOS
  ctx.mainElement().addEventListener("mousedown", async () => {
    if (ctx.isMacOS()) {
      try {
        if (currentWindow) {
          await currentWindow.startDragging();
        }
      } catch (error) {
        console.warn('Err:', error);
      }
    }
  });
};

/**
 * Initialize application configuration from backend
 * @param {Ctx} ctx - Application context
 * @returns {Promise<Object>} Configuration object
 * @throws {Error} If configuration loading fails
 */
const initConfig = async (ctx) => {
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

    ctx.setFormat(config.format);
    ctx.setTimerIcon(config.timerIcon);
    ctx.setWithoutNotification(config.withoutNotification);
    ctx.setMaxTimerClockNumber(config.maxTimerClockNumber);
    ctx.setUseTZ(config.usetz);
    ctx.setConvTZ(config.convtz);
    ctx.setDisableHover(config.disableHover);

    return config;
  } catch (error) {
    // Fallback for testing environment where Tauri APIs are not available
    // Use default configuration
    console.warn('Could not load config from Tauri, using defaults:', error);
    const defaultConfig = getDefaultConfig();

    ctx.setFormat(defaultConfig.format);
    ctx.setTimerIcon(defaultConfig.timerIcon);
    ctx.setWithoutNotification(defaultConfig.withoutNotification);
    ctx.setMaxTimerClockNumber(defaultConfig.maxTimerClockNumber);
    ctx.setUseTZ(defaultConfig.usetz);
    ctx.setConvTZ(defaultConfig.convtz);
    ctx.setDisableHover(defaultConfig.disableHover);

    return defaultConfig;
  }
};

/**
 * Initialize application styles based on configuration
 * @param {Ctx} ctx - Application context
 * @param {Object} cfg - Configuration object
 */
const initStyles = (ctx, cfg) => {
  const appStyle = ctx.mainElement().style;

  appStyle.fontFamily = cfg.font;
  appStyle.color = cfg.color;

  const isNumericSize = typeof cfg.size === "number" || /^[\d.]+$/.test(cfg.size);
  const sizeUnit = isNumericSize ? "px" : "";
  appStyle.fontSize = `${cfg.size}${sizeUnit}`;
};

/**
 * Initialize keyboard event handlers
 * @param {Ctx} ctx - Application context
 * @param {Object} cfg - Configuration object
 * @param {Clocks} clocks - Clocks instance
 * @returns {Object} Cleanup functions
 */
const initKeyboardHandlers = (ctx, cfg, clocks) => {
  const pressedKeys = new Set();

  const keydownHandler = async (event) => {
    pressedKeys.add(event.key);
    await operationKeysHandler(event, pressedKeys, ctx, cfg, clocks);
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
 * @param {Ctx} ctx - Application context
 */
const main = async (ctx) => {
  try {
    const cfg = await initConfig(ctx);
    const clocks = new Clocks(cfg.clocks, cfg.epochClockName);
    initStyles(ctx, cfg);
    initClocks(ctx, cfg, clocks);
    adjustWindowSize(ctx, clocks);

    startClocks(ctx, clocks);

    const { cleanup } = initKeyboardHandlers(ctx, cfg, clocks);

    window.addEventListener('beforeunload', () => {
      cleanup();
    });
  } catch (error) {
    console.error('Err:', error);
    ctx.mainElement().textContent = `Err: ${error.message}`;
  }
};
