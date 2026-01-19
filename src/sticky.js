import { readClipboardText, writeClipboardText } from './util.js';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { invoke } from '@tauri-apps/api/core';

/**
 * Sticky note class
 */
class StickyNote {
  constructor(text, ctx) {
    this.ctx = ctx;
    this.text = text;
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

    this.create();
    this.setupEventListeners();
  }

  create() {
    // Create sticky note container
    this.element = document.createElement('div');
    this.element.className = 'sticky-note';
    this.element.style.position = 'fixed';
    // Position at center of screen initially
    this.element.style.left = '50%';
    this.element.style.top = '50%';
    this.element.style.transform = 'translate(-50%, -50%)';
    this.element.style.width = '300px';
    this.element.style.backgroundColor = 'rgba(0, 0, 0, 0.7)';
    this.element.style.border = '1px solid rgba(255, 255, 255, 0.3)';
    this.element.style.boxShadow = '2px 2px 5px rgba(0, 0, 0, 0.3)';
    this.element.style.display = 'flex';
    this.element.style.flexDirection = 'column';
    this.element.style.zIndex = '10000';
    this.element.style.cursor = 'default';
    this.element.style.webkitAppRegion = 'no-drag';
    this.element.style.minWidth = '200px';
    this.element.style.minHeight = '50px';
    this.element.style.fontFamily = 'Courier, monospace';
    this.element.style.fontSize = '14px';
    this.element.style.color = '#fff';

    // Create header
    this.headerElement = document.createElement('div');
    this.headerElement.className = 'sticky-note-header';
    this.headerElement.style.display = 'flex';
    this.headerElement.style.justifyContent = 'space-between';
    this.headerElement.style.alignItems = 'center';
    this.headerElement.style.padding = '4px 8px';
    this.headerElement.style.backgroundColor = 'rgba(255, 255, 255, 0.1)';
    this.headerElement.style.borderBottom = '1px solid rgba(255, 255, 255, 0.2)';
    this.headerElement.style.cursor = 'move';
    this.headerElement.style.userSelect = 'none';
    this.headerElement.style.fontFamily = 'Courier, monospace';
    this.headerElement.style.fontSize = '14px';
    this.headerElement.style.color = '#fff';

    // Create left buttons container
    const leftButtons = document.createElement('div');
    leftButtons.style.display = 'flex';
    leftButtons.style.gap = '4px';
    leftButtons.style.alignItems = 'center';

    // Create expand/collapse button
    this.expandButton = document.createElement('button');
    this.expandButton.className = 'sticky-note-expand';
    this.expandButton.textContent = '>';
    this.expandButton.style.border = 'none';
    this.expandButton.style.background = 'transparent';
    this.expandButton.style.cursor = 'pointer';
    this.expandButton.style.fontSize = '14px';
    this.expandButton.style.padding = '2px 6px';
    this.expandButton.style.margin = '0';
    this.expandButton.style.userSelect = 'none';
    this.expandButton.style.fontFamily = 'Courier, monospace';
    this.expandButton.style.color = '#fff';

    // Create copy button
    this.copyButton = document.createElement('button');
    this.copyButton.className = 'sticky-note-copy';
    this.copyButton.textContent = 'copy';
    this.copyButton.style.border = 'none';
    this.copyButton.style.background = 'transparent';
    this.copyButton.style.cursor = 'pointer';
    this.copyButton.style.fontSize = '14px';
    this.copyButton.style.padding = '2px 6px';
    this.copyButton.style.margin = '0';
    this.copyButton.style.userSelect = 'none';
    this.copyButton.style.fontFamily = 'Courier, monospace';
    this.copyButton.style.color = '#fff';

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
    this.closeButton.style.fontFamily = 'Courier, monospace';
    this.closeButton.style.color = '#fff';

    this.headerElement.appendChild(leftButtons);
    this.headerElement.appendChild(this.closeButton);

    // Create content area
    this.contentElement = document.createElement('div');
    this.contentElement.className = 'sticky-note-content';
    this.contentElement.style.padding = '8px';
    this.contentElement.style.flex = 'none';
    this.contentElement.style.overflow = 'hidden';
    this.contentElement.style.cursor = 'text';
    this.contentElement.style.userSelect = 'text';
    this.contentElement.style.position = 'relative';

    // Create text element (preformatted text)
    this.textElement = document.createElement('pre');
    this.textElement.className = 'sticky-note-text';
    this.textElement.textContent = this.text;
    this.textElement.style.margin = '0';
    this.textElement.style.whiteSpace = 'pre-wrap';
    this.textElement.style.wordWrap = 'break-word';
    this.textElement.style.fontFamily = 'Courier, monospace';
    this.textElement.style.fontSize = '14px';
    this.textElement.style.color = '#fff';
    this.textElement.style.userSelect = 'text';

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

    this.element.appendChild(this.headerElement);
    this.element.appendChild(this.contentElement);

    // Append to body element instead of mclocks to avoid overlap
    document.body.appendChild(this.element);

    // Append resize handle after element is in DOM
    this.element.appendChild(resizeHandle);

    // Set initial collapsed state
    this.updateCollapsedState();
  }

