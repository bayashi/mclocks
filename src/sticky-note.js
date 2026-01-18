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
    await invoke('debug_log', { message: `saveStickyNotes called with ${notes.length} notes` });
    await invoke('save_sticky_notes', { notes });
    await invoke('debug_log', { message: `saveStickyNotes completed successfully` });
  } catch (e) {
    try {
      await invoke('debug_log', { message: `saveStickyNotes ERROR: ${e.message || e}` });
    } catch {
      // Ignore log errors
    }
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
  try {
    await invoke('debug_log', { message: `removeStickyNote called for windowId: ${windowId}` });
    const allNotes = await loadStickyNotes();
    await invoke('debug_log', { message: `removeStickyNote: loaded ${allNotes.length} notes` });
    const filteredNotes = allNotes.filter(note => note.id !== windowId);
    await invoke('debug_log', { message: `removeStickyNote: filtered to ${filteredNotes.length} notes (removing ${windowId})` });
    await saveStickyNotes(filteredNotes);
    await invoke('debug_log', { message: `removeStickyNote: saved ${filteredNotes.length} notes` });
  } catch (e) {
    try {
      await invoke('debug_log', { message: `removeStickyNote ERROR: ${e.message || e}` });
    } catch {
      // Ignore log errors
    }
    throw e;
  }
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
 * Closes a sticky note window by its label
 * @param {string} windowLabel - The label of the window to close
 * @returns {Promise<void>}
 */
export async function closeStickyNote(windowLabel) {
  const webview = stickyNoteWindows.get(windowLabel);
  if (webview) {
    try {
      // Remove from JSON BEFORE closing window to ensure data is saved
      await removeStickyNote(windowLabel);
      stickyNoteWindows.delete(windowLabel);
      await webview.close();
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
  // Remove trailing empty lines from count
  const lines = truncatedText.split('\n');
  const nonEmptyLines = lines.filter((line, index) => index < lines.length - 1 || line.trim().length > 0);
  const lineCount = nonEmptyLines.length > 0 ? nonEmptyLines.length : 1; // At least 1 line
  const maxLineLength = Math.max(...nonEmptyLines.map(line => line.length), 0);

  // Always preserve line breaks in text
  const textWithNewlines = escapedText;

  const padding = 16;
  const lineHeight = fontSizeNum * 1.5;
  const charWidth = fontSizeNum * 0.6;

  // Calculate width: text-based minimum, cap at maximum
  const minWidth = maxLineLength * charWidth + padding;
  const maxWidth = 600;
  const calculatedWidth = Math.min(minWidth, maxWidth);

  // Window size: initially set to single-line size, can expand to multi-line when toggled
  const dragBarHeight = 20;
  const maxVisibleLines = 8;
  const visibleLines = Math.min(lineCount, maxVisibleLines);
  // contentArea has padding: 8px (top+bottom = 16px), which is included in the element height
  // due to box-sizing: border-box, so we need to add it to window height calculation
  const contentAreaPadding = 16; // top 8px + bottom 8px
  // Single line height for initial display (lineHeight + 8px extra for descenders like p, g, etc.)
  const singleLineHeight = lineHeight + 8;
  const singleLineWindowHeight = singleLineHeight + contentAreaPadding + dragBarHeight;
  const multiLineHeight = visibleLines * lineHeight + contentAreaPadding + dragBarHeight;
  const windowWidth = Math.round(calculatedWidth);
  // Initial window height is single-line size
  const windowHeight = Math.round(singleLineWindowHeight);

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
      const copyButton = document.getElementById('copy-button');
      const collapseButton = document.getElementById('collapse-button');
      const dragBar = document.getElementById('drag-bar');
      const windowLabel = '${finalWindowId}';

      // Window display state: 'single' or 'multi'
      let displayState = 'single';
      const lineCount = ${lineCount};
      const lineHeight = ${lineHeight};
      const padding = ${padding};
      const fontSizeNum = ${fontSizeNum};
      const maxVisibleLines = 8;
      const dragBarHeight = 20;
      // Single line height (lineHeight + 8px extra for descenders like p, g, etc.)
      const singleLineHeight = lineHeight + 8;

      // Get current window using Tauri internal API
      let currentWindow = null;
      try {
        if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.core && window.__TAURI_INTERNALS__.core.windows) {
          currentWindow = await window.__TAURI_INTERNALS__.core.windows.getCurrent();
        }
      } catch (e) {
        // Try standard API as fallback
        try {
          const windowModule = await import('@tauri-apps/api/window');
          currentWindow = windowModule.getCurrentWindow();
        } catch (err) {
          // Ignore errors
        }
      }

      // Set up window move handler to save position
      if (currentWindow) {
        try {

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

      // Toggle between single-line and multi-line display using CSS only
      const toggleDisplayMode = () => {
        if (displayState === 'single') {
          // Switch to multi-line mode
          displayState = 'multi';
          if (collapseButton) collapseButton.textContent = '⌄';
          
          if (contentArea) {
            contentArea.style.webkitAppRegion = 'no-drag';
            contentArea.style.maxHeight = 'none';
            // Show scrollbar if content exceeds visible lines
            if (lineCount > maxVisibleLines) {
              contentArea.style.overflowY = 'auto';
            } else {
              contentArea.style.overflowY = 'hidden';
            }
          }
        } else {
          // Switch to single-line mode
          displayState = 'single';
          if (collapseButton) collapseButton.textContent = '>';
          
          if (contentArea) {
            contentArea.style.webkitAppRegion = 'no-drag';
            contentArea.style.maxHeight = singleLineHeight + 'px';
            contentArea.style.overflowY = 'hidden';
          }
        }
      };

      if (contentArea) {
        // Single click to bring to front (only in multi-line mode)
        contentArea.addEventListener('mousedown', async (e) => {
          if (displayState === 'multi') {
            e.stopPropagation();
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
          }
        });
      }

      if (copyButton && contentArea) {
        const handleCopy = async (e) => {
          e.preventDefault();
          e.stopPropagation();
          try {
            const text = contentArea.textContent || contentArea.innerText || '';
            if (navigator.clipboard && navigator.clipboard.writeText) {
              await navigator.clipboard.writeText(text);
            } else {
              // Fallback for older browsers
              const textArea = document.createElement('textarea');
              textArea.value = text;
              textArea.style.position = 'fixed';
              textArea.style.opacity = '0';
              document.body.appendChild(textArea);
              textArea.select();
              document.execCommand('copy');
              document.body.removeChild(textArea);
            }
            // Change button text to copied and restore after 1 second
            const originalText = copyButton.textContent;
            copyButton.textContent = 'copied';
            setTimeout(() => {
              copyButton.textContent = originalText;
            }, 1000);
          } catch (e) {
            // Ignore errors
          }
        };
        copyButton.addEventListener('click', handleCopy);
        copyButton.addEventListener('mousedown', (e) => {
          e.preventDefault();
          e.stopPropagation();
        });
      }

      if (collapseButton) {
        // Initialize button text based on initial state
        collapseButton.textContent = '>';
        // Ensure initial state is single-line
        if (contentArea) {
          contentArea.style.overflowY = 'hidden';
        }
        collapseButton.addEventListener('click', async (e) => {
          e.preventDefault();
          e.stopPropagation();
          await toggleDisplayMode();
        });
        collapseButton.addEventListener('mousedown', (e) => {
          e.preventDefault();
          e.stopPropagation();
        });
      }

      if (closeButton) {
        const handleClose = async (e) => {
          e.preventDefault();
          e.stopPropagation();
          
          // Try to call close_sticky_note_window command which updates JSON and closes window
          let commandCalled = false;
          
          // Method 1: Try __TAURI_INTERNALS__.core.invoke
          if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.core && window.__TAURI_INTERNALS__.core.invoke) {
            try {
              await window.__TAURI_INTERNALS__.core.invoke('debug_log', {
                message: 'Close button clicked, calling close_sticky_note_window for: ' + windowLabel
              });
              await window.__TAURI_INTERNALS__.core.invoke('close_sticky_note_window', {
                windowLabel: windowLabel
              });
              await window.__TAURI_INTERNALS__.core.invoke('debug_log', {
                message: 'close_sticky_note_window completed for: ' + windowLabel
              });
              commandCalled = true;
              return; // Window should be closed by the command
            } catch (invokeErr) {
              try {
                await window.__TAURI_INTERNALS__.core.invoke('debug_log', {
                  message: 'close_sticky_note_window ERROR for ' + windowLabel + ': ' + (invokeErr.message || invokeErr)
                });
              } catch {
                // Ignore log errors
              }
              // Fall through to window.close()
            }
          }
          
          // Fallback: Just close the window (onCloseRequested should handle JSON update)
          try {
            if (currentWindow && typeof currentWindow.close === 'function') {
              await currentWindow.close();
            } else if (typeof window.close === 'function') {
              window.close();
            }
          } catch (closeErr) {
            // Window close failed
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
      justify-content: space-between;
      padding-left: 4px;
      padding-right: 4px;
    }
    #drag-bar-left {
      display: flex;
      align-items: center;
      gap: 4px;
    }
    #collapse-button {
      background: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.3);
      color: ${cfg.color};
      font-size: ${fontSize * 0.5}px;
      line-height: 1;
      cursor: pointer;
      -webkit-app-region: no-drag;
      padding: 1px 4px;
      border-radius: 2px;
      display: flex;
      align-items: center;
      justify-content: center;
      opacity: 0.8;
    }
    #collapse-button:hover {
      background: rgba(255, 255, 255, 0.2);
      border-color: rgba(255, 255, 255, 0.5);
      opacity: 1;
    }
    #copy-button {
      background: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.3);
      color: ${cfg.color};
      font-size: ${fontSize * 0.5}px;
      line-height: 1;
      cursor: pointer;
      -webkit-app-region: no-drag;
      padding: 1px 4px;
      border-radius: 2px;
      display: flex;
      align-items: center;
      justify-content: center;
      opacity: 0.8;
    }
    #copy-button:hover {
      background: rgba(255, 255, 255, 0.2);
      border-color: rgba(255, 255, 255, 0.5);
      opacity: 1;
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
      overflow-y: hidden;
      box-sizing: border-box;
      white-space: pre-wrap;
      word-wrap: break-word;
      background: ${backgroundColor};
      -webkit-app-region: no-drag;
      -webkit-user-select: text;
      user-select: text;
      max-height: ${singleLineHeight}px;
    }
  </style>
