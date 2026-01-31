import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { writeClipboardText, saveWindowStateSafely, scheduleSaveWindowStateSafely, STICKY_WINDOW_STATE_SAVE_DELAY_MS } from './util.js';

/**
 * Sticky note class for window-based display
 */
class StickyNoteWindow {
  constructor(text) {
    this.text = text;
    this.config = {
      font: "Courier, monospace",
      color: "#fff",
      size: 14
    };
    this.isExpanded = false;
    this.element = null;
    this.headerElement = null;
    this.contentElement = null;
    this.expandButton = null;
    this.copyButton = null;
    this.closeButton = null;
    this.textElement = null;
    this.isDragging = false;
    this.isResizing = false;
    this.dragStartX = 0;
    this.dragStartY = 0;
    this.initialLeft = 0;
    this.initialTop = 0;
    this.resizeStartX = 0;
    this.resizeStartY = 0;
    this.initialWidth = 0;
    this.initialHeight = 0;
    this.originalWidth = 300; // Store original width for restoring when collapsing
    this.windowLabel = null; // Window label for persistence
    this.saveDebounceTimer = null; // Debounce timer for saving state

    // Initialize config first (will be loaded asynchronously)
    this.config = {
      font: "Courier, monospace",
      color: "#fff",
      size: 14
    };
    this.init();
  }

  async init() {
    try {
      const currentWindow = getCurrentWindow();
      this.windowLabel = currentWindow.label;
      await currentWindow.setAlwaysOnTop(true);
      // Initially disable resizing (single-line mode)
      await currentWindow.setResizable(false);
    } catch {
      // Ignore error
    }

    // Load config for font, color, and size
    await this.loadConfig();
    // Load saved state if available
    await this.loadSavedState();
    this.create();
    this.setupEventListeners();
    // Listen for window close event to save state
    this.setupWindowCloseHandler();
  }

  setupWindowCloseHandler() {
    // Save state when window is about to close
    window.addEventListener('beforeunload', async () => {
      try {
        // Clear debounce timer and save immediately
        if (this.saveDebounceTimer) {
          clearTimeout(this.saveDebounceTimer);
          this.saveDebounceTimer = null;
        }
        // Save final window state
        await saveWindowStateSafely();
      } catch {
        // Ignore error
      }
    });
  }

  async loadConfig() {
    try {
      const config = await invoke("load_config", {});
      this.config = {
        font: config.font || "Courier, monospace",
        color: config.color || "#fff",
        size: config.size || 14
      };
    } catch {
      // Use defaults if config loading fails
      this.config = {
        font: "Courier, monospace",
        color: "#fff",
        size: 14
      };
    }
  }

  async loadSavedState() {
    if (!this.windowLabel) {
      return;
    }
    try {
      const savedState = await invoke("load_sticky_note_state", { label: this.windowLabel });
      if (savedState) {
        this.text = savedState.text || this.text;
        this.isExpanded = savedState.isExpanded || false;
      }
    } catch {
      // Ignore error, use defaults
    }
  }

  async saveState() {
    if (!this.windowLabel || !this.element) {
      return;
    }
    // Clear existing debounce timer
    if (this.saveDebounceTimer) {
      clearTimeout(this.saveDebounceTimer);
    }
    // Debounce save operation
    this.saveDebounceTimer = setTimeout(async () => {
      try {
        // Save text and expanded state to custom file
        const state = {
          text: this.text,
          isExpanded: this.isExpanded
        };
        await invoke("save_sticky_note_state", {
          label: this.windowLabel,
          stickyState: state
        });
      } catch {
        // Ignore error
      }
    }, 300); // 300ms debounce
  }

