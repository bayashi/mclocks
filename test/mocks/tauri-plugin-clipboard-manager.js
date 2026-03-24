// Browser Clipboard API shim for WebdriverIO (Vite --mode e2e only; see vite.config.js).

export async function writeText(text) {
  if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  throw new Error('Clipboard API not available');
}

export async function readText() {
  if (typeof navigator !== 'undefined' && navigator.clipboard?.readText) {
    return navigator.clipboard.readText();
  }
  throw new Error('Clipboard API not available');
}

export async function readImage() {
  return null;
}

export async function writeImage() {
  throw new Error('writeImage is not supported in the e2e clipboard mock');
}
