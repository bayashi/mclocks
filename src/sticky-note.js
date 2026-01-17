import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { invoke } from '@tauri-apps/api/core';
import { escapeHTML } from './util.js';

/**
 * Creates a sticky note window with the given text and configuration
 * @param {string} text - The text to display in the sticky note
 * @param {Object} cfg - Configuration object with font, size, color
 * @returns {Promise<void>}
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

export async function createStickyNote(text, cfg) {
  const fontSize = typeof cfg.size === 'number' || /^[\d.]+$/.test(cfg.size) ? `${cfg.size}px` : cfg.size;
  const fontSizeNum = parseFloat(fontSize);
  const escapedText = escapeHTML(text);
  const textWithNewlines = escapedText.replace(/\n/g, '<br>');
  const backgroundColor = getBackgroundColor(cfg.color);

  // Calculate window size based on text content
  const lines = text.split('\n');
  const maxLineLength = Math.max(...lines.map(line => line.length), 0);

  const padding = 16;
  const lineHeight = fontSizeNum * 1.5;
  const charWidth = fontSizeNum * 0.6;

  // Calculate width: ensure minimum, cap at maximum
  const minWidth = 250;
  const maxWidth = 600;
  const calculatedWidth = Math.max(minWidth, Math.min(maxLineLength * charWidth + padding, maxWidth));

  // Fixed height: always 6 lines maximum, use scroll for more
  // Add 20px for drag bar at the top
  const visibleLines = 6;
  const dragBarHeight = 20;
  const calculatedHeight = visibleLines * lineHeight + padding + dragBarHeight;

  const windowWidth = Math.round(calculatedWidth);
  const windowHeight = Math.round(calculatedHeight);

  const htmlContent = `<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>mclocks sticky note</title>
  <script type="module">
    import { getCurrentWindow } from '@tauri-apps/api/window';
    document.addEventListener('DOMContentLoaded', () => {
      const contentArea = document.getElementById('content-area');
      const dragBar = document.getElementById('drag-bar');

      if (contentArea) {
        contentArea.addEventListener('mousedown', async () => {
          try {
            const window = getCurrentWindow();
            await window.setAlwaysOnTop(true);
            setTimeout(async () => {
              try {
                await window.setAlwaysOnTop(false);
              } catch (e) {
                // Ignore error
              }
            }, 100);
          } catch (e) {
            // Ignore error
          }
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
      background: transparent;
      -webkit-app-region: drag;
      flex-shrink: 0;
      border-bottom: 1px solid rgba(255, 255, 255, 0.2);
    }
    #content-area {
      flex: 1;
      margin: 0;
      padding: 8px;
      overflow-x: hidden;
      overflow-y: auto;
      box-sizing: border-box;
      white-space: pre-wrap;
      word-wrap: break-word;
      background: ${backgroundColor};
      -webkit-app-region: no-drag;
      -webkit-user-select: text;
      user-select: text;
    }
  </style>
</head>
<body>
  <div id="drag-bar"></div>
  <div id="content-area">${textWithNewlines}</div>
</body>
</html>`;

  const windowId = `sticky_note_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
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

  try {
    const webview = new WebviewWindow(windowId, windowOptions);
    // After a short delay, set alwaysOnTop to false so it can be hidden by other windows
    setTimeout(async () => {
      try {
        await webview.setAlwaysOnTop(false);
      } catch (e) {
        // Ignore error if window is already closed
      }
    }, 500);
  } catch (error) {
    console.error('Failed to create sticky note window:', error);
    throw error;
  }
}
