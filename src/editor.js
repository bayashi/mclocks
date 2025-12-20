import { invoke } from '@tauri-apps/api/core';
import { openMessageDialog } from './util.js';

/**
 * Opens text in editor with error handling
 * @param {string} text - The text to open in editor
 * @param {string} errorContext - Error context message for error dialog (e.g., "open editor")
 * @returns {Promise<void>}
 */
export async function openTextInEditor(text, errorContext = "open editor") {
  try {
    await invoke('open_text_in_editor', { text });
  } catch (error) {
    await openMessageDialog(`Failed to ${errorContext}: ${error}`, "mclocks Error", "error");
  }
}

