import { invoke } from '@tauri-apps/api/core';

import { readClipboardText, openMessageDialog } from '../util.js';

const debugLog = import.meta.env.DEV
	? (...args) => console.info(...args)
	: () => {};

const MAX_TEXT_BYTES = 128 * 1024; // 128KB

function getTextByteSize(text) {
	return new TextEncoder().encode(text).length;
}

export async function createSticky() {
	debugLog('[sticky] createSticky: start');
	let text;
	try {
		debugLog('[sticky] clipboard: read start');
		text = await readClipboardText();
		debugLog('[sticky] clipboard: read ok length=%d', text?.length ?? 0);
	} catch (error) {
		debugLog('[sticky] clipboard: read failed', error);
		try {
			await openMessageDialog(`Failed to read clipboard: ${error}`, "mclocks Error", "error");
		} catch {
			alert(`Failed to read clipboard: ${error}`);
		}
		return;
	}

	if (!text) {
		debugLog('[sticky] clipboard: empty');
		try {
			await openMessageDialog("Clipboard is empty", "mclocks", "info");
		} catch {
			alert("Clipboard is empty");
		}
		return;
	}

	const textBytes = getTextByteSize(text);
	debugLog('[sticky] clipboard: text size=%d bytes', textBytes);
	if (textBytes >= MAX_TEXT_BYTES) {
		debugLog('[sticky] clipboard: text too large');
		const sizeKB = (textBytes / 1024).toFixed(1);
		try {
			await openMessageDialog(`Text is too large (${sizeKB} KB). Maximum size is 128 KB.`, "mclocks Error", "error");
		} catch {
			alert(`Text is too large (${sizeKB} KB). Maximum size is 128 KB.`);
		}
		return;
	}

	try {
		debugLog('[sticky] invoke create_sticky: start');
		const label = await invoke('create_sticky', { text });
		debugLog('[sticky] invoke create_sticky: ok label=%s', label);
	} catch (error) {
		debugLog('[sticky] invoke create_sticky: failed', error);
		try {
			await openMessageDialog(`Failed to create sticky: ${error}`, "mclocks Error", "error");
		} catch {
			alert(`Failed to create sticky: ${error}`);
		}
	}
}
