import { StickyNoteWindowController } from './sticky-window-controller.js';

// Initialize sticky note from URL parameter or clipboard
window.addEventListener('DOMContentLoaded', async () => {
  const urlParams = new URLSearchParams(window.location.search);
  const text = urlParams.get('text');

  if (text) {
    const decodedText = decodeURIComponent(text);
    const controller = new StickyNoteWindowController(decodedText);
    await controller.init();
  }
});