  colorToRgba(color, opacity) {
    // Convert color string to rgba format
    // Supports hex (#fff, #ffffff), rgb(r, g, b), and named colors
    if (color.startsWith('#')) {
      const hex = color.slice(1);
      const r = hex.length === 3 ? parseInt(hex[0] + hex[0], 16) : parseInt(hex.slice(0, 2), 16);
      const g = hex.length === 3 ? parseInt(hex[1] + hex[1], 16) : parseInt(hex.slice(2, 4), 16);
      const b = hex.length === 3 ? parseInt(hex[2] + hex[2], 16) : parseInt(hex.slice(4, 6), 16);
      return `rgba(${r}, ${g}, ${b}, ${opacity})`;
    } else if (color.startsWith('rgb')) {
      const match = color.match(/\d+/g);
      if (match && match.length >= 3) {
        return `rgba(${match[0]}, ${match[1]}, ${match[2]}, ${opacity})`;
      }
    }
    // Fallback: use the color as-is and apply opacity via CSS
    return color;
  }

  create() {
    const container = document.querySelector('#sticky-container');
    if (!container) {
      return;
    }

    // Create sticky note container
    this.element = document.createElement('div');
    this.element.className = 'sticky-note';
    this.element.style.position = 'relative';
    this.element.style.left = '0';
    this.element.style.top = '0';
    this.element.style.width = '300px';
    this.element.style.backgroundColor = 'rgba(0, 0, 0, 0.7)';
    // Use clock color for border with opacity
    const borderColor = this.colorToRgba(this.config.color, 0.3);
    this.element.style.border = `1px solid ${borderColor}`;
    this.element.style.boxShadow = '2px 2px 5px rgba(0, 0, 0, 0.3)';
    this.element.style.display = 'flex';
    this.element.style.flexDirection = 'column';
    this.element.style.zIndex = '10000';
    this.element.style.cursor = 'default';
    this.element.style.webkitAppRegion = 'no-drag';
    this.element.style.minWidth = '200px';
    this.element.style.minHeight = '50px';
    this.element.style.fontFamily = this.config.font;
    const isNumericSize = typeof this.config.size === "number" || /^[\d.]+$/.test(this.config.size);
    const sizeUnit = isNumericSize ? "px" : "";
    this.element.style.fontSize = `${this.config.size}${sizeUnit}`;
    this.element.style.color = this.config.color;

    // Create header
    this.headerElement = document.createElement('div');
    this.headerElement.className = 'sticky-note-header';
    this.headerElement.style.display = 'flex';
    this.headerElement.style.justifyContent = 'space-between';
    this.headerElement.style.alignItems = 'center';
    this.headerElement.style.padding = '4px 8px';
    const headerBgColor = this.colorToRgba(this.config.color, 0.1);
    this.headerElement.style.backgroundColor = headerBgColor;
    const borderBottomColor = this.colorToRgba(this.config.color, 0.2);
    this.headerElement.style.borderBottom = `1px solid ${borderBottomColor}`;
    this.headerElement.style.cursor = 'move';
    this.headerElement.style.userSelect = 'none';
    this.headerElement.style.fontFamily = this.config.font;
    const headerSizeUnit = typeof this.config.size === "number" || /^[\d.]+$/.test(this.config.size) ? "px" : "";
    this.headerElement.style.fontSize = `${this.config.size}${headerSizeUnit}`;
    this.headerElement.style.color = this.config.color;
    this.headerElement.style.webkitAppRegion = 'drag';

    // Create left buttons container
    const leftButtons = document.createElement('div');
    leftButtons.style.display = 'flex';
    leftButtons.style.gap = '4px';
    leftButtons.style.alignItems = 'center';
    leftButtons.style.webkitAppRegion = 'no-drag';
    leftButtons.style.paddingTop = '2px';

    // Create expand/collapse button
    this.expandButton = document.createElement('button');
    this.expandButton.className = 'sticky-note-expand';
    this.expandButton.textContent = '▶';
    this.expandButton.style.border = 'none';
    this.expandButton.style.background = 'transparent';
    this.expandButton.style.cursor = 'pointer';
    this.expandButton.style.fontSize = '14px';
    this.expandButton.style.padding = '0';
    this.expandButton.style.margin = '0';
    this.expandButton.style.marginTop = '2px';
    this.expandButton.style.userSelect = 'none';
    this.expandButton.style.fontFamily = this.config.font;
    this.expandButton.style.color = this.config.color;
    this.expandButton.style.width = '20px';
    this.expandButton.style.height = '20px';
    this.expandButton.style.minWidth = '20px';
    this.expandButton.style.minHeight = '20px';
    this.expandButton.style.display = 'flex';
    this.expandButton.style.alignItems = 'center';
    this.expandButton.style.justifyContent = 'center';
    this.expandButton.style.flexShrink = '0';
    this.expandButton.style.boxSizing = 'border-box';

    // Create copy button
    this.copyButton = document.createElement('button');
    this.copyButton.className = 'sticky-note-copy';
    this.copyButton.textContent = '⧉';
    this.copyButton.style.border = 'none';
    this.copyButton.style.background = 'transparent';
    this.copyButton.style.cursor = 'pointer';
    this.copyButton.style.fontSize = '14px';
    this.copyButton.style.padding = '0';
    this.copyButton.style.margin = '0';
    this.copyButton.style.marginTop = '2px';
    this.copyButton.style.userSelect = 'none';
    this.copyButton.style.fontFamily = this.config.font;
    this.copyButton.style.color = this.config.color;
    this.copyButton.style.width = '20px';
    this.copyButton.style.height = '20px';
    this.copyButton.style.minWidth = '20px';
    this.copyButton.style.minHeight = '20px';
    this.copyButton.style.display = 'flex';
    this.copyButton.style.alignItems = 'center';
    this.copyButton.style.justifyContent = 'center';
    this.copyButton.style.flexShrink = '0';
    this.copyButton.style.boxSizing = 'border-box';

    leftButtons.appendChild(this.expandButton);
    leftButtons.appendChild(this.copyButton);

    // Create close button
    this.closeButton = document.createElement('button');
    this.closeButton.className = 'sticky-note-close';
    this.closeButton.textContent = '×';
    this.closeButton.style.border = 'none';
    this.closeButton.style.background = 'transparent';
    this.closeButton.style.cursor = 'pointer';
    this.closeButton.style.fontSize = '18px';
    this.closeButton.style.padding = '2px 8px';
    this.closeButton.style.margin = '0';
    this.closeButton.style.userSelect = 'none';
    this.closeButton.style.lineHeight = '1';
    this.closeButton.style.fontFamily = this.config.font;
    this.closeButton.style.color = this.config.color;
    this.closeButton.style.webkitAppRegion = 'no-drag';

    this.headerElement.appendChild(leftButtons);
    this.headerElement.appendChild(this.closeButton);

    // Create content area
    this.contentElement = document.createElement('div');
    this.contentElement.className = 'sticky-note-content';
    this.contentElement.style.padding = '0';
    this.contentElement.style.flex = 'none';
    this.contentElement.style.overflow = 'hidden';
    this.contentElement.style.cursor = 'text';
    this.contentElement.style.userSelect = 'text';
    this.contentElement.style.position = 'relative';
    this.contentElement.style.webkitAppRegion = 'no-drag';

    // Create text element (preformatted text, editable)
    this.textElement = document.createElement('pre');
    this.textElement.className = 'sticky-note-text';
    this.textElement.textContent = this.text;
    this.textElement.contentEditable = 'true';
    this.textElement.style.margin = '0';
    this.textElement.style.padding = '4px 8px';
    this.textElement.style.whiteSpace = 'pre-wrap';
    this.textElement.style.wordWrap = 'break-word';
    this.textElement.style.fontFamily = this.config.font;
    const textSizeUnit = typeof this.config.size === "number" || /^[\d.]+$/.test(this.config.size) ? "px" : "";
    this.textElement.style.fontSize = `${this.config.size}${textSizeUnit}`;
    this.textElement.style.color = this.config.color;
    this.textElement.style.userSelect = 'text';
    this.textElement.style.lineHeight = '1.2';

    this.contentElement.appendChild(this.textElement);

    // Create resize handle
    const resizeHandle = document.createElement('div');
    resizeHandle.className = 'sticky-note-resize';
    resizeHandle.style.position = 'absolute';
    resizeHandle.style.bottom = '0';
    resizeHandle.style.right = '0';
    resizeHandle.style.width = '16px';
    resizeHandle.style.height = '16px';
    resizeHandle.style.cursor = 'nwse-resize';
    resizeHandle.style.backgroundColor = 'transparent';
    resizeHandle.style.zIndex = '1001';
    resizeHandle.style.webkitAppRegion = 'no-drag';

    this.element.appendChild(this.headerElement);
    this.element.appendChild(this.contentElement);

    container.appendChild(this.element);
    this.element.appendChild(resizeHandle);

    // Restore saved state if available
    this.restoreSavedState();

    // Set initial collapsed state
    this.updateCollapsedState();

    // Verify font is applied correctly
    const computedFont = window.getComputedStyle(this.textElement).fontFamily;
    const expectedFont = this.config.font;
    if (computedFont !== expectedFont && !computedFont.includes(expectedFont.split(',')[0])) {
      // If font doesn't match, force it
      this.textElement.style.fontFamily = expectedFont;
      this.element.style.fontFamily = expectedFont;
    }
  }