  setupEventListeners() {
    // Expand/collapse button
    this.expandButton.addEventListener('click', (e) => {
      e.stopPropagation();
      this.toggleExpand();
    });

    // Copy button
    this.copyButton.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await writeClipboardText(this.text);
      } catch (error) {
        console.error('Failed to copy text:', error);
      }
    });

    // Close button
    this.closeButton.addEventListener('click', (e) => {
      e.stopPropagation();
      this.close();
    });

    // Header drag
    this.headerElement.addEventListener('mousedown', (e) => {
      if (e.target === this.expandButton || e.target === this.copyButton || e.target === this.closeButton) {
        return;
      }
      this.startDragging(e);
    });

    // Resize handle
    const resizeHandle = this.element.querySelector('.sticky-note-resize');
    resizeHandle.addEventListener('mousedown', (e) => {
      e.stopPropagation();
      this.startResizing(e);
    });

    // Mouse move and up handlers on document
    document.addEventListener('mousemove', (e) => {
      if (this.isDragging) {
        this.drag(e);
      }
      if (this.isResizing) {
        this.resize(e);
      }
    });

    document.addEventListener('mouseup', () => {
      this.isDragging = false;
      this.isResizing = false;
    });
  }

  toggleExpand() {
    this.isExpanded = !this.isExpanded;
    this.updateCollapsedState();
  }

  updateCollapsedState() {
    if (this.isExpanded) {
      this.contentElement.style.overflowY = 'auto';
      this.contentElement.style.maxHeight = 'none';
      this.contentElement.style.height = 'auto';
      this.contentElement.style.flex = '1';
      this.expandButton.textContent = '⌄';
    } else {
      const lineHeight = parseInt(window.getComputedStyle(this.textElement).lineHeight) || 20;
      this.contentElement.style.overflowY = 'hidden';
      this.contentElement.style.maxHeight = `${lineHeight}px`;
      this.contentElement.style.flex = 'none';
      this.expandButton.textContent = '>';
      // Ensure element has minimum height
      const headerHeight = this.headerElement.getBoundingClientRect().height || 30;
      const contentPadding = 16; // 8px top + 8px bottom
      this.element.style.height = `${headerHeight + lineHeight + contentPadding}px`;
    }
  }

  startDragging(e) {
    this.isDragging = true;
    this.dragStartX = e.clientX;
    this.dragStartY = e.clientY;
    const rect = this.element.getBoundingClientRect();
    this.initialLeft = rect.left;
    this.initialTop = rect.top;
    // Remove transform when dragging starts
    this.element.style.transform = 'none';
    this.element.style.left = `${this.initialLeft}px`;
    this.element.style.top = `${this.initialTop}px`;
    e.preventDefault();
  }

  drag(e) {
    if (!this.isDragging) return;

    const deltaX = e.clientX - this.dragStartX;
    const deltaY = e.clientY - this.dragStartY;

    const newLeft = this.initialLeft + deltaX;
    const newTop = this.initialTop + deltaY;

    this.element.style.left = `${newLeft}px`;
    this.element.style.top = `${newTop}px`;
  }

  startResizing(e) {
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

    const deltaX = e.clientX - this.resizeStartX;
    const deltaY = e.clientY - this.resizeStartY;

    const newWidth = Math.max(200, this.initialWidth + deltaX);
    const newHeight = Math.max(50, this.initialHeight + deltaY);

    this.element.style.width = `${newWidth}px`;
    if (this.isExpanded) {
      this.element.style.height = `${newHeight}px`;
      this.contentElement.style.height = 'auto';
    } else {
      // When collapsed, only resize width, keep height minimal
      const headerHeight = this.headerElement.getBoundingClientRect().height;
      const lineHeight = parseInt(window.getComputedStyle(this.textElement).lineHeight) || 20;
      const contentPadding = 16; // 8px top + 8px bottom
      this.element.style.height = `${headerHeight + lineHeight + contentPadding}px`;
    }
  }

  close() {
    if (this.element && this.element.parentNode) {
      this.element.parentNode.removeChild(this.element);
    }
  }
}

/**
 * Create a new sticky note window from clipboard text
 * @param {Ctx} ctx - Application context
 */
export async function createStickyNoteFromClipboard(ctx) {
  try {
    const text = await readClipboardText();
    if (!text) {
      return;
    }

    // Create unique label for the window
    const label = `sticky-${Date.now()}`;
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

  // Create new webview window
  // Use absolute path based on current window location
  let baseUrl = window.location.origin;
  if (window.location.pathname !== '/' && window.location.pathname !== '/index.html') {
    baseUrl += window.location.pathname.substring(0, window.location.pathname.lastIndexOf('/'));
  }
  const url = `${baseUrl}/sticky.html?text=${encodedText}`;

  try {
    const webview = new WebviewWindow(label, {
      url: url,
      title: 'Sticky Note',
      width: 300,
      height: 100,
      resizable: false, // Initially false (single-line mode), will be enabled when expanded
      minimizable: true,
      maximizable: false,
      transparent: true,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      shadow: false
    });

    webview.once('tauri://created', () => {
      // Window created successfully
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
    const allStates = await invoke("load_all_sticky_note_states", {});
    for (const [label, state] of Object.entries(allStates)) {
      await createStickyNoteWindow(label, state.text);
    }
  } catch (error) {
    // Ignore error
  }
}
