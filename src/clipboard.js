import { readClipboardText, openMessageDialog } from './util.js';
import { openTextInEditor } from './editor.js';

/**
 * Common helper function to process clipboard text, transform it, and open in editor
 * @param {Function} transformFn - Function that takes clipboard text and returns transformed text
 * @param {string} errorContext - Error context message for error dialog (e.g., "open editor")
 * @returns {Promise<void>}
 */
export async function processClipboardAndOpenEditor(transformFn, errorContext = "open editor") {
  try {
    const clipboardText = await readClipboardText();
    if (!clipboardText) {
      await openMessageDialog("Clipboard is empty", "mclocks", "info");
      return;
    }

    const transformedText = transformFn(clipboardText);
    await openTextInEditor(transformedText, errorContext);
  } catch (error) {
    await openMessageDialog(`Failed to ${errorContext}: ${error}`, "mclocks Error", "error");
  }
}

/**
 * Handles Ctrl + i / Ctrl + Shift + i: Quotes each line of clipboard text with quotes, appends comma to the end (except the last line), and opens in editor
 * @param {string} quoteChar - The quote character to use (' or ")
 */
export async function quoteAndAppendCommaClipboardHandler(quoteChar) {
  await processClipboardAndOpenEditor((clipboardText) => {
    const lines = clipboardText.split(/\r?\n/);

    // Find the index of the last non-empty line
    let lastNonEmptyIndex = -1;
    for (let i = lines.length - 1; i >= 0; i--) {
      if (lines[i].trim() !== '') {
        lastNonEmptyIndex = i;
        break;
      }
    }

    const transformedLines = lines.map((line, index) => {
      if (line.trim() === '') {
        return '';
      }
      const quoted = `${quoteChar}${line.trimStart()}${quoteChar}`;
      return index === lastNonEmptyIndex ? quoted : `${quoted},`;
    });
    return transformedLines.join('\n');
  }, "open editor");
}

