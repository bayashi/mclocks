export const DEFAULT_MAX_DISPLAY_LINES = 12;

export function measureRenderedLineCount({
	text,
	fontFamily,
	fontSizePx,
	lineHeightPx,
	contentWidthPx
}) {
	const safeLineHeight = Number.isFinite(lineHeightPx) && lineHeightPx > 0 ? lineHeightPx : 16.8;
	const safeFontSize = Number.isFinite(fontSizePx) && fontSizePx > 0 ? fontSizePx : 14;
	const safeWidth = Number.isFinite(contentWidthPx) && contentWidthPx > 0 ? contentWidthPx : 1;

	const probe = document.createElement('pre');
	probe.style.position = 'absolute';
	probe.style.left = '-99999px';
	probe.style.top = '0';
	probe.style.visibility = 'hidden';
	probe.style.pointerEvents = 'none';
	probe.style.margin = '0';
	probe.style.padding = '0';
	probe.style.border = '0';
	probe.style.whiteSpace = 'pre-wrap';
	probe.style.wordWrap = 'break-word';
	probe.style.fontFamily = fontFamily;
	probe.style.fontSize = `${safeFontSize}px`;
	probe.style.lineHeight = `${safeLineHeight}px`;
	probe.style.width = `${safeWidth}px`;
	probe.textContent = text || '';

	document.body.appendChild(probe);
	const height = probe.scrollHeight;
	document.body.removeChild(probe);

	return Math.max(1, Math.ceil(height / safeLineHeight));
}

export function computeExpandedContentLayout({
	totalLines,
	maxDisplayLines = DEFAULT_MAX_DISPLAY_LINES,
	lineHeightPx,
	textPaddingPx
}) {
	const safeTotalLines = Number.isFinite(totalLines) && totalLines > 0 ? Math.floor(totalLines) : 1;
	const safeMaxLines = Number.isFinite(maxDisplayLines) && maxDisplayLines > 0 ? Math.floor(maxDisplayLines) : DEFAULT_MAX_DISPLAY_LINES;
	const safeLineHeight = Number.isFinite(lineHeightPx) && lineHeightPx > 0 ? lineHeightPx : 16.8;
	const safeTextPadding = Number.isFinite(textPaddingPx) && textPaddingPx >= 0 ? textPaddingPx : 0;

	const displayLines = Math.min(safeTotalLines, safeMaxLines);
	const contentHeight = displayLines * safeLineHeight + safeTextPadding;
	const needsScroll = safeTotalLines > displayLines;

	return {
		displayLines,
		contentHeight,
		needsScroll,
		totalLines: safeTotalLines,
		maxDisplayLines: safeMaxLines
	};
}
