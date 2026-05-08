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
		const wrap = document.createElement("div");
		wrap.className = "code-block-wrap";
		pre.parentNode.insertBefore(wrap, pre);
		wrap.appendChild(pre);
		const btn = document.createElement("button");
		btn.type = "button";
		btn.className = "copy-btn";
		btn.innerHTML = copyIconHtml;
		btn.setAttribute("aria-label", "Copy");
		btn.title = "Copy";
		let revertTimerId = null;
		btn.onclick = () => {
			navigator.clipboard.writeText(code.textContent || "");
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
		wrap.appendChild(btn);
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

	if (window.hljs && typeof window.hljs.highlightElement === "function") {
		document.querySelectorAll("pre code").forEach((code) => {
			if ((code.className || "").split(/\s+/).includes("language-mermaid")) {
				return;
			}
			window.hljs.highlightElement(code);
		});
	}

	document.querySelectorAll("pre code").forEach((code) => {
		if (code.closest(".mermaid-block")) {
			return;
		}
		attachCodeBlockCopyUi(code.parentElement, code);
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
