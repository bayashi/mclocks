import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';
import { message } from '@tauri-apps/plugin-dialog';
import { platform } from '@tauri-apps/plugin-os';

const ESCAPE_MAP = new Map([
  ['&', '&amp;'],
  ["'", '&#x27;'],
  ['`', '&#x60;'],
  ['"', '&quot;'],
  ['<', '&lt;'],
  ['>', '&gt;']
]);

/**
 * Escapes HTML special characters in a string
 * @param {string} str - The string to escape
 * @returns {string} The escaped string
 */
export const escapeHTML = (str) => {
  return (str ?? '').replace(/[&'`"<>]/g, (match) => ESCAPE_MAP.get(match));
};

/**
 * Pads a number with leading zero if less than 10
 * @param {number} n - The number to pad
 * @returns {string|number} The padded number or original if >= 10
 */
export const pad = (n) => (n >= 0 && n < 10 ? `0${n}` : n);

/**
 * Trims whitespace and removes line breaks from a string
 * @param {string} str - The string to trim
 * @returns {string} The trimmed string
 */
export const trim = (str) => {
  if (str == null) {
    return '';
  }
  return str.replace(/^[\s\t]+/, '')
    .replace(/[\s\t]+$/, '')
    .replace(/\r?\n/g, '');
};

/**
 * Returns unique timezones from clocks with no duplication
 * @param {Object} clocks - The clocks object with getAllClocks method
 * @returns {string[]} Array of unique timezone strings
 */
export const uniqueTimezones = (clocks) => {
  const timezoneSet = new Set();

  clocks.getAllClocks().forEach((clock) => {
    if (clock.timezone?.length > 0) {
      timezoneSet.add(clock.timezone);
    }
  });

  return Array.from(timezoneSet);
};

/**
 * Checks if notification permission is granted
 * @returns {Promise<boolean>} True if permission is granted
 */
const checkPermission = async () => {
  if (!(await isPermissionGranted())) {
    return (await requestPermission()) === 'granted';
  }
  return true;
};

/**
 * Enqueues a notification if permission is granted
 * @param {string} title - The notification title
 * @param {string} body - The notification body
 * @returns {Promise<void>}
 */
export const enqueueNotification = async (title, body) => {
  if (!(await checkPermission())) {
    return;
  }
  sendNotification({ title, body });
};

/**
 * Writes text to clipboard
 * @param {string} text - The text to write
 * @returns {Promise<void>}
 */
export const writeClipboardText = async (text) => {
  await writeText(text);
};

/**
 * Reads text from clipboard
 * @returns {Promise<string>} The clipboard text
 */
export const readClipboardText = () => {
  return readText();
};

/**
 * Opens a message dialog
 * @param {string} body - The dialog body text
 * @param {string} [title='mclocks'] - The dialog title
 * @param {string} [kind='info'] - The dialog kind
 * @returns {Promise<void>}
 */
export const openMessageDialog = async (body, title = 'mclocks', kind = 'info') => {
  const ret = await message(body, { title, kind });
  return ret;
};

// Platform detection with memoization
// Fallback for testing environment where Tauri APIs are not available
let currentPlatform, isMac, isWin;
try {
    currentPlatform = platform().toLowerCase();
    isMac = currentPlatform === 'macos';
    isWin = currentPlatform === 'windows';
} catch (error) {
    // Fallback for testing environment
    const ua = typeof navigator !== 'undefined' ? navigator.userAgent.toLowerCase() : '';
    currentPlatform = ua.includes('mac') ? 'macos' : ua.includes('win') ? 'windows' : 'linux';
    isMac = currentPlatform === 'macos';
    isWin = currentPlatform === 'windows';
}

/**
 * Checks if the current platform is macOS
 * @returns {boolean} True if running on macOS
 */
export const isMacOS = () => isMac;

/**
 * Checks if the current platform is Windows
 * @returns {boolean} True if running on Windows
 */
export const isWindowsOS = () => isWin;