</head>
<body>
  <div id="drag-bar">
    <div id="drag-bar-left">
      <button id="collapse-button">></button>
      <button id="copy-button">copy</button>
    </div>
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
    resizable: true,
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

  // Set up onCloseRequested to update JSON before closing
  // We prevent default close, then handle cleanup on main window side where invoke works
  webview.onCloseRequested(async (event) => {
    await invoke('debug_log', { message: `onCloseRequested fired for: ${finalWindowId}` });
    try {
      // Prevent default close behavior so we can update JSON first
      event.preventDefault();
      
      // Update JSON and close window on main window side where invoke is guaranteed to work
      await invoke('debug_log', { message: `onCloseRequested: calling removeStickyNote for: ${finalWindowId}` });
      await removeStickyNote(finalWindowId);
      stickyNoteWindows.delete(finalWindowId);
      await invoke('debug_log', { message: `onCloseRequested: calling webview.close() for: ${finalWindowId}` });
      
      // Close the window after JSON is updated
      try {
        await webview.close();
      } catch (closeErr) {
        await invoke('debug_log', { message: `onCloseRequested: webview.close() error for ${finalWindowId}: ${closeErr.message || closeErr}` });
      }
    } catch (e) {
      await invoke('debug_log', { message: `onCloseRequested error for ${finalWindowId}: ${e.message || e}` });
      // If update fails, still allow window to close
      stickyNoteWindows.delete(finalWindowId);
      try {
        await webview.close();
      } catch (closeErr) {
        // Ignore if already closed
      }
    }
  });

  // Set up position update handler on main window side
  let positionUpdateTimeout = null;
  webview.onMoved(async () => {
    try {
      // Debounce position updates
      if (positionUpdateTimeout) {
        clearTimeout(positionUpdateTimeout);
      }
      positionUpdateTimeout = setTimeout(async () => {
        try {
          const position = await webview.innerPosition();
          await invoke('update_sticky_note_position', {
            windowLabel: finalWindowId,
            x: position.x,
            y: position.y
          });
        } catch (e) {
          // Ignore errors
        }
      }, 500);
    } catch (e) {
      // Ignore errors
    }
  });

  // If position was set, apply it after window is created
  if (x !== null && y !== null) {
    try {
      await webview.setPosition(new LogicalPosition(x, y));
    } catch {
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
    } catch {
      // Ignore error if window is already closed
    }
  }, 500);

  return webview;
}
