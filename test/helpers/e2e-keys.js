import { platform } from 'node:process';

// Matches src/keys.js: Win uses Ctrl/Alt; macOS uses Command (Meta) for "base" and Control for "alt".
const isMacHost = platform === 'darwin';

/**
 * Base shortcut (Ctrl+* on Windows/Linux, Cmd+* on macOS in the app).
 * @param {...string} keys
 * @returns {string[]}
 */
export function e2eChordBase(...keys) {
    return isMacHost ? ['Meta', ...keys] : ['Control', ...keys];
}

/**
 * Ctrl+Alt+* on Windows; Cmd+Ctrl+* on macOS (app's pressingAltKey + base).
 * @param {...string} keys
 * @returns {string[]}
 */
export function e2eChordBaseAlt(...keys) {
    return isMacHost ? ['Meta', 'Control', ...keys] : ['Control', 'Alt', ...keys];
}

/**
 * Ctrl+Shift+* on Windows; Cmd+Shift+* on macOS.
 * @param {...string} keys
 * @returns {string[]}
 */
export function e2eChordBaseShift(...keys) {
    return isMacHost ? ['Meta', 'Shift', ...keys] : ['Control', 'Shift', ...keys];
}

/**
 * Ctrl+Alt+Shift+* on Windows; Cmd+Ctrl+Shift+* on macOS.
 * @param {...string} keys
 * @returns {string[]}
 */
export function e2eChordBaseAltShift(...keys) {
    return isMacHost ? ['Meta', 'Control', 'Shift', ...keys] : ['Control', 'Alt', 'Shift', ...keys];
}
