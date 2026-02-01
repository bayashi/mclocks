import { invoke } from '@tauri-apps/api/core';
import { writeClipboardText, saveWindowStateSafely, scheduleSaveWindowStateSafely, STICKY_WINDOW_STATE_SAVE_DELAY_MS, ignoreOnMoved, setIgnoreOnMoved } from './util.js';
import { DEFAULT_MAX_DISPLAY_LINES, computeExpandedContentLayout, measureRenderedLineCount } from './sticky-layout.js';
import { StickyWindowAdapter } from './sticky-window-adapter.js';
import { StickyNoteWindowView } from './sticky-window-view.js';
import { applyThemeCssVars } from './theme.js';

export class StickyNoteWindowController {
  constructor(text) {
    this.text = text;
    this.config = {
      font: "Courier, monospace",
      color: "#fff",
      size: 14
    };
    this.isExpanded = false;
    this.originalWidth = 300; // Store original width for restoring when collapsing
    this.windowLabel = null; // Window label for persistence
    this.saveDebounceTimer = null; // Debounce timer for saving state
    this.isInitialLoad = true; // Flag to preserve restored window size on first render

    this.window = new StickyWindowAdapter();
    this.view = null;
  }

  async init() {
    try {
      this.windowLabel = this.window.getLabel();
      // Initially disable resizing (single-line mode)
      await this.window.setResizable(false);
    } catch {
      // Ignore error
    }

    await this.loadConfig();
    try {
      applyThemeCssVars(this.config);
    } catch {
      // Ignore error
    }
    await this.loadSavedState();

    const container = document.querySelector('#sticky-container');
    if (!container) {
      return;
    }

    this.view = new StickyNoteWindowView(container);
    this.view.onToggleExpand(async () => {
      await this.toggleExpand();
    });
    this.view.onCopy(async () => {
      try {
        await writeClipboardText(this.text);
        return true;
      } catch {
        return false;
      }
    });
    this.view.onClose(async () => {
      await this.closeWindow();
    });
    this.view.onTextInput(async (text) => {
      this.text = text;
      await this.saveStickyState();
    });
    this.view.onTextBlur(async (text) => {
      this.text = text;
      await this.saveStickyState();
    });
    this.view.onResizeStart(async () => {
      setIgnoreOnMoved(true);
    });
    this.view.onResizeEnd(async ({ elementWidthPx, elementHeightPx }) => {
      await this.onResizeEnd({ elementWidthPx, elementHeightPx });
    });

    this.view.mount({ text: this.text, config: this.config });
    await this.view.captureBaseSizeAfterRestore({ isExpanded: this.isExpanded }).then(({ baseWidthPx }) => {
      if (baseWidthPx) {
        this.originalWidth = baseWidthPx;
      }
    });
    this.view.ensureFontApplied(this.config.font);

    await this.updateCollapsedState();
    this.isInitialLoad = false;

    this.setupWindowCloseHandler();
    this.setupWindowMovedHandler();
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

  setupWindowMovedHandler() {
    // Listen for window position changes
    (async () => {
      try {
        this.window.onMoved(() => {
          if (ignoreOnMoved()) {
            return;
          }
          scheduleSaveWindowStateSafely(STICKY_WINDOW_STATE_SAVE_DELAY_MS);
        }).catch(() => {
          // Ignore error
        });
      } catch {
        // Ignore error
      }
    })();
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

  async saveStickyState() {
    if (!this.windowLabel) {
      return;
    }

    // Clear existing debounce timer
    if (this.saveDebounceTimer) {
      clearTimeout(this.saveDebounceTimer);
    }

    // Debounce save operation
    this.saveDebounceTimer = setTimeout(async () => {
      try {
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

  async closeWindow() {
    try {
      // Save final window state before closing
      await saveWindowStateSafely();
      // Delete from persistence file before closing
      if (this.windowLabel) {
        await invoke("delete_sticky_note_state", { label: this.windowLabel });
      }
      await this.window.close();
    } catch {
      // Ignore error
    }
  }

  async toggleExpand() {
    this.isExpanded = !this.isExpanded;
    await this.updateCollapsedState();
    await this.saveStickyState();
    scheduleSaveWindowStateSafely(STICKY_WINDOW_STATE_SAVE_DELAY_MS);
  }

  async updateCollapsedState() {
    if (!this.view) {
      return;
    }

    const { fontSizePx, lineHeightPx, headerHeightPx, elementWidthPx } = this.view.getLayoutMetrics();
    const textPaddingPx = 8; // 4px top + 4px bottom from textElement padding

    if (this.isExpanded) {
      const contentWidthPx = Math.max(1, elementWidthPx - 16); // 8px left + 8px right (text padding)
      const totalLines = measureRenderedLineCount({
        text: this.text,
        fontFamily: this.config.font,
        fontSizePx,
        lineHeightPx,
        contentWidthPx
      });
      const { needsScroll, contentHeight } = computeExpandedContentLayout({
        totalLines,
        maxDisplayLines: DEFAULT_MAX_DISPLAY_LINES,
        lineHeightPx,
        textPaddingPx
      });

      this.view.applyExpandedLayout({
        needsScroll,
        contentHeightPx: contentHeight,
        headerHeightPx,
        isInitialLoad: this.isInitialLoad,
        originalWidthPx: this.originalWidth
      });

      // Enable window resizing
      await this.window.setResizable(true);

      // Clear size constraints when expanding
      try {
        await this.window.setMaxSize(null);
        await this.window.setMinSize(null);
      } catch {
        // Ignore error
      }

      if (!this.isInitialLoad) {
        await this.resizeWindowToViewSize();
      }
    } else {
      const singleLineHeightPx = lineHeightPx + textPaddingPx;
      this.view.applyCollapsedLayout({
        singleLineHeightPx,
        headerHeightPx,
        originalWidthPx: this.originalWidth
      });

      // Disable window resizing
      await this.window.setResizable(false);

      if (!this.isInitialLoad) {
        await this.resizeWindowToViewSize();
      }
    }
  }

  async onResizeEnd({ elementWidthPx, elementHeightPx }) {
    let shouldScheduleSave = false;
    try {
      this.originalWidth = elementWidthPx;
      await this.resizeWindowToElementSize({ elementWidthPx, elementHeightPx });
      await this.saveStickyState();
      shouldScheduleSave = true;
    } finally {
      setIgnoreOnMoved(false);
    }
    if (shouldScheduleSave) {
      scheduleSaveWindowStateSafely(STICKY_WINDOW_STATE_SAVE_DELAY_MS);
    }
  }

  async resizeWindowToViewSize() {
    if (!this.view) {
      return;
    }
    const { elementWidthPx, elementHeightPx } = this.view.getElementSizePx();
    await this.resizeWindowToElementSize({ elementWidthPx, elementHeightPx });
  }

  async resizeWindowToElementSize({ elementWidthPx, elementHeightPx }) {
    setIgnoreOnMoved(true);
    try {
      // Add border width (1px top + 1px bottom = 2px) to ensure border is visible
      const borderWidth = 2;
      const newWidth = Math.ceil(elementWidthPx) + borderWidth;
      const newHeight = Math.ceil(elementHeightPx) + borderWidth;

      await this.window.setSize({
        type: 'Logical',
        width: newWidth,
        height: newHeight
      });
    } catch {
      // Ignore error
    } finally {
      setIgnoreOnMoved(false);
    }
  }
}
