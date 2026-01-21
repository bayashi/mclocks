import { readClipboardText, writeClipboardText, isWindowsOS } from './util.js';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { invoke } from '@tauri-apps/api/core';

/**
 * Create a sticky note from clipboard text
 * @param {Object} ctx - Application context
 */
export async function createStickyNoteFromClipboard(ctx) {
  try {
    const text = await readClipboardText();
    if (!text || text.trim().length === 0) {
      return;
    }

    // Create unique label for the window using UUID v4
    const label = `sticky-${crypto.randomUUID()}`;
    await createStickyNoteWindow(label, text);
  } catch (error) {
    // Ignore error
  }
}

/**
 * Create a sticky note window with specified label and text
 * @param {string} label - Window label
 * @param {string} text - Note text
 */
async function createStickyNoteWindow(label, text) {
  const encodedText = encodeURIComponent(text);

  // Load saved window state and sticky note state
  let savedWindowState = null;
  let savedStickyState = null;
  try {
    savedWindowState = await invoke("load_sticky_note_window_state", { label });
    savedStickyState = await invoke("load_sticky_note_state", { label });
  } catch (error) {
    // Ignore error, use defaults
  }

  // Determine if window should be expanded
  const isExpanded = savedStickyState?.isExpanded || false;

  // Use saved size if expanded, otherwise use defaults
  const width = (isExpanded && savedWindowState?.width) ? savedWindowState.width : 300;
  const height = (isExpanded && savedWindowState?.height) ? savedWindowState.height : 100;
  const x = savedWindowState?.x;
  const y = savedWindowState?.y;

  // Create new webview window
  // Use absolute path based on current window location
  let baseUrl = window.location.origin;
  if (window.location.pathname !== '/' && window.location.pathname !== '/index.html') {
    baseUrl += window.location.pathname.substring(0, window.location.pathname.lastIndexOf('/'));
  }
  const url = `${baseUrl}/sticky.html?text=${encodedText}`;

  try {
    // Use mocked WebviewWindow for testing if available, otherwise use the imported one
    const WebviewWindowClass = (window.__TAURI_INTERNALS__?.WebviewWindow) || WebviewWindow;
    const webview = new WebviewWindowClass(label, {
      url: url,
      title: 'Sticky Note',
      width: width,
      height: height,
      x: x,
      y: y,
      resizable: false, // Initially false (single-line mode), will be enabled when expanded
      minimizable: true,
      maximizable: false,
      transparent: true,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      shadow: false
    });

    webview.once('tauri://created', async () => {
      // Window created successfully
      // Apply saved window state if available
      // Wait a bit for window to be fully initialized
      await new Promise(resolve => setTimeout(resolve, 200));
      if (savedWindowState) {
        try {
          // Apply size first, then position (this helps with multi-display setups)
          // Only restore size if window is expanded
          if (isExpanded && savedWindowState.width !== null && savedWindowState.height !== null) {
            await webview.setSize({ type: 'Logical', width: savedWindowState.width, height: savedWindowState.height });
            // Wait a bit after setting size
            await new Promise(resolve => setTimeout(resolve, 50));
          }
          // Apply position after size to ensure proper placement
          if (savedWindowState.x !== null && savedWindowState.y !== null) {
            // Use correct coordinate system based on platform
            // Windows: Physical coordinates, macOS: Logical coordinates
            if (isWindowsOS()) {
              await webview.setPosition({ type: 'Physical', x: Math.round(savedWindowState.x), y: Math.round(savedWindowState.y) });
            } else {
              await webview.setPosition({ type: 'Logical', x: savedWindowState.x, y: savedWindowState.y });
            }
          }
        } catch (error) {
          // Ignore error
        }
      }
    });

    webview.once('tauri://error', (e) => {
      // Ignore error
    });
  } catch (createError) {
    // Ignore error
  }
}

/**
 * Restore all saved sticky note windows on app startup
 */
export async function restoreStickyNotes() {
  try {
    // Clean up window-state file for deleted sticky notes
    await invoke("cleanup_window_state", {});

    // Restore saved sticky notes
    const allStates = await invoke("load_all_sticky_note_states", {});
    for (const [label, state] of Object.entries(allStates)) {
      await createStickyNoteWindow(label, state.text);
    }
  } catch (error) {
    // Ignore error
  }
}