  async restoreSavedState() {
    // Window position and size are automatically restored by window-state plugin
    // No manual restoration needed
  }

  setupEventListeners() {
    // Expand/collapse button
    this.expandButton.addEventListener('click', async (e) => {
      e.stopPropagation();
      await this.toggleExpand();
    });

    // Copy button
    this.copyButton.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await writeClipboardText(this.text);

        // Show checkmark feedback
        const originalText = this.copyButton.textContent;
        this.copyButton.textContent = '✓';

        // Restore original icon after 500ms
        setTimeout(() => {
          this.copyButton.textContent = originalText;
        }, 500);
      } catch {
        // Ignore error
      }
    });

    // Close button
    this.closeButton.addEventListener('click', async (e) => {
      e.stopPropagation();
      e.preventDefault();
      try {
        // Save final window state before closing
        await saveWindowStateSafely();
        // Delete from persistence file before closing
        if (this.windowLabel) {
          await invoke("delete_sticky_note_state", { label: this.windowLabel });
        }
        const currentWindow = getCurrentWindow();
        await currentWindow.close();
      } catch {
        // Ignore error
      }
    });

    // Header drag (for window dragging)
    this.headerElement.addEventListener('mousedown', async (e) => {
      if (e.target === this.expandButton || e.target === this.copyButton || e.target === this.closeButton) {
        return;
      }
      // Window dragging is handled by webkit-app-region: drag
    });

    // Resize handle
    const resizeHandle = this.element.querySelector('.sticky-note-resize');
    if (resizeHandle) {
      resizeHandle.addEventListener('mousedown', (e) => {
        e.stopPropagation();
        // Only allow resizing when expanded
        if (!this.isExpanded) {
          e.preventDefault();
          return;
        }
        this.startResizing(e);
      });
    }

    // Mouse move and up handlers on document
    document.addEventListener('mousemove', (e) => {
      if (this.isResizing) {
        this.resize(e);
      }
    });

    document.addEventListener('mouseup', async () => {
      if (this.isResizing) {
        this.isResizing = false;
        // Save text and expanded state after resize ends
        await this.saveState();
        scheduleSaveWindowStateSafely(STICKY_WINDOW_STATE_SAVE_DELAY_MS);
      }
    });

    // Listen for window position changes
    (async () => {
      try {
        const currentWindow = getCurrentWindow();
        currentWindow.onMoved(async () => {
          // Skip saving during resize to avoid hanging
          if (this.isResizing) {
            return;
          }
          // Save text and expanded state, and schedule window state save
          await this.saveState();
          scheduleSaveWindowStateSafely(STICKY_WINDOW_STATE_SAVE_DELAY_MS);
        }).catch(() => {
          // Ignore error
        });
      } catch {
        // Ignore error
      }
    })();

    // Listen for text changes
    this.textElement.addEventListener('input', async () => {
      this.text = this.textElement.textContent || '';
      await this.saveState();
    });

    // Listen for blur event to save text when editing is finished
    this.textElement.addEventListener('blur', async () => {
      this.text = this.textElement.textContent || '';
      await this.saveState();
    });
  }

  async toggleExpand() {
    this.isExpanded = !this.isExpanded;
    await this.updateCollapsedState();
    await this.saveState();
    scheduleSaveWindowStateSafely(STICKY_WINDOW_STATE_SAVE_DELAY_MS);
  }

  measureRenderedLineCount() {
    const computedStyle = window.getComputedStyle(this.textElement);
    const fontSize = parseFloat(computedStyle.fontSize) || 14;
    const lineHeightValue = parseFloat(computedStyle.lineHeight) || fontSize * 1.2;
    const lineHeight = lineHeightValue;

    const elementWidth = this.element.getBoundingClientRect().width || parseFloat(this.element.style.width) || 300;
    const contentWidth = Math.max(1, elementWidth - 16); // 8px left + 8px right (text padding)

    const probe = document.createElement('pre');
    probe.style.position = 'absolute';
    probe.style.left = '-99999px';
    probe.style.top = '0';
    probe.style.visibility = 'hidden';
    probe.style.pointerEvents = 'none';
    probe.style.margin = '0';
    probe.style.padding = '0';
    probe.style.border = '0';
    probe.style.whiteSpace = 'pre-wrap';
    probe.style.wordWrap = 'break-word';
    probe.style.fontFamily = this.config.font;
    probe.style.fontSize = `${fontSize}px`;
    probe.style.lineHeight = `${lineHeight}px`;
    probe.style.width = `${contentWidth}px`;
    probe.textContent = this.text;

    document.body.appendChild(probe);
    const height = probe.scrollHeight;
    document.body.removeChild(probe);

    return Math.max(1, Math.ceil(height / lineHeight));
  }

  async updateCollapsedState() {
    // Get actual line height from computed style
    const computedStyle = window.getComputedStyle(this.textElement);
    const fontSize = parseFloat(computedStyle.fontSize) || 14;
    const lineHeightValue = parseFloat(computedStyle.lineHeight) || fontSize * 1.2;
    const lineHeight = lineHeightValue;

    const headerHeight = this.headerElement.getBoundingClientRect().height || 30;
    const textPadding = 8; // 4px top + 4px bottom from textElement padding
    const resizeHandle = this.element.querySelector('.sticky-note-resize');

    if (this.isExpanded) {
      const maxDisplayLines = 12;
      const totalLines = this.measureRenderedLineCount();
      const displayLines = Math.min(totalLines, maxDisplayLines);
      const contentHeight = displayLines * lineHeight + textPadding;
      const needsScroll = totalLines > displayLines;

      this.contentElement.style.overflowY = needsScroll ? 'auto' : 'hidden';
      this.contentElement.style.maxHeight = `${contentHeight}px`;
      this.contentElement.style.height = `${contentHeight}px`;
      this.contentElement.style.flex = 'none';
      this.expandButton.textContent = '▼';

      // Set element height
      const totalHeight = headerHeight + contentHeight;
      this.element.style.height = `${totalHeight}px`;

      // If width was changed, keep it; otherwise use original width
      if (!this.element.style.width || this.element.style.width === '300px') {
        this.element.style.width = `${this.originalWidth}px`;
      }

      // Show and enable resize handle
      if (resizeHandle) {
        resizeHandle.style.display = 'block';
        resizeHandle.style.cursor = 'nwse-resize';
        resizeHandle.style.pointerEvents = 'auto';
      }

      // Enable window resizing
      await this.setWindowResizable(true);

      // Clear size constraints when expanding
      try {
        const currentWindow = getCurrentWindow();
        await currentWindow.setMaxSize(null);
        await currentWindow.setMinSize(null);
      } catch {
        // Ignore error
      }

      // Resize window to match content
      this.resizeWindow();
    } else {
      this.contentElement.style.overflowY = 'hidden';
      // Single line height: lineHeight + text padding (4px top + 4px bottom)
      const singleLineHeight = lineHeight + textPadding;
      this.contentElement.style.maxHeight = `${singleLineHeight}px`;
      this.contentElement.style.height = `${singleLineHeight}px`;
      this.contentElement.style.flex = 'none';
      this.expandButton.textContent = '▶';

      // Restore original width when collapsing
      this.element.style.width = `${this.originalWidth}px`;

      // Set element height to single line
      const totalHeight = headerHeight + singleLineHeight;
      this.element.style.height = `${totalHeight}px`;

      // Hide and disable resize handle
      if (resizeHandle) {
        resizeHandle.style.display = 'none';
        resizeHandle.style.cursor = 'default';
        resizeHandle.style.pointerEvents = 'none';
      }

      // Disable window resizing
      this.setWindowResizable(false);

      // Resize window to match content
      this.resizeWindow();
    }
  }

  startResizing(e) {
    // Only allow resizing when expanded
    if (!this.isExpanded) {
      return;
    }

    this.isResizing = true;
    this.resizeStartX = e.clientX;
    this.resizeStartY = e.clientY;
    const rect = this.element.getBoundingClientRect();
    this.initialWidth = rect.width;
    this.initialHeight = rect.height;
    e.preventDefault();
  }

  resize(e) {
    if (!this.isResizing) return;
    // Only allow resizing when expanded
    if (!this.isExpanded) return;

    const deltaX = e.clientX - this.resizeStartX;
    const deltaY = e.clientY - this.resizeStartY;

    const newWidth = Math.max(200, this.initialWidth + deltaX);
    const newHeight = Math.max(50, this.initialHeight + deltaY);

    // Update both width and height when expanded
    this.element.style.width = `${newWidth}px`;
    this.element.style.height = `${newHeight}px`;

    // Update content height to fill available space
    const headerHeight = this.headerElement.getBoundingClientRect().height || 30;
    const textPadding = 8; // 4px top + 4px bottom from textElement padding
    const availableContentHeight = newHeight - headerHeight;
    this.contentElement.style.height = `${availableContentHeight}px`;
    this.contentElement.style.maxHeight = `${availableContentHeight}px`;

    // Recalculate display lines when width changes (text wrapping changes)
    if (this.isExpanded) {
      const computedStyle = window.getComputedStyle(this.textElement);
      const fontSize = parseFloat(computedStyle.fontSize) || 14;
      const lineHeightValue = parseFloat(computedStyle.lineHeight) || fontSize * 1.2;
      const maxDisplayLines = 12;
      const totalLines = this.measureRenderedLineCount();
      const displayLines = Math.min(totalLines, maxDisplayLines);
      const maxDisplayHeight = displayLines * lineHeightValue + textPadding;
      const needsScroll = totalLines > displayLines || availableContentHeight < maxDisplayHeight;
      this.contentElement.style.overflowY = needsScroll ? 'auto' : 'hidden';
    }

    // Resize window to match content (use requestAnimationFrame to ensure DOM is updated)
    requestAnimationFrame(async () => {
      await this.resizeWindow();
      // Don't save window state during resize to avoid hanging
      // It will be saved when resize ends (mouseup event)
    });
  }

  async setWindowResizable(resizable) {
    try {
      const currentWindow = getCurrentWindow();
      await currentWindow.setResizable(resizable);
    } catch {
      // Ignore error
    }
  }

  async resizeWindow() {
    try {
      const currentWindow = getCurrentWindow();

      // Get size from style directly to avoid timing issues
      const elementWidth = parseFloat(this.element.style.width) || this.element.getBoundingClientRect().width;
      const elementHeight = parseFloat(this.element.style.height) || this.element.getBoundingClientRect().height;

      // Add border width (1px top + 1px bottom = 2px) to ensure border is visible
      const borderWidth = 2;
      const newWidth = Math.ceil(elementWidth) + borderWidth;
      const newHeight = Math.ceil(elementHeight) + borderWidth;

      // Remove any size constraints before resizing
      try {
        await currentWindow.setMaxSize(null);
        await currentWindow.setMinSize(null);
      } catch {
        // Ignore error
      }

      await currentWindow.setSize({
        type: 'Logical',
        width: newWidth,
        height: newHeight
      });
    } catch {
      // Ignore error
    }
  }
}

// Initialize sticky note from URL parameter or clipboard
window.addEventListener('DOMContentLoaded', async () => {
  const urlParams = new URLSearchParams(window.location.search);
  const text = urlParams.get('text');

  if (text) {
    const decodedText = decodeURIComponent(text);
    new StickyNoteWindow(decodedText);
  }
});
