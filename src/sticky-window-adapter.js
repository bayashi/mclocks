import { getCurrentWindow } from '@tauri-apps/api/window';

export class StickyWindowAdapter {
  constructor() {
    this.currentWindow = getCurrentWindow();
  }

  getLabel() {
    return this.currentWindow.label;
  }

  async setAlwaysOnTop(alwaysOnTop) {
    try {
      await this.currentWindow.setAlwaysOnTop(alwaysOnTop);
    } catch {
      // Ignore error
    }
  }

  async setResizable(resizable) {
    try {
      await this.currentWindow.setResizable(resizable);
    } catch {
      // Ignore error
    }
  }

  async setMaxSize(size) {
    try {
      await this.currentWindow.setMaxSize(size);
    } catch {
      // Ignore error
    }
  }

  async setMinSize(size) {
    try {
      await this.currentWindow.setMinSize(size);
    } catch {
      // Ignore error
    }
  }

  async setSize(size) {
    try {
      await this.currentWindow.setSize(size);
    } catch {
      // Ignore error
    }
  }

  async close() {
    try {
      await this.currentWindow.close();
    } catch {
      // Ignore error
    }
  }

  onMoved(handler) {
    try {
      return this.currentWindow.onMoved(handler);
    } catch {
      return Promise.resolve(() => {});
    }
  }
}
