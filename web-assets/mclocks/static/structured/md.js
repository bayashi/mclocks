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
			const container = document.createElement("div");
			container.className = "mermaid-diagram";
			try {
				const id = `mclocks-mermaid-${index}`;
				const rendered = await window.mermaid.render(id, source);
				container.innerHTML = rendered.svg;
				pre.replaceWith(container);
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

	const copyIconHtml =
		'<svg class="copy-btn-svg" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">' +
		'<rect x="10" y="10" width="10" height="10" rx="1.85" ry="1.85" stroke="currentColor" stroke-width="1.65"/>' +
		'<rect x="4" y="4" width="10" height="10" rx="1.85" ry="1.85" stroke="currentColor" stroke-width="1.65"/>' +
		"</svg>";

	const copiedIconHtml =
		'<svg class="copy-btn-svg" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">' +
		'<path d="M7.05 13.42L10.2 16.62L17.92 8.92" stroke="currentColor" stroke-width="1.85" stroke-linecap="round" stroke-linejoin="round"/>' +
		"</svg>";

	document.querySelectorAll("pre code").forEach((code) => {
		const pre = code.parentElement;
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
