import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { LogicalPosition } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { escapeHTML } from './util.js';

// Store references to all created sticky note windows
const stickyNoteWindows = new Map();

/**
 * Saves sticky notes to file
 * @param {Array<{id: string, text: string}>} notes - Array of sticky notes
 * @returns {Promise<void>}
 */
async function saveStickyNotes(notes) {
  try {
    await invoke('save_sticky_notes', { notes });
  } catch (e) {
    console.error('Failed to save sticky notes:', e);
  }
}

/**
 * Loads sticky notes from file
 * @returns {Promise<Array<{id: string, text: string}>>}
 */
export async function loadStickyNotes() {
  try {
    return await invoke('load_sticky_notes');
  } catch (e) {
    console.error('Failed to load sticky notes:', e);
    return [];
  }
}

/**
 * Removes a sticky note from saved notes
 * @param {string} windowId - The ID of the sticky note to remove
 * @returns {Promise<void>}
 */
async function removeStickyNote(windowId) {
  const allNotes = await loadStickyNotes();
  const filteredNotes = allNotes.filter(note => note.id !== windowId);
  await saveStickyNotes(filteredNotes);
}

/**
 * Adds a sticky note to saved notes
 * @param {string} windowId - The ID of the sticky note
 * @param {string} text - The text content
 * @param {number} x - Optional x position
 * @param {number} y - Optional y position
 * @returns {Promise<void>}
 */
async function addStickyNote(windowId, text, x = null, y = null) {
  const allNotes = await loadStickyNotes();
  // Remove existing note with same ID if any, then add new one
  const filteredNotes = allNotes.filter(note => note.id !== windowId);
  const note = { id: windowId, text };
  if (x !== null && y !== null) {
    note.x = x;
    note.y = y;
  }
  filteredNotes.push(note);
  await saveStickyNotes(filteredNotes);
}

/**
 * Updates sticky note position
 * @param {string} windowId - The ID of the sticky note
 * @param {number} x - x position
 * @param {number} y - y position
 * @returns {Promise<void>}
 */
async function updateStickyNotePosition(windowId, x, y) {
  const allNotes = await loadStickyNotes();
  const note = allNotes.find(n => n.id === windowId);
  if (note) {
    note.x = x;
    note.y = y;
    await saveStickyNotes(allNotes);
  }
}

/**
 * Closes a sticky note window by its label
 * @param {string} windowLabel - The label of the window to close
 * @returns {Promise<void>}
 */
export async function closeStickyNote(windowLabel) {
  const webview = stickyNoteWindows.get(windowLabel);
  if (webview) {
    try {
      await webview.close();
      stickyNoteWindows.delete(windowLabel);
      await removeStickyNote(windowLabel);
    } catch (e) {
      console.error('Failed to close sticky note window:', e);
    }
  }
}

/**
 * Creates a sticky note window with the given text and configuration
 * @param {string} text - The text to display in the sticky note
 * @param {Object} cfg - Configuration object with font, size, color
 * @param {string} windowId - Optional window ID (for restoring from saved notes)
 * @returns {Promise<WebviewWindow>}
 */
function hexToRgb(hex) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : null;
}

function getBackgroundColor(textColor) {
  const rgb = hexToRgb(textColor);
  if (!rgb) {
    // Default: assume light text, use dark background
    return 'rgba(0, 0, 0, 0.7)';
  }

  // Calculate luminance
  const luminance = (0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b) / 255;

  // If text is light (luminance > 0.5), use dark background
  // If text is dark (luminance <= 0.5), use light background
  if (luminance > 0.5) {
    return 'rgba(0, 0, 0, 0.7)';
  } else {
    return 'rgba(255, 255, 255, 0.7)';
  }
}

