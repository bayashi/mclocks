(async function(){
	const normalizeOpenExternalLinkInNewTab = (value) => value !== "false";

	const isExternalHref = (href) => {
		try {
			const resolved = new URL(href, window.location.href);
			if (resolved.protocol === "http:" || resolved.protocol === "https:") {
				return resolved.origin !== window.location.origin;
			}
			return true;
		} catch (_) {
			return false;
		}
	};

	const applyContentLinkTargets = (openExternalLinkInNewTab) => {
		document.querySelectorAll("#content a[href]").forEach((anchor) => {
			const href = (anchor.getAttribute("href") || "").trim();
			if (!href || href.startsWith("#") || !openExternalLinkInNewTab || !isExternalHref(href)) {
				anchor.removeAttribute("target");
				anchor.removeAttribute("rel");
				return;
			}
			anchor.setAttribute("target", "_blank");
			anchor.setAttribute("rel", "noopener noreferrer");
		});
	};

	const openExternalLinkInNewTab = normalizeOpenExternalLinkInNewTab(document.body?.dataset?.openExternalLinkInNewTab);
	applyContentLinkTargets(openExternalLinkInNewTab);

	if (typeof window.mclocksSetupResizer === "function") {
		window.mclocksSetupResizer("mclocks-md-toc-width", "toc-resizer", 200, 400, "--toc-width");
	}

	const copyIconHtml =
		'<svg class="copy-btn-svg" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">' +
		'<rect x="10" y="10" width="10" height="10" rx="1.85" ry="1.85" stroke="currentColor" stroke-width="1.65"/>' +
		'<rect x="4" y="4" width="10" height="10" rx="1.85" ry="1.85" stroke="currentColor" stroke-width="1.65"/>' +
		"</svg>";

	const copiedIconHtml =
		'<svg class="copy-btn-svg" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">' +
		'<path d="M7.05 13.42L10.2 16.62L17.92 8.92" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"/>' +
		"</svg>";

	const normalizeNewlines = (s) => s.replace(/\r\n/g, "\n").replace(/\r/g, "\n");

	const DATETIME_OFFSET_UNITS = new Set(["Y", "M", "D", "H", "m", "s"]);

	const CODE_DATETIME_PLACEHOLDER_BASES = [
		["YYYYMMDD HH:mm:ss", "%Y%m%d %H:%M:%S"],
		["YYYYMMDDHHmmss", "%Y%m%d%H%M%S"],
		["YYYY-MM-DD", "%Y-%m-%d"],
		["YYYYMMDD", "%Y%m%d"],
		["HH:mm:ss", "%H:%M:%S"],
		["HHmmss", "%H%M%S"],
		["YYYY", "%Y"],
		["MM", "%m"],
		["DD", "%d"],
		["HH", "%H"],
		["mm", "%M"],
		["ss", "%S"],
	];

	const pad2 = (n) => String(n).padStart(2, "0");

	const formatLocalDateTime = (dt, fmt) => {
		const Y = dt.getFullYear();
		const m = dt.getMonth() + 1;
		const d = dt.getDate();
		const H = dt.getHours();
		const M = dt.getMinutes();
		const S = dt.getSeconds();
		return fmt
			.replace(/%Y/g, String(Y))
			.replace(/%m/g, pad2(m))
			.replace(/%d/g, pad2(d))
			.replace(/%H/g, pad2(H))
			.replace(/%M/g, pad2(M))
			.replace(/%S/g, pad2(S));
	};

	const addCalendarMonths = (d0, deltaMonths) => {
		const d = new Date(d0.getTime());
		const day = d.getDate();
		d.setMonth(d.getMonth() + deltaMonths);
		if (d.getDate() !== day) {
			d.setDate(0);
		}
		return d;
	};

	const shiftDateTimeByOffset = (rest, d0) => {
		if (rest.length < 3) {
			return null;
		}
		const sign = rest[0];
		if (sign !== "+" && sign !== "-") {
			return null;
		}
		const unit = rest[rest.length - 1];
		if (!DATETIME_OFFSET_UNITS.has(unit)) {
			return null;
		}
		const numPart = rest.slice(1, -1);
		if (!/^\d+$/.test(numPart)) {
			return null;
		}
		const value = parseInt(numPart, 10);
		if (value <= 0) {
			return null;
		}
		const signed = sign === "-" ? -value : value;
		const d = new Date(d0.getTime());
		if (unit === "Y") {
			return addCalendarMonths(d, signed * 12);
		}
		if (unit === "M") {
			return addCalendarMonths(d, signed);
		}
		if (unit === "D") {
			d.setDate(d.getDate() + signed);
			return d;
		}
		if (unit === "H") {
			d.setHours(d.getHours() + signed);
			return d;
		}
		if (unit === "m") {
			d.setMinutes(d.getMinutes() + signed);
			return d;
		}
		if (unit === "s") {
			d.setSeconds(d.getSeconds() + signed);
			return d;
		}
		return null;
	};

	const tryExpandDatetimePlaceholderInner = (inner, now) => {
		const d0 = new Date(now.getTime());
		for (const [base, fmt] of CODE_DATETIME_PLACEHOLDER_BASES) {
			if (!inner.startsWith(base)) {
				continue;
			}
			const tail = inner.slice(base.length);
			let shifted;
			if (tail === "") {
				shifted = d0;
			} else {
				const s = shiftDateTimeByOffset(tail, d0);
				if (!s) {
					return null;
				}
				shifted = s;
			}
			return formatLocalDateTime(shifted, fmt);
		}
		return null;
	};

	const extractAllPlaceholderFieldsInOrder = (text) => {
		const now = new Date();
		const rows = [];
		const seen = new Set();
		let i = 0;
		while (i < text.length) {
			const a = text.indexOf("{{", i);
			if (a < 0) {
				break;
			}
			const b = text.indexOf("}}", a + 2);
			if (b < 0) {
				break;
			}
			const full = text.slice(a, b + 2);
			const inner = text.slice(a + 2, b);
			if (!seen.has(full)) {
				const um = full.match(/^\{\{_([A-Za-z0-9_]+)_\}\}$/);
				if (um) {
					seen.add(full);
					rows.push({ kind: "user", full, shortId: um[1] });
				} else {
					const def = tryExpandDatetimePlaceholderInner(inner, now);
					if (def !== null) {
						seen.add(full);
						rows.push({ kind: "datetime", full, inner, defaultVal: def });
					}
				}
			}
			i = b + 2;
		}
		return rows;
	};

	const tryHljsHighlightNonMermaid = (codeEl) => {
		if (!window.hljs || typeof window.hljs.highlightElement !== "function") {
			return;
		}
		if ((codeEl.className || "").split(/\s+/).includes("language-mermaid")) {
			return;
		}
		window.hljs.highlightElement(codeEl);
	};

	const applyPlaceholdersFromFields = (original, fieldsRoot) => {
		let out = original;
		for (const input of fieldsRoot.querySelectorAll("input[data-placeholder-literal]")) {
			const literal = input.getAttribute("data-placeholder-literal");
			if (!literal) {
				continue;
			}
			const kind = input.getAttribute("data-placeholder-kind");
			let rep;
			if (kind === "datetime") {
				const v = String(input.value || "").trim();
				const def = input.getAttribute("data-default-replacement") || "";
				rep = v !== "" ? v : def !== "" ? def : literal;
			} else {
				const v = String(input.value || "").trim();
				rep = v !== "" ? v : literal;
			}
			out = out.split(literal).join(rep);
		}
		return out;
	};

	const copyPlainTextViaExecCommand = (text) => {
		if (!text || typeof document.execCommand !== "function") {
			return false;
		}
		let ta = null;
		try {
			ta = document.createElement("textarea");
			ta.value = text;
			ta.setAttribute("aria-hidden", "true");
			ta.style.position = "fixed";
			ta.style.top = "-1000px";
			ta.style.left = "0";
			ta.style.opacity = "0";
			ta.style.pointerEvents = "none";
			document.body.appendChild(ta);
			ta.focus();
			ta.select();
			ta.setSelectionRange(0, text.length);
			return document.execCommand("copy") === true;
		} catch (_) {
			return false;
		} finally {
			if (ta && ta.parentNode) {
				ta.parentNode.removeChild(ta);
			}
		}
	};

	const attachCodeBlockCopyUi = (pre, code) => {
		if (!pre || !pre.parentNode || pre.parentElement?.classList.contains("code-block-wrap")) {
			return;
		}
		const originalForCopy = normalizeNewlines(code.textContent || "");
		const placeholderRows = extractAllPlaceholderFieldsInOrder(originalForCopy);
		let fieldsEl = null;
		const wrap = document.createElement("div");
		wrap.className = "code-block-wrap";
		pre.parentNode.insertBefore(wrap, pre);
		wrap.appendChild(pre);
		if (placeholderRows.length > 0) {
			fieldsEl = document.createElement("div");
			fieldsEl.className = "md-code-placeholder-fields";
			let codeDisplayRaf = null;
			const refreshCodeDisplayFromFields = () => {
				code.textContent = applyPlaceholdersFromFields(originalForCopy, fieldsEl);
				tryHljsHighlightNonMermaid(code);
			};
			const scheduleRefreshCodeDisplay = () => {
				if (codeDisplayRaf !== null) {
					window.cancelAnimationFrame(codeDisplayRaf);
				}
				codeDisplayRaf = window.requestAnimationFrame(() => {
					codeDisplayRaf = null;
					refreshCodeDisplayFromFields();
				});
			};
			for (const row of placeholderRows) {
				const wrapRow = document.createElement("div");
				wrapRow.className = "md-code-placeholder-row";
				const input = document.createElement("input");
				input.type = "text";
				input.className = "md-code-placeholder-input";
				input.setAttribute("data-placeholder-literal", row.full);
				if (row.kind === "user") {
					input.placeholder = row.shortId;
					input.setAttribute("aria-label", row.shortId);
				} else {
					input.setAttribute("data-placeholder-kind", "datetime");
					input.setAttribute("data-default-replacement", row.defaultVal);
					input.value = row.defaultVal;
					input.placeholder = row.inner;
					input.setAttribute("aria-label", row.inner);
				}
				input.addEventListener("input", scheduleRefreshCodeDisplay);
				wrapRow.appendChild(input);
				fieldsEl.appendChild(wrapRow);
			}
			wrap.appendChild(fieldsEl);
			refreshCodeDisplayFromFields();
		}
		const btn = document.createElement("button");
		btn.type = "button";
		btn.className = "copy-btn";
		btn.innerHTML = copyIconHtml;
		btn.setAttribute("aria-label", "Copy");
		btn.title = "Copy";
		let revertTimerId = null;
		const flashCopyOk = () => {
			btn.innerHTML = copiedIconHtml;
			btn.classList.remove("is-copy-reacted");
			btn.classList.remove("is-copy-done");
			window.requestAnimationFrame(() => {
				void btn.offsetWidth;
				btn.classList.add("is-copy-reacted");
				btn.classList.add("is-copy-done");
			});
			if (revertTimerId !== null) {
				window.clearTimeout(revertTimerId);
			}
			revertTimerId = window.setTimeout(() => {
				revertTimerId = null;
				btn.innerHTML = copyIconHtml;
				btn.classList.remove("is-copy-reacted");
				btn.classList.remove("is-copy-done");
				btn.blur();
			}, 1200);
		};
		const copyBlockText = async () => {
			let text =
				fieldsEl !== null
					? applyPlaceholdersFromFields(originalForCopy, fieldsEl)
					: normalizeNewlines(code.textContent || "");
			const titleDefault = "Copy";
			const onCopied = () => {
				btn.title = "Copied";
				flashCopyOk();
				window.setTimeout(() => {
					btn.title = titleDefault;
				}, 1200);
			};
			try {
				if (navigator.clipboard && typeof navigator.clipboard.writeText === "function") {
					await navigator.clipboard.writeText(text);
					onCopied();
					return;
				}
			} catch (_) {
				/* fall through to execCommand */
			}
			if (copyPlainTextViaExecCommand(text)) {
				onCopied();
				return;
			}
			btn.title = "Copy failed";
			window.setTimeout(() => {
				btn.title = titleDefault;
			}, 2200);
		};
		btn.onclick = () => {
			void copyBlockText();
		};
		wrap.appendChild(btn);
	};

	const getCellTextForCopy = (cell) => {
		if (!cell) {
			return "";
		}
		const blockAppendNewline = new Set([
			"P",
			"DIV",
			"LI",
			"H1",
			"H2",
			"H3",
			"H4",
			"H5",
			"H6",
			"PRE",
		]);
		const out = [];
		const walk = (node) => {
			if (node.nodeType === Node.TEXT_NODE) {
				out.push(node.nodeValue || "");
				return;
			}
			if (node.nodeType !== Node.ELEMENT_NODE) {
				return;
			}
			const tag = node.tagName;
			if (tag === "BR") {
				out.push("\n");
				return;
			}
			for (const c of node.childNodes) {
				walk(c);
			}
			if (blockAppendNewline.has(tag)) {
				out.push("\n");
			}
		};
		for (const c of cell.childNodes) {
			walk(c);
		}
		return out
			.join("")
			.replace(/\r\n/g, "\n")
			.replace(/\r/g, "\n")
			.trim();
	};

	const tableToMatrix = (table) => {
		const rows = [];
		let maxCols = 0;
		const sections = Array.from(table.querySelectorAll(":scope > thead, :scope > tbody, :scope > tfoot"));
		const pushRow = (tr) => {
			const cells = Array.from(tr.querySelectorAll(":scope > th, :scope > td")).map((cell) => getCellTextForCopy(cell));
			maxCols = Math.max(maxCols, cells.length);
			rows.push(cells);
		};
		if (sections.length > 0) {
			for (const sec of sections) {
				for (const tr of sec.querySelectorAll(":scope > tr")) {
					pushRow(tr);
				}
			}
		} else {
			for (const tr of table.querySelectorAll(":scope > tr")) {
				pushRow(tr);
			}
		}
		rows.forEach((r) => {
			while (r.length < maxCols) {
				r.push("");
			}
		});
		return rows;
	};

	const escapeCsvCell = (value) => {
		const s = String(value);
		if (/[",\r\n]/.test(s)) {
			return '"' + s.replace(/"/g, '""') + '"';
		}
		return s;
	};

	const matrixToCsv = (matrix) => matrix.map((row) => row.map(escapeCsvCell).join(",")).join("\n");

	const escapeTsvCell = (value) => {
		const s = String(value);
		if (/[\t\r\n"]/.test(s)) {
			return '"' + s.replace(/"/g, '""') + '"';
		}
		return s;
	};

	const matrixToTsv = (matrix) => matrix.map((row) => row.map(escapeTsvCell).join("\t")).join("\n");

	const attachTableCopyUi = (table) => {
		if (!table || !table.parentNode || table.parentElement?.classList.contains("md-table-scroll")) {
			return;
		}
		const wrap = document.createElement("div");
		wrap.className = "md-table-wrap";
		const toolbar = document.createElement("div");
		toolbar.className = "md-table-toolbar";
		toolbar.setAttribute("role", "toolbar");
		toolbar.setAttribute("aria-label", "Copy table");
		const scroll = document.createElement("div");
		scroll.className = "md-table-scroll";
		const mkBtn = (format, ariaLabel, displayLabel) => {
			const b = document.createElement("button");
			b.type = "button";
			b.className = "md-table-format-btn";
			b.textContent = displayLabel;
			b.dataset.copyLabel = displayLabel;
			b.setAttribute("data-format", format);
			b.setAttribute("aria-label", ariaLabel);
			b.title = ariaLabel;
			return b;
		};
		const btnCsv = mkBtn("csv", "Copy as CSV", "CSV");
		const btnTsv = mkBtn("tsv", "Copy as TSV", "TSV");
		toolbar.appendChild(btnCsv);
		toolbar.appendChild(btnTsv);
		table.parentNode.insertBefore(wrap, table);
		wrap.appendChild(toolbar);
		scroll.appendChild(table);
		wrap.appendChild(scroll);
		const flashFormatBtn = (btn, timerRef) => {
			const restoreLabel = btn.dataset.copyLabel || "";
			btn.innerHTML = copiedIconHtml;
			btn.classList.add("md-table-format-btn--icon");
			btn.classList.remove("is-copy-reacted");
			btn.classList.remove("is-copy-done");
			window.requestAnimationFrame(() => {
				void btn.offsetWidth;
				btn.classList.add("is-copy-reacted");
				btn.classList.add("is-copy-done");
			});
			if (timerRef.v !== null) {
				window.clearTimeout(timerRef.v);
			}
			timerRef.v = window.setTimeout(() => {
				timerRef.v = null;
				btn.classList.remove("md-table-format-btn--icon");
				btn.textContent = restoreLabel;
				btn.classList.remove("is-copy-reacted");
				btn.classList.remove("is-copy-done");
				btn.blur();
			}, 1200);
		};
		const copyMatrix = async (text, btn, timerRef, titleOk) => {
			const restoreTitle = btn.title;
			const okExec = copyPlainTextViaExecCommand(text);
			if (okExec) {
				btn.title = titleOk;
				flashFormatBtn(btn, timerRef);
				window.setTimeout(() => {
					btn.title = restoreTitle;
				}, 1200);
				return;
			}
			try {
				if (navigator.clipboard && typeof navigator.clipboard.writeText === "function") {
					await navigator.clipboard.writeText(text);
				} else {
					btn.title = "Copy failed";
					window.setTimeout(() => {
						btn.title = restoreTitle;
					}, 2200);
					return;
				}
				btn.title = titleOk;
				flashFormatBtn(btn, timerRef);
				window.setTimeout(() => {
					btn.title = restoreTitle;
				}, 1200);
			} catch (_) {
				btn.title = "Copy failed";
				window.setTimeout(() => {
					btn.title = restoreTitle;
				}, 2200);
			}
		};
		const timerCsv = { v: null };
		const timerTsv = { v: null };
		btnCsv.addEventListener("click", () => {
			const text = matrixToCsv(tableToMatrix(table));
			void copyMatrix(text, btnCsv, timerCsv, "Copied CSV");
		});
		btnTsv.addEventListener("click", () => {
			const text = matrixToTsv(tableToMatrix(table));
			void copyMatrix(text, btnTsv, timerTsv, "Copied TSV");
		});
	};

	const renderMermaidBlocks = async () => {
		const codeBlocks = Array.from(document.querySelectorAll("pre code.language-mermaid"));
		if (!codeBlocks.length || !window.mermaid || typeof window.mermaid.render !== "function") {
			return;
		}
		if (typeof window.mermaid.initialize === "function") {
			window.mermaid.initialize({
				startOnLoad: false,
				securityLevel: "strict",
				theme: "base",
				fontSize: 10,
				themeVariables: {
					fontFamily:
						'"Segoe UI","Yu Gothic UI","Meiryo","Hiragino Kaku Gothic ProN","Noto Sans JP",sans-serif',
					fontSize: "10px",
					darkMode: true,
					background: "#0b0c10",
					mainBkg: "#12131a",
					secondBkg: "#12131a",
					primaryColor: "#1a1b24",
					secondaryColor: "#12131a",
					tertiaryColor: "#0b0c10",
					primaryTextColor: "#e2e4ed",
					secondaryTextColor: "#aeb0bf",
					tertiaryTextColor: "#8c8e9c",
					primaryBorderColor: "#343542",
					secondaryBorderColor: "#2e2f3a",
					lineColor: "#5f616e",
					textColor: "#e2e4ed",
					actorBkg: "#15161f",
					actorBorder: "#393a46",
					actorTextColor: "#eceef5",
					actorLineColor: "#676973",
					signalColor: "#9ca3af",
					signalTextColor: "#e6e8f0",
					labelBoxBkgColor: "#15161f",
					labelBoxBorderColor: "#3b3c49",
					labelTextColor: "#bfc2d4",
					loopTextColor: "#8c8e9c",
					noteBorderColor: "#3b3c49",
					noteBkgColor: "#13141c",
					noteTextColor: "#b4b8cc",
					activationBkgColor: "#282a37",
					activationBorderColor: "#4b4d60",
				},
				sequence: {
					// row metrics; sync with md.css --mermaid-font-size (flat Ve.messageFontSize uses root fontSize in setConf)
					useMaxWidth: true,
					diagramMarginX: 25,
					diagramMarginY: 5,
					actorMargin: 25,
					width: 75,
					height: 33,
					boxMargin: 5,
					boxTextMargin: 3,
					noteMargin: 5,
					messageMargin: 17,
					wrapPadding: 5,
					actorFontSize: 10,
					noteFontSize: 10,
					messageFontSize: 10,
				},
			});
		}
		for (let index = 0; index < codeBlocks.length; index += 1) {
			const code = codeBlocks[index];
			const pre = code.parentElement;
			if (!pre) {
				continue;
			}
			const source = code.textContent || "";
			const tabRId = `mclocks-mermaid-tab-r-${index}`;
			const tabSId = `mclocks-mermaid-tab-s-${index}`;
			const panelRId = `mclocks-mermaid-render-${index}`;
			const panelSId = `mclocks-mermaid-source-${index}`;
			const wrapper = document.createElement("div");
			wrapper.className = "mermaid-block";
			const tabstrip = document.createElement("div");
			tabstrip.className = "mermaid-tabstrip";
			tabstrip.setAttribute("role", "tablist");
			tabstrip.setAttribute("aria-label", "Mermaid view");
			const btnDiagram = document.createElement("button");
			btnDiagram.type = "button";
			btnDiagram.className = "mermaid-mode-btn mermaid-mode-btn--diagram is-active";
			btnDiagram.setAttribute("data-mermaid-mode", "diagram");
			btnDiagram.setAttribute("role", "tab");
			btnDiagram.setAttribute("id", tabRId);
			btnDiagram.setAttribute("aria-selected", "true");
			btnDiagram.setAttribute("aria-controls", panelRId);
			btnDiagram.setAttribute("aria-label", "Diagram");
			btnDiagram.title = "Diagram";
			btnDiagram.textContent = "\u21C4\u25B0";
			const btnSource = document.createElement("button");
			btnSource.type = "button";
			btnSource.className = "mermaid-mode-btn mermaid-mode-btn--source";
			btnSource.setAttribute("data-mermaid-mode", "source");
			btnSource.setAttribute("role", "tab");
			btnSource.setAttribute("id", tabSId);
			btnSource.setAttribute("aria-selected", "false");
			btnSource.setAttribute("aria-controls", panelSId);
			btnSource.setAttribute("aria-label", "Source");
			btnSource.title = "Source";
			btnSource.textContent = "</>";
			tabstrip.appendChild(btnDiagram);
			tabstrip.appendChild(btnSource);
			const btnCopy = document.createElement("button");
			btnCopy.type = "button";
			btnCopy.className = "copy-btn";
			btnCopy.innerHTML = copyIconHtml;
			const head = document.createElement("div");
			head.className = "mermaid-block-head";
			head.appendChild(tabstrip);
			head.appendChild(btnCopy);
			const body = document.createElement("div");
			body.className = "mermaid-block-body";
			const panelRender = document.createElement("div");
			panelRender.className = "mermaid-panel mermaid-panel--render";
			panelRender.id = panelRId;
			panelRender.setAttribute("role", "tabpanel");
			panelRender.setAttribute("aria-labelledby", tabRId);
			const panelSource = document.createElement("div");
			panelSource.className = "mermaid-panel mermaid-panel--source";
			panelSource.id = panelSId;
			panelSource.setAttribute("role", "tabpanel");
			panelSource.setAttribute("aria-labelledby", tabSId);
			panelSource.hidden = true;
			const preSource = document.createElement("pre");
			const codeSource = document.createElement("code");
			codeSource.className = (code.getAttribute("class") || "").trim() || "language-mermaid";
			codeSource.textContent = source;
			preSource.appendChild(codeSource);
			panelSource.appendChild(preSource);
			body.appendChild(panelRender);
			body.appendChild(panelSource);
			wrapper.appendChild(head);
			wrapper.appendChild(body);
			const diagramMount = document.createElement("div");
			diagramMount.className = "mermaid-diagram";
			try {
				const id = `mclocks-mermaid-${index}`;
				const rendered = await window.mermaid.render(id, source);
				diagramMount.innerHTML = rendered.svg;
				panelRender.appendChild(diagramMount);
				pre.replaceWith(wrapper);
				let mermaidCopyRevertTimerId = null;
				const updateMermaidCopyChrome = () => {
					const diagramOn = !panelRender.hidden;
					btnCopy.disabled = diagramOn;
					if (diagramOn) {
						btnCopy.setAttribute("aria-label", "Copy unavailable — use Source tab");
						btnCopy.title = "Switch to Source tab to copy";
						return;
					}
					btnCopy.setAttribute("aria-label", "Copy");
					btnCopy.title = "Copy";
				};
				const flashMermaidCopyOk = () => {
					btnCopy.innerHTML = copiedIconHtml;
					btnCopy.classList.remove("is-copy-reacted");
					btnCopy.classList.remove("is-copy-done");
					window.requestAnimationFrame(() => {
						void btnCopy.offsetWidth;
						btnCopy.classList.add("is-copy-reacted");
						btnCopy.classList.add("is-copy-done");
					});
					if (mermaidCopyRevertTimerId !== null) {
						window.clearTimeout(mermaidCopyRevertTimerId);
					}
					mermaidCopyRevertTimerId = window.setTimeout(() => {
						mermaidCopyRevertTimerId = null;
						btnCopy.innerHTML = copyIconHtml;
						btnCopy.classList.remove("is-copy-reacted");
						btnCopy.classList.remove("is-copy-done");
						btnCopy.blur();
						updateMermaidCopyChrome();
					}, 1400);
				};
				btnCopy.addEventListener("click", async () => {
					if (!panelSource.hidden) {
						const text = codeSource.textContent || "";
						if (copyPlainTextViaExecCommand(text)) {
							btnCopy.title = "Copied";
							flashMermaidCopyOk();
							return;
						}
						try {
							await navigator.clipboard.writeText(text);
						} catch (_) {
							btnCopy.title = "Copy failed";
							window.setTimeout(updateMermaidCopyChrome, 2200);
							return;
						}
						btnCopy.title = "Copied";
						flashMermaidCopyOk();
					}
				});
				const setMode = (mode) => {
					const isDiagram = mode === "diagram";
					btnDiagram.classList.toggle("is-active", isDiagram);
					btnSource.classList.toggle("is-active", !isDiagram);
					btnDiagram.setAttribute("aria-selected", String(isDiagram));
					btnSource.setAttribute("aria-selected", String(!isDiagram));
					panelRender.hidden = !isDiagram;
					panelSource.hidden = isDiagram;
					updateMermaidCopyChrome();
				};
				updateMermaidCopyChrome();
				btnDiagram.addEventListener("click", () => setMode("diagram"));
				btnSource.addEventListener("click", () => setMode("source"));
			} catch (_) {
				// Keep the original code block if rendering fails.
			}
		}
	};

	await renderMermaidBlocks();

	document.querySelectorAll("pre code").forEach((code) => {
		tryHljsHighlightNonMermaid(code);
	});

	document.querySelectorAll("pre code").forEach((code) => {
		if (code.closest(".mermaid-block")) {
			return;
		}
		attachCodeBlockCopyUi(code.parentElement, code);
	});

	document.querySelectorAll("#content table").forEach((table) => {
		if (table.closest("td, th")) {
			return;
		}
		attachTableCopyUi(table);
	});

	const pathCopyBtn = document.getElementById("path-copy-btn");
	const pathLabel = document.getElementById("main-header-path");
	if (pathCopyBtn && pathLabel) {
		pathCopyBtn.addEventListener("click", () => {
			navigator.clipboard.writeText(pathLabel.textContent || "");
			pathCopyBtn.textContent = "Copied!";
			pathCopyBtn.blur();
			setTimeout(() => {
				pathCopyBtn.textContent = "Copy";
				pathCopyBtn.blur();
			}, 2000);
		});
	}

	const summaryList = document.getElementById("summary-list");
	if (summaryList) {
		const pad2 = (n) => String(n).padStart(2, "0");
		const toLocalTime = (value) => {
			const n = Number(value);
			if (!Number.isFinite(n)) {
				return null;
			}
			const d = new Date(n);
			const y = d.getFullYear();
			const mo = pad2(d.getMonth() + 1);
			const da = pad2(d.getDate());
			const h = pad2(d.getHours());
			const mi = pad2(d.getMinutes());
			const s = pad2(d.getSeconds());
			return `${y}-${mo}-${da} ${h}:${mi}:${s}`;
		};
		summaryList.querySelectorAll("li").forEach((item) => {
			const label = item.querySelector(".label");
			const value = item.querySelector(".value");
			if (!label || !value) {
				return;
			}
			if (label.textContent?.trim() !== "Last Mod") {
				return;
			}
			const formatted = toLocalTime(value.textContent?.trim());
			if (formatted) {
				value.textContent = formatted;
			}
		});
	}

	const tocList = document.getElementById("toc-list");
	if (!tocList) {
		return;
	}

	const links = tocList.querySelectorAll("a");
	const headings = document.querySelectorAll("#content h1, #content h2, #content h3, #content h4");
	const observer = new IntersectionObserver(
		(entries) => {
			entries.forEach((entry) => {
				if (entry.isIntersecting) {
					const id = entry.target.id;
					links.forEach((a) => a.classList.toggle("active", a.getAttribute("href") === `#${id}`));
				}
			});
		},
		{ rootMargin: "0px 0px -80% 0px", threshold: 0 }
	);
	headings.forEach((h) => observer.observe(h));
})();
