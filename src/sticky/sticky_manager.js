import { invoke } from '@tauri-apps/api/core';
import { readImage } from '@tauri-apps/plugin-clipboard-manager';

import { readClipboardText, openMessageDialog } from '../util.js';

const MAX_TEXT_BYTES = 128 * 1024; // 128KB

function getTextByteSize(text) {
	return new TextEncoder().encode(text).length;
}

/// Convert RGBA pixel data to PNG base64 using Canvas API
function rgbaToPngBase64(rgba, width, height) {
	const canvas = document.createElement('canvas');
	canvas.width = width;
	canvas.height = height;
	const ctx = canvas.getContext('2d');
	const imageData = new ImageData(new Uint8ClampedArray(rgba), width, height);
	ctx.putImageData(imageData, 0, 0);
	// toDataURL returns "data:image/png;base64,..."
	const dataUrl = canvas.toDataURL('image/png');
	const base64 = dataUrl.split(',')[1];
	return base64;
}

async function tryCreateImageSticky() {
	let img;
	try {
		img = await readImage();
	} catch {
		return false;
	}

	if (!img) {
		return false;
	}

	let width, height, rgba;
	try {
		const size = await img.size();
		width = size.width;
		height = size.height;
		rgba = await img.rgba();
	} catch {
		return false;
	}

	if (!width || !height || !rgba || rgba.length === 0) {
		return false;
	}

	const imageBase64 = rgbaToPngBase64(rgba, width, height);
	await invoke('create_sticky_image', { imageBase64 });
	return true;
}

export async function createSticky() {
	// Try text first; if readText throws (e.g. clipboard has image only), fall through to image
	let text = null;
	try {
		text = await readClipboardText();
	} catch {
		// No text available, will try image below
	}

	if (text) {
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
		return;
	}

	// No text in clipboard, try image
	try {
		const created = await tryCreateImageSticky();
		if (created) {
			return;
		}
	} catch (error) {
		try {
			await openMessageDialog(`Failed to create image sticky: ${error}`, "mclocks Error", "error");
		} catch {
			alert(`Failed to create image sticky: ${error}`);
		}
		return;
	}

	try {
		await openMessageDialog("Clipboard is empty", "mclocks", "info");
	} catch {
		alert("Clipboard is empty");
	}
}
