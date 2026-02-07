import { invoke } from '@tauri-apps/api/core';

import { readClipboardText, openMessageDialog } from '../util.js';

const MAX_TEXT_BYTES = 128 * 1024; // 128KB

function getTextByteSize(text) {
	return new TextEncoder().encode(text).length;
}

export async function createSticky() {
	let text;
	try {
		text = await readClipboardText();
	} catch (error) {
		try {
			await openMessageDialog(`Failed to read clipboard: ${error}`, "mclocks Error", "error");
		} catch {
			alert(`Failed to read clipboard: ${error}`);
		}
		return;
	}

	if (!text) {
		try {
			await openMessageDialog("Clipboard is empty", "mclocks", "info");
		} catch {
			alert("Clipboard is empty");
		}
		return;
	}

	const textBytes = getTextByteSize(text);
	if (textBytes >= MAX_TEXT_BYTES) {
		const sizeKB = (textBytes / 1024).toFixed(1);
		try {
			await openMessageDialog(`Text is too large (${sizeKB} KB). Maximum size is 128 KB.`, "mclocks Error", "error");
		} catch {
			alert(`Text is too large (${sizeKB} KB). Maximum size is 128 KB.`);
		}
		return;
	}

	try {
		await invoke('create_sticky', { text });
	} catch (error) {
		try {
			await openMessageDialog(`Failed to create sticky: ${error}`, "mclocks Error", "error");
		} catch {
			alert(`Failed to create sticky: ${error}`);
		}
	}
}
