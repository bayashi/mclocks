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

function trimTrailingEmptyLines(lines) {
  const trimmed = [...lines];
  while (trimmed.length > 0 && trimmed[trimmed.length - 1] === '') {
    trimmed.pop();
  }
  return trimmed;
}

function escapeMarkdownCell(text) {
  return String(text ?? '')
    .replace(/\r?\n/g, '<br>')
    .replace(/\|/g, '\\|');
}

function buildMarkdownTable(header, rows) {
  const headerLine = `| ${header.map(escapeMarkdownCell).join(' | ')} |`;
  const separatorLine = `|${new Array(header.length).fill('---').join('|')}|`;
  const rowLines = rows.map((cells) => `| ${cells.map(escapeMarkdownCell).join(' | ')} |`);
  return [headerLine, separatorLine, ...rowLines].join('\n');
}

function trimTrailingEmptyRows(rows) {
  const trimmed = [...rows];
  while (trimmed.length > 0 && trimmed[trimmed.length - 1].every((cell) => cell === '')) {
    trimmed.pop();
  }
  return trimmed;
}

function parseTsvClipboard(text) {
  const rows = [];
  let currentRow = [];
  let currentCell = '';
  let inQuotes = false;

  for (let i = 0; i < text.length; i++) {
    const ch = text[i];

    if (inQuotes) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          currentCell += '"';
          i++;
        } else {
          inQuotes = false;
        }
      } else {
        currentCell += ch;
      }
      continue;
    }

    if (ch === '"') {
      if (currentCell.length === 0) {
        inQuotes = true;
      } else {
        currentCell += ch;
      }
      continue;
    }

    if (ch === '\t') {
      currentRow.push(currentCell);
      currentCell = '';
      continue;
    }

    if (ch === '\n') {
      currentRow.push(currentCell);
      rows.push(currentRow);
      currentRow = [];
      currentCell = '';
      continue;
    }

    if (ch !== '\r') {
      currentCell += ch;
    }
  }

  currentRow.push(currentCell);
  rows.push(currentRow);
  return trimTrailingEmptyRows(rows);
}

/**
 * Handles Ctrl/Cmd + t: Converts clipboard text into a markdown table and opens in editor
 * - TSV-like clipboard text (e.g. copied from Excel) becomes a multi-column table
 * - Non-TSV clipboard text becomes a single-column table
 */
export async function markdownTableFromClipboardHandler() {
  await processClipboardAndOpenEditor((clipboardText) => {
    const isTabular = clipboardText.includes('\t');
    if (!isTabular) {
      const lines = trimTrailingEmptyLines(clipboardText.split(/\r?\n/));
      if (lines.length === 0) {
        return buildMarkdownTable(['value'], []);
      }
      const rows = lines.map((line) => [line]);
      return buildMarkdownTable(['value'], rows);
    }

    const cells = parseTsvClipboard(clipboardText);
    if (cells.length === 0) {
      return buildMarkdownTable(['value'], []);
    }
    const maxColumns = cells.reduce((max, row) => Math.max(max, row.length), 1);
    const normalized = cells.map((row) => {
      const padded = [...row];
      while (padded.length < maxColumns) {
        padded.push('');
      }
      return padded;
    });
    const header = normalized[0];
    const rows = normalized.slice(1);
    return buildMarkdownTable(header, rows);
  }, "open editor");
}

/**
 * Handles Ctrl/Cmd + Shift + t: Opens a two-column markdown table template in editor
 */
export async function openMarkdownTableTemplateInEditor() {
  const template = buildMarkdownTable(['h1', 'h2', 'h3'], [['v1', 'v2', 'v3'], ['v1', 'v2', 'v3']]);
  await openTextInEditor(template, "open editor");
}

