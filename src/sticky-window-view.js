export class StickyNoteWindowView {
  constructor(container) {
    this.container = container;
    this.element = null;
    this.headerElement = null;
    this.contentElement = null;
    this.expandButton = null;
    this.copyButton = null;
    this.closeButton = null;
    this.textElement = null;
    this.resizeHandle = null;

    this.onToggleExpandHandler = null;
    this.onCopyHandler = null;
    this.onCloseHandler = null;
    this.onTextInputHandler = null;
    this.onTextBlurHandler = null;
    this.onResizeStartHandler = null;
    this.onResizeEndHandler = null;

    this.isResizeEnabled = false;
  }

  mount({ text, config }) {
    if (!this.container) {
      return;
    }

    // Create sticky note container
    this.element = document.createElement('div');
    this.element.className = 'sticky-note';
    this.element.style.position = 'relative';
    this.element.style.left = '0';
    this.element.style.top = '0';
    // Let the window define the width on initial load (plugin restores window size)
    this.element.style.width = '100%';
    this.element.style.height = '100%';
    this.element.style.backgroundColor = 'rgba(0, 0, 0, 0.7)';
    // Use clock color for border with opacity
    const borderColor = this.colorToRgba(config.color, 0.3);
    this.element.style.border = `1px solid ${borderColor}`;
    this.element.style.boxShadow = '2px 2px 5px rgba(0, 0, 0, 0.3)';
    this.element.style.display = 'flex';
    this.element.style.flexDirection = 'column';
    this.element.style.zIndex = '10000';
    this.element.style.cursor = 'default';
    this.element.style.webkitAppRegion = 'no-drag';
    this.element.style.minWidth = '200px';
    this.element.style.minHeight = '50px';
    this.element.style.fontFamily = config.font;
    const isNumericSize = typeof config.size === "number" || /^[\d.]+$/.test(config.size);
    const sizeUnit = isNumericSize ? "px" : "";
    this.element.style.fontSize = `${config.size}${sizeUnit}`;
    this.element.style.color = config.color;

    // Create header
    this.headerElement = document.createElement('div');
    this.headerElement.className = 'sticky-note-header';
    this.headerElement.style.display = 'flex';
    this.headerElement.style.justifyContent = 'space-between';
    this.headerElement.style.alignItems = 'center';
    this.headerElement.style.padding = '4px 8px';
    const headerBgColor = this.colorToRgba(config.color, 0.1);
    this.headerElement.style.backgroundColor = headerBgColor;
    const borderBottomColor = this.colorToRgba(config.color, 0.2);
    this.headerElement.style.borderBottom = `1px solid ${borderBottomColor}`;
    this.headerElement.style.cursor = 'move';
    this.headerElement.style.userSelect = 'none';
    this.headerElement.style.fontFamily = config.font;
    const headerSizeUnit = typeof config.size === "number" || /^[\d.]+$/.test(config.size) ? "px" : "";
    this.headerElement.style.fontSize = `${config.size}${headerSizeUnit}`;
    this.headerElement.style.color = config.color;
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
    this.expandButton.style.fontFamily = config.font;
    this.expandButton.style.color = config.color;
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
    this.copyButton.style.fontFamily = config.font;
    this.copyButton.style.color = config.color;
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
    this.closeButton.style.fontFamily = config.font;
    this.closeButton.style.color = config.color;
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
    this.textElement.textContent = text;
    this.textElement.contentEditable = 'true';
    this.textElement.style.margin = '0';
    this.textElement.style.padding = '4px 8px';
    this.textElement.style.whiteSpace = 'pre-wrap';
    this.textElement.style.wordWrap = 'break-word';
    this.textElement.style.fontFamily = config.font;
    const textSizeUnit = typeof config.size === "number" || /^[\d.]+$/.test(config.size) ? "px" : "";
    this.textElement.style.fontSize = `${config.size}${textSizeUnit}`;
    this.textElement.style.color = config.color;
    this.textElement.style.userSelect = 'text';
    this.textElement.style.lineHeight = '1.2';

    this.contentElement.appendChild(this.textElement);

    // Create resize handle
    this.resizeHandle = document.createElement('div');
    this.resizeHandle.className = 'sticky-note-resize';
    this.resizeHandle.style.position = 'absolute';
    this.resizeHandle.style.bottom = '0';
    this.resizeHandle.style.right = '0';
    this.resizeHandle.style.width = '16px';
    this.resizeHandle.style.height = '16px';
    this.resizeHandle.style.cursor = 'nwse-resize';
    this.resizeHandle.style.backgroundColor = 'transparent';
    this.resizeHandle.style.zIndex = '1001';
    this.resizeHandle.style.webkitAppRegion = 'no-drag';

    this.element.appendChild(this.headerElement);
    this.element.appendChild(this.contentElement);

    this.container.appendChild(this.element);
    this.element.appendChild(this.resizeHandle);

    this.setupDomEventListeners();
  }

  setupDomEventListeners() {
    if (!this.element) {
      return;
    }

    // Expand/collapse button
    if (this.expandButton) {
      this.expandButton.addEventListener('click', async (e) => {
        e.stopPropagation();
        if (this.onToggleExpandHandler) {
          await this.onToggleExpandHandler();
        }
      });
    }

    // Copy button
    if (this.copyButton) {
      this.copyButton.addEventListener('click', async (e) => {
        e.stopPropagation();
        if (!this.onCopyHandler) {
          return;
        }
        try {
          const ok = await this.onCopyHandler();
          if (!ok) {
            return;
          }
          this.showCopyFeedback();
        } catch {
          // Ignore error
        }
      });
    }

    // Close button
    if (this.closeButton) {
      this.closeButton.addEventListener('click', async (e) => {
        e.stopPropagation();
        e.preventDefault();
        if (this.onCloseHandler) {
          await this.onCloseHandler();
        }
      });
    }

    // Header drag (for window dragging)
    if (this.headerElement) {
      this.headerElement.addEventListener('mousedown', async (e) => {
        if (e.target === this.expandButton || e.target === this.copyButton || e.target === this.closeButton) {
          return;
        }
        // Window dragging is handled by webkit-app-region: drag
      });
    }

    // Resize handle
    if (this.resizeHandle) {
      this.resizeHandle.addEventListener('mousedown', async (e) => {
        e.stopPropagation();
        if (!this.isResizeEnabled) {
          e.preventDefault();
          return;
        }
        await this.beginResize(e);
      });
    }

    // Listen for text changes
    if (this.textElement) {
      this.textElement.addEventListener('input', async () => {
        if (this.onTextInputHandler) {
          await this.onTextInputHandler(this.getText());
        }
      });

      // Listen for blur event to save text when editing is finished
      this.textElement.addEventListener('blur', async () => {
        if (this.onTextBlurHandler) {
          await this.onTextBlurHandler(this.getText());
        }
      });
    }
  }

  async beginResize(e) {
    if (!this.element || !this.headerElement || !this.contentElement) {
      return;
    }

    if (this.onResizeStartHandler) {
      try {
        await this.onResizeStartHandler();
      } catch {
        // Ignore error
      }
    }

    const startX = e.clientX;
    const startY = e.clientY;
    const rect = this.element.getBoundingClientRect();
    const startWidth = rect.width;
    const startHeight = rect.height;
    e.preventDefault();

    let lastWidth = startWidth;
    let lastHeight = startHeight;

    const onMouseMove = (ev) => {
      const deltaX = ev.clientX - startX;
      const deltaY = ev.clientY - startY;
      const newWidth = Math.max(200, startWidth + deltaX);
      const newHeight = Math.max(50, startHeight + deltaY);

      lastWidth = newWidth;
      lastHeight = newHeight;

      this.element.style.width = `${newWidth}px`;
      this.element.style.height = `${newHeight}px`;

      const headerHeight = this.headerElement.getBoundingClientRect().height || 30;
      const availableContentHeight = newHeight - headerHeight;
      this.contentElement.style.height = `${availableContentHeight}px`;
      this.contentElement.style.maxHeight = `${availableContentHeight}px`;
      this.contentElement.style.overflowY = 'auto';
    };

    const onMouseUp = async () => {
      document.removeEventListener('mousemove', onMouseMove);
      if (!this.onResizeEndHandler) {
        return;
      }
      try {
        await this.onResizeEndHandler({
          elementWidthPx: lastWidth,
          elementHeightPx: lastHeight
        });
      } catch {
        // Ignore error
      }
    };

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp, { once: true });
  }

  captureBaseSizeAfterRestore({ isExpanded }) {
    return new Promise((resolve) => {
      requestAnimationFrame(() => {
        if (!this.element) {
          resolve({ baseWidthPx: null, baseHeightPx: null });
          return;
        }
        const rect = this.element.getBoundingClientRect();
        const borderWidth = 2;
        const width = Math.max(200, rect.width - borderWidth);
        if (width > 0) {
          this.element.style.width = `${width}px`;
        }
        let height = null;
        if (isExpanded && this.headerElement && this.contentElement) {
          height = Math.max(50, rect.height - borderWidth);
          this.element.style.height = `${height}px`;
          const headerHeight = this.headerElement.getBoundingClientRect().height || 30;
          const availableContentHeight = Math.max(1, height - headerHeight);
          this.contentElement.style.height = `${availableContentHeight}px`;
          this.contentElement.style.maxHeight = `${availableContentHeight}px`;
        }
        resolve({ baseWidthPx: width, baseHeightPx: height });
      });
    });
  }

  ensureFontApplied(expectedFont) {
    if (!this.textElement || !this.element) {
      return;
    }
    const computedFont = window.getComputedStyle(this.textElement).fontFamily;
    if (computedFont !== expectedFont && !computedFont.includes(expectedFont.split(',')[0])) {
      // If font doesn't match, force it
      this.textElement.style.fontFamily = expectedFont;
      this.element.style.fontFamily = expectedFont;
    }
  }

  applyExpandedLayout({ needsScroll, contentHeightPx, headerHeightPx, isInitialLoad, originalWidthPx }) {
    if (!this.element || !this.contentElement || !this.expandButton) {
      return;
    }

    this.contentElement.style.overflowY = needsScroll ? 'auto' : 'hidden';
    this.contentElement.style.flex = 'none';
    this.expandButton.textContent = '▼';

    if (isInitialLoad) {
      const currentContentHeight = parseFloat(this.contentElement.style.height);
      if (currentContentHeight) {
        this.contentElement.style.maxHeight = `${currentContentHeight}px`;
      } else {
        this.contentElement.style.maxHeight = `${contentHeightPx}px`;
      }
    } else {
      this.contentElement.style.maxHeight = `${contentHeightPx}px`;
      this.contentElement.style.height = `${contentHeightPx}px`;
      const totalHeight = headerHeightPx + contentHeightPx;
      this.element.style.height = `${totalHeight}px`;
    }

    if (!this.element.style.width || this.element.style.width === '300px') {
      this.element.style.width = `${originalWidthPx}px`;
    }

    this.setResizeHandleEnabled(true);
  }

  applyCollapsedLayout({ singleLineHeightPx, headerHeightPx, originalWidthPx }) {
    if (!this.element || !this.contentElement || !this.expandButton) {
      return;
    }

    this.contentElement.style.overflowY = 'hidden';
    this.contentElement.style.maxHeight = `${singleLineHeightPx}px`;
    this.contentElement.style.height = `${singleLineHeightPx}px`;
    this.contentElement.style.flex = 'none';
    this.expandButton.textContent = '▶';

    this.element.style.width = `${originalWidthPx}px`;

    const totalHeight = headerHeightPx + singleLineHeightPx;
    this.element.style.height = `${totalHeight}px`;

    this.setResizeHandleEnabled(false);
  }

  setResizeHandleEnabled(enabled) {
    this.isResizeEnabled = enabled;
    if (!this.resizeHandle) {
      return;
    }
    if (enabled) {
      this.resizeHandle.style.display = 'block';
      this.resizeHandle.style.cursor = 'nwse-resize';
      this.resizeHandle.style.pointerEvents = 'auto';
    } else {
      this.resizeHandle.style.display = 'none';
      this.resizeHandle.style.cursor = 'default';
      this.resizeHandle.style.pointerEvents = 'none';
    }
  }

  getText() {
    if (!this.textElement) {
      return '';
    }
    return this.textElement.textContent || '';
  }

  getLayoutMetrics() {
    if (!this.element || !this.headerElement || !this.textElement) {
      return {
        fontSizePx: 14,
        lineHeightPx: 16.8,
        headerHeightPx: 30,
        elementWidthPx: 300
      };
    }

    const computedStyle = window.getComputedStyle(this.textElement);
    const fontSize = parseFloat(computedStyle.fontSize) || 14;
    const lineHeightValue = parseFloat(computedStyle.lineHeight) || fontSize * 1.2;
    const headerHeight = this.headerElement.getBoundingClientRect().height || 30;
    const elementWidth = this.element.getBoundingClientRect().width || parseFloat(this.element.style.width) || 300;

    return {
      fontSizePx: fontSize,
      lineHeightPx: lineHeightValue,
      headerHeightPx: headerHeight,
      elementWidthPx: elementWidth
    };
  }

  getElementSizePx() {
    if (!this.element) {
      return { elementWidthPx: 0, elementHeightPx: 0 };
    }
    const elementWidthPx = parseFloat(this.element.style.width) || this.element.getBoundingClientRect().width;
    const elementHeightPx = parseFloat(this.element.style.height) || this.element.getBoundingClientRect().height;
    return { elementWidthPx, elementHeightPx };
  }

  showCopyFeedback() {
    if (!this.copyButton) {
      return;
    }
    const originalText = this.copyButton.textContent;
    this.copyButton.textContent = '✓';
    setTimeout(() => {
      if (this.copyButton) {
        this.copyButton.textContent = originalText;
      }
    }, 500);
  }

  onToggleExpand(handler) {
    this.onToggleExpandHandler = handler;
  }

  onCopy(handler) {
    this.onCopyHandler = handler;
  }

  onClose(handler) {
    this.onCloseHandler = handler;
  }

  onTextInput(handler) {
    this.onTextInputHandler = handler;
  }

  onTextBlur(handler) {
    this.onTextBlurHandler = handler;
  }

  onResizeStart(handler) {
    this.onResizeStartHandler = handler;
  }

  onResizeEnd(handler) {
    this.onResizeEndHandler = handler;
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
}