export async function createStickyNote(text, cfg, windowId = null, x = null, y = null) {
  // Limit text size to 1KB (1024 bytes)
  const MAX_TEXT_SIZE = 1024;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  let truncatedText = text;

  // Calculate byte size and truncate if necessary
  let byteSize = encoder.encode(truncatedText).length;
  if (byteSize > MAX_TEXT_SIZE) {
    // Truncate to fit within 1KB, ensuring we don't break UTF-8 sequences
    const bytes = encoder.encode(truncatedText);
    truncatedText = decoder.decode(bytes.slice(0, MAX_TEXT_SIZE));
  }

  const fontSize = typeof cfg.size === 'number' || /^[\d.]+$/.test(cfg.size) ? `${cfg.size}px` : cfg.size;
  const fontSizeNum = parseFloat(fontSize);
  const escapedText = escapeHTML(truncatedText);
  const backgroundColor = getBackgroundColor(cfg.color);
  // Header background is more opaque (higher alpha value)
  const headerBackgroundColor = backgroundColor.replace(/rgba?\(([^)]+),\s*([\d.]+)\)/, (match, colors, alpha) => {
    const newAlpha = Math.min(parseFloat(alpha) + 0.2, 1.0);
    return `rgba(${colors}, ${newAlpha})`;
  });

  // Calculate window size based on text content
  const lines = truncatedText.split('\n');
  const lineCount = lines.length;
  const maxLineLength = Math.max(...lines.map(line => line.length), 0);

  // For short text (single line or short), don't wrap
  const isShortText = lineCount === 1 && maxLineLength < 50;
  const textWithNewlines = isShortText ? escapedText : escapedText.replace(/\n/g, '<br>');

  const padding = 16;
  const lineHeight = fontSizeNum * 1.5;
  const charWidth = fontSizeNum * 0.6;

  // Calculate width: text-based minimum, cap at maximum
  const minWidth = maxLineLength * charWidth + padding;
  const maxWidth = 600;
  const calculatedWidth = Math.min(minWidth, maxWidth);

  // Dynamic height: minimum 2 lines, maximum 6 lines, use scroll for more
  // Add 20px for drag bar at the top
  const minVisibleLines = 2;
  const maxVisibleLines = 6;
  const visibleLines = Math.max(minVisibleLines, Math.min(lineCount, maxVisibleLines));
  const dragBarHeight = 20;
  const calculatedHeight = visibleLines * lineHeight + padding + dragBarHeight;

  const windowWidth = Math.round(calculatedWidth);
  const windowHeight = Math.round(calculatedHeight);

  const finalWindowId = windowId || `${Date.now()}_${crypto.randomUUID()}`;

  const htmlContent = `<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>mclocks sticky note</title>
  <script>
    document.addEventListener('DOMContentLoaded', async () => {
      const contentArea = document.getElementById('content-area');
      const closeButton = document.getElementById('close-button');
      const windowLabel = '${finalWindowId}';

      // Set up window move handler to save position
      if (window.__TAURI_INTERNALS__) {
        try {
          const { getCurrentWindow } = await import('@tauri-apps/api/window');
          const currentWindow = getCurrentWindow();

          let positionUpdateTimeout = null;
          currentWindow.onMoved(() => {
            // Debounce position updates
            if (positionUpdateTimeout) {
              clearTimeout(positionUpdateTimeout);
            }
            positionUpdateTimeout = setTimeout(async () => {
              try {
                const position = await currentWindow.innerPosition();
                if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.core && window.__TAURI_INTERNALS__.core.invoke) {
                  await window.__TAURI_INTERNALS__.core.invoke('update_sticky_note_position', {
                    windowLabel: windowLabel,
                    x: position.x,
                    y: position.y
                  });
                }
              } catch (e) {
                // Ignore errors
              }
            }, 500);
          });
        } catch (e) {
          // Ignore errors
        }
      }

      if (contentArea) {
        contentArea.addEventListener('mousedown', async () => {
          try {
            if (window.__TAURI_INTERNALS__) {
              const { getCurrentWindow } = await import('@tauri-apps/api/window');
              const window = getCurrentWindow();
              await window.setAlwaysOnTop(true);
              setTimeout(async () => {
                try {
                  await window.setAlwaysOnTop(false);
                } catch (e) {
                  // Ignore error
                }
              }, 100);
            }
          } catch (e) {
            // Ignore error
          }
        });
      }

      if (closeButton) {
        const handleClose = async (e) => {
          e.preventDefault();
          e.stopPropagation();
          try {
            // First try the standard window.close() method
            if (window.close) {
              window.close();
              return;
            }

            // If that doesn't work, try Tauri's internal API
            if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.core) {
              const core = window.__TAURI_INTERNALS__.core;
              if (core.windows && core.windows.getCurrent) {
                try {
                  const currentWindow = await core.windows.getCurrent();
                  if (currentWindow && currentWindow.close) {
                    await currentWindow.close();
                    return;
                  }
                } catch (err) {
                  // Fall through to invoke method
                }
              }
              // Fallback: use invoke with the window label
              if (core.invoke) {
                await core.invoke('close_sticky_note_window', {
                  windowLabel: windowLabel
                });
              }
            }
          } catch (e) {
            console.error('Failed to close window:', e);
          }
        };
        closeButton.addEventListener('click', handleClose);
        closeButton.addEventListener('mousedown', (e) => {
          e.preventDefault();
          e.stopPropagation();
        });
      }
    });
  </script>
  <style>
    ::-webkit-scrollbar {
      width: 12px;
    }
    ::-webkit-scrollbar-track {
      background: rgba(0, 0, 0, 0.1);
    }
    ::-webkit-scrollbar-thumb {
      background: rgba(255, 255, 255, 0.3);
      border-radius: 6px;
    }
    ::-webkit-scrollbar-thumb:hover {
      background: rgba(255, 255, 255, 0.5);
    }
    html, body {
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      background: transparent;
    }
    :root {
      background: transparent;
      -webkit-user-select: none;
      user-select: none;
      font: ${fontSize} ${cfg.font};
      color: ${cfg.color};
    }
    body {
      display: flex;
      flex-direction: column;
      border: 1px solid rgba(255, 255, 255, 0.3);
      box-sizing: border-box;
    }
    #drag-bar {
      height: 20px;
      background: ${headerBackgroundColor};
      -webkit-app-region: drag;
      flex-shrink: 0;
      border-bottom: 1px solid rgba(255, 255, 255, 0.2);
      display: flex;
      align-items: center;
      justify-content: flex-end;
      padding-right: 4px;
    }
    #close-button {
      width: 16px;
      height: 16px;
      background: transparent;
      border: none;
      color: ${cfg.color};
      font-size: ${fontSize};
      line-height: 1;
      cursor: pointer;
      -webkit-app-region: no-drag;
      padding: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      opacity: 0.7;
    }
    #close-button:hover {
      opacity: 1;
    }
    #content-area {
      flex: 1;
      margin: 0;
      padding: 8px;
      overflow-x: hidden;
      overflow-y: auto;
      box-sizing: border-box;
      white-space: ${isShortText ? 'nowrap' : 'pre-wrap'};
      word-wrap: break-word;
      background: ${backgroundColor};
      -webkit-app-region: no-drag;
      -webkit-user-select: text;
      user-select: text;
    }
  </style>
</head>
<body>
  <div id="drag-bar">
    <button id="close-button">×</button>
  </div>
  <div id="content-area">${textWithNewlines}</div>
</body>
</html>`;

  const tempFilePath = await invoke('create_sticky_note_html_file', { htmlContent });
  const fileUrl = `file://${tempFilePath.replace(/\\/g, '/')}`;

  const windowOptions = {
    url: fileUrl,
    title: 'mclocks sticky note',
    width: windowWidth,
    height: windowHeight,
    resizable: false,
    minimizable: false,
    maximizable: false,
    transparent: true,
    decorations: false,
    shadow: false,
    alwaysOnTop: true,
    visible: true,
  };

  // Set position if provided
  if (x !== null && y !== null) {
    windowOptions.x = x;
    windowOptions.y = y;
  }

  const webview = new WebviewWindow(finalWindowId, windowOptions);

  // Store the window reference for potential manual closing
  stickyNoteWindows.set(finalWindowId, webview);

  // If position was set, apply it after window is created
  if (x !== null && y !== null) {
    try {
      await webview.setPosition(new LogicalPosition(x, y));
    } catch (e) {
      // Ignore error if position setting fails
    }
  }

  // Save sticky note to file (only if not restoring)
  if (!windowId) {
    await addStickyNote(finalWindowId, truncatedText);
  }

  // After a short delay, set alwaysOnTop to false so it can be hidden by other windows
  setTimeout(async () => {
    try {
      await webview.setAlwaysOnTop(false);
    } catch (e) {
      // Ignore error if window is already closed
    }
  }, 500);

  return webview;
}
