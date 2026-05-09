"use strict";

const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const { spawn, execFileSync } = require("child_process");

/**
 * Hard override: set `true` to always append traces (same as setting / env below).
 */
const DEVELOP_DEBUG_LOG = false;

/** @type {vscode.OutputChannel | undefined} */
let previewOutput;

function isoNow() {
	return new Date().toISOString();
}

function developDebugEnabled() {
	if (DEVELOP_DEBUG_LOG) {
		return true;
	}
	if (String(process.env.MCLOCKS_PREVIEW_DEBUG || "").trim() === "1") {
		return true;
	}
	if (String(process.env.MCLOCKS_MD_PREVIEW_DEBUG || "").trim() === "1") {
		return true;
	}
	try {
		return vscode.workspace.getConfiguration("mclocks.mcPreview").get("debug") === true;
	} catch {
		return false;
	}
}

/**
 * @param {string} msg
 */
function dbg(msg) {
	if (!previewOutput || !developDebugEnabled()) {
		return;
	}
	previewOutput.appendLine(`${isoNow()} ${msg}`);
}

function dbgReveal() {
	if (!previewOutput || !developDebugEnabled()) {
		return;
	}
	previewOutput.show(true);
}

/**
 * @param {string} s
 * @param {number} max
 */
function clip(s, max) {
	if (!s || s.length <= max) {
		return s;
	}
	return `${s.slice(0, max)}…(+${s.length - max} chars)`;
}

/**
 * @param {import("vscode").Uri} uri
 */
function isDiskLikeResourceUri(uri) {
	return (
		vscode.Uri.isUri(uri) &&
		(uri.scheme === "file" || uri.scheme === "vscode-remote")
	);
}

/**
 * @param {import("vscode").Uri} uri
 */
function uriFsLikePath(uri) {
	const p = uri.fsPath;
	return p && p.length > 0 ? p : uri.path;
}

/**
 * @returns {string | null}
 */
function findBundledScript() {
	const roots = vscode.workspace.workspaceFolders || [];
	for (const f of roots) {
		const candidate = vscode.Uri.joinPath(f.uri, "scripts", "mc-preview").fsPath;
		try {
			if (fs.statSync(candidate).isFile()) {
				return candidate;
			}
		} catch {
			// continue
		}
	}
	return null;
}

/**
 * @param {string} configured
 * @returns {string}
 */
function resolveBash(configured) {
	const t = (configured || "").trim();
	if (t) {
		return t;
	}
	return process.platform === "win32" ? "bash.exe" : "bash";
}

/**
 * /home/... など drvfs 以外の Linux パスを Windows 側が読める UNC へ（wslpath -w）。
 * @param {string} posix
 * @param {import("vscode").WorkspaceConfiguration} conf
 */
function wslLinuxFsPathToWindowsPath(posix, conf) {
	if (conf.get("wslMapLinuxFsToWindowsPath") === false) {
		return posix;
	}
	try {
		const win = execFileSync("wslpath", ["-w", posix], {
			encoding: "utf8",
			timeout: 12000,
			maxBuffer: 1024 * 1024,
			windowsHide: true,
		});
		const t = win.replace(/\0/g, "").trim().replace(/\r?\n/g, "");
		if (t.length > 0) {
			dbg(`wslpath -w: ${clip(posix, 120)} → ${clip(t, 200)}`);
			return t;
		}
	} catch (e) {
		dbg(`wslpath -w failed: ${clip(String(e && e.message ? e.message : e), 200)}`);
	}
	const dist = (process.env.WSL_DISTRO_NAME || "").trim();
	if (!dist) {
		return posix;
	}
	const unc =
		`\\\\wsl$\\${dist}` +
		posix.replace(/\//g, "\\");
	dbg(`wslpath fallback UNC: ${clip(unc, 240)}`);
	return unc;
}

/**
 * POST body 1 行目: Windows の mclocks が canonicalize できるパス。WSL は /mnt/c→ドライブ、/home 等は UNC。
 * @param {string} fsPath
 * @param {import("vscode").WorkspaceConfiguration} conf
 */
function pathForMcPreviewPayload(fsPath, conf) {
	const raw = String(fsPath || "").trim();
	const posix = raw.replace(/\\/g, "/");
	const mapDrv = conf.get("wslMapMntCToWindowsDrive") !== false;
	if (process.platform === "win32") {
		const m = /^([a-zA-Z]):\/*(.*)$/i.exec(posix);
		return m ? `${m[1].toUpperCase()}:${m[2] ? `/${m[2]}` : ""}` : posix;
	}
	if (mapDrv) {
		const mn = /^\/mnt\/([a-zA-Z])\/?(.*)$/i.exec(posix);
		if (mn) {
			const letter = mn[1].toUpperCase();
			const rest = String(mn[2] || "").replace(/^\/+/, "");
			return rest ? `${letter}:/${rest}` : `${letter}:/`;
		}
	}
	if (
		process.platform !== "win32" &&
		isProbablyWslGuest() &&
		posix.startsWith("/") &&
		!/^\/mnt\/[a-zA-Z]\//i.test(posix)
	) {
		return wslLinuxFsPathToWindowsPath(posix, conf);
	}
	return raw.length > 0 ? raw : posix;
}

/**
 * @param {import("vscode").WorkspaceConfiguration} conf
 */
function normalizePort(conf) {
	const raw = conf.get("webPort");
	const n =
		typeof raw === "number"
			? raw
			: typeof raw === "string"
				? parseInt(String(raw).trim(), 10)
				: NaN;
	return Number.isFinite(n) && n > 0 ? n : 3030;
}

function isProbablyWslGuest() {
	if (process.env.WSL_DISTRO_NAME) {
		return true;
	}
	if (process.env.WSL_INTEROP) {
		return true;
	}
	const rn = vscode.env.remoteName;
	return typeof rn === "string" && rn.toLowerCase().includes("wsl");
}

/**
 * Typical WSL2: first IPv4 nameserver is the Windows host as seen from the distro.
 */
function readEtcResolvNameserverIpv4Sync() {
	try {
		const text = fs.readFileSync("/etc/resolv.conf", { encoding: "utf8", flag: "r" });
		for (const line of text.split("\n")) {
			const trimmed = line.trim();
			if (!trimmed || trimmed.startsWith("#")) {
				continue;
			}
			const m =
				/^nameserver\s+(\d{1,3}(?:\.\d{1,3}){3})\s*$/i.exec(trimmed);
			if (m) {
				return m[1];
			}
		}
	} catch {
		return null;
	}
	return null;
}

/**
 * @param {import("vscode").WorkspaceConfiguration} conf
 */
function previewTarget(conf) {
	let host =
		String(conf.get("webHost") || "127.0.0.1").trim() || "127.0.0.1";
	const port = normalizePort(conf);
	const mapLh = conf.get("wslMapLocalhostToWindowsHost") !== false;
	const isLh = /^127\.0\.0\.1$/i.test(host) || /^localhost$/i.test(host);
	if (
		mapLh &&
		isLh &&
		process.platform !== "win32" &&
		isProbablyWslGuest()
	) {
		const gw = readEtcResolvNameserverIpv4Sync();
		if (gw) {
			host = gw;
		}
	}
	return { host, port };
}

/**
 * WSL での到達順: localhost 転送がある環境では先に 127.0.0.1 が効く。broad bind の Windows 側なら名前解決ホストも試す。
 * @returns {string[]}
 */
function previewPostUrlCandidates(conf) {
	const { host, port } = previewTarget(conf);
	const gw = `http://${host}:${port}/preview`;
	const loop = `http://127.0.0.1:${port}/preview`;
	const mapLh = conf.get("wslMapLocalhostToWindowsHost") !== false;
	if (!isProbablyWslGuest() || !mapLh) {
		return [gw];
	}
	const hl = String(host).toLowerCase();
	if (hl === "127.0.0.1" || hl === "localhost") {
		return [gw];
	}
	return [loop, gw];
}

/**
 * Windows の curl.exe（WSL から /mnt/c/... 経由）。既定パスが無ければ null。
 * @param {import("vscode").WorkspaceConfiguration} conf
 */
function resolveWindowsCurlPathInWsl(conf) {
	const custom = String(conf.get("wslWindowsCurlPath") || "").trim();
	const candidates = custom
		? [custom]
		: [
				"/mnt/c/Windows/System32/curl.exe",
				"/mnt/c/Windows/Sysnative/curl.exe",
			];
	for (const p of candidates) {
		try {
			if (p && fs.existsSync(p) && fs.statSync(p).isFile()) {
				return p;
			}
		} catch {
			// continue
		}
	}
	return null;
}

const CURL_HTTP_MARKER = "__MC_HTTP_STATUS__";

/**
 * curl `-w` appends `\n__MC_HTTP_STATUS__<code>` after the response body (no `--fail`, so body is preserved on 4xx).
 * @param {string} combined
 * @returns {{ httpCode: number, body: string }}
 */
function parseCurlHttpSuffix(combined) {
	const mark = `\n${CURL_HTTP_MARKER}`;
	const i = combined.lastIndexOf(mark);
	if (i < 0) {
		return { httpCode: NaN, body: combined };
	}
	const body = combined.slice(0, i);
	const tail = combined.slice(i + mark.length).trim();
	const n = parseInt(tail, 10);
	return {
		httpCode: Number.isFinite(n) ? n : NaN,
		body,
	};
}

/**
 * @param {{ code: number|null, spawnError?: boolean, httpCode?: number, responseBody?: string }} r
 */
function isMcPreviewCurlSuccess(r) {
	if (r.spawnError || r.code !== 0) {
		return false;
	}
	const httpOk =
		typeof r.httpCode === "number" &&
		Number.isFinite(r.httpCode) &&
		r.httpCode === 200;
	const textOk = String(r.responseBody || "").trim() === "OK";
	return httpOk && textOk;
}

/**
 * @param {string} curlBin
 * @param {string} url
 * @param {string} body
 * @param {(s: string) => void} [logAppend]
 * @param {boolean} [stdinBody] pass body on stdin (--data-binary @-) so UNC backslashes are not argv-mangled
 * @returns {Promise<{code:number|null,out:string,url:string,spawnError?:boolean,httpCode:number,responseBody:string}>}
 */
function curlPostPreviewOnce(curlBin, url, body, logAppend, stdinBody) {
	let out = "";
	const bodyNote = clip(body, 240);
	const via = stdinBody ? "stdin (@-)" : "argv";
	logAppend?.(
		`curl try (${via}): ${curlBin} -sS --connect-timeout 8 --max-time 45 -X POST <url> --data-binary -w <http_code> <len=${body.length}>`,
	);
	logAppend?.(`  url: ${url}`);
	logAppend?.(`  body: ${bodyNote}`);
	const args = [
		"-sS",
		"--connect-timeout",
		"8",
		"--max-time",
		"45",
		"-X",
		"POST",
		url,
		"--data-binary",
		stdinBody ? "@-" : body,
		"-w",
		`\n${CURL_HTTP_MARKER}%{http_code}`,
	];
	return new Promise((resolve) => {
		const child = spawn(curlBin, args, {
			windowsHide: true,
			env: process.env,
			stdio: stdinBody ? ["pipe", "pipe", "pipe"] : undefined,
		});
		if (stdinBody && child.stdin) {
			child.stdin.write(Buffer.from(body, "utf8"));
			child.stdin.end();
		}
		child.stdout.on("data", (d) => {
			out += d.toString();
		});
		child.stderr.on("data", (d) => {
			out += d.toString();
		});
		child.on("error", (e) => {
			logAppend?.(`curl spawn error: ${e.message}`);
			resolve({
				code: null,
				out: e.message,
				url,
				spawnError: true,
				httpCode: NaN,
				responseBody: "",
			});
		});
		child.on("close", (code) => {
			const { httpCode, body: responseBody } = parseCurlHttpSuffix(out);
			logAppend?.(
				`curl exit pid=${typeof child.pid === "number" ? child.pid : "?"} code=${code ?? "?"} httpStatus=${Number.isFinite(httpCode) ? httpCode : "?"} body=${clip(String(responseBody || "").trim(), 600)}`,
			);
			resolve({
				code: code ?? 1,
				out,
				url,
				spawnError: false,
				httpCode,
				responseBody,
			});
		});
	});
}

/**
 * @returns {"curl"|"bash"}
 */
function resolvedTransport(conf) {
	const raw = String((conf.get("transport") || "auto").trim().toLowerCase());
	if (raw === "curl" || raw === "auto") {
		return "curl";
	}
	if (raw === "bash") {
		return "bash";
	}
	return "curl";
}

/**
 * mc-preview と同じ MCLOCKS_* を子プロセスに渡す。
 * @param {import("vscode").WorkspaceConfiguration} conf
 * @returns {NodeJS.ProcessEnv}
 */
function buildSpawnEnv(conf) {
	const env =
		process.platform === "win32"
			? { ...process.env, MSYS2_ARG_CONV_EXACT: "1" }
			: { ...process.env };
	const { host, port } = previewTarget(conf);
	env.MCLOCKS_WEB_HOST = host;
	env.MCLOCKS_WEB_PORT = String(port);
	return env;
}

/**
 * @param {string} out
 * @param {import("vscode").WorkspaceConfiguration} conf
 * @returns {string}
 */
function friendlyFailureDetail(out, conf) {
	const raw = (out || "").trim();
	const { host, port } = previewTarget(conf);
	if (
		raw.includes("Connection refused") ||
		/\bcurl:\s*\(7\)/.test(raw)
	) {
		const wslHint = isProbablyWslGuest()
			? ` WSL から Windows の mclocks へは、Linux の curl ではなく「Windows の curl.exe」(設定 wslInvokeWindowsCurl)か、Windows 側で MCLOCKS_MAIN_HTTP_BIND_ALL=1 が必要なことがあります。`
			: "";
		return `${raw}\n(${host}:${port} で待ち受けている mclocks のメイン HTTP がありません。mclocks を起動するか、起動ログのポートと「mclocks.mcPreview.webPort」を合わせてください。)${wslHint}`;
	}
	if (/\bcurl:\s*\(28\)/.test(raw) && isProbablyWslGuest()) {
		return `${raw}\n(WSL→Windows ホスト IP への TCP がタイムアウトしました。ファイアウォールか、mclocks が 127.0.0.1 のみリッスンしている可能性があります。拡張の「Windows curl.exe 経由」(wslInvokeWindowsCurl)を有効にするか、環境変数 MCLOCKS_MAIN_HTTP_BIND_ALL=1 で mclocks を起動してください。)`;
	}
	return raw;
}

/**
 * Bash がスクリプトファイルを読むための経路。mclocks POST 用パスとは別（WSL では /mnt/c/... が必要）。
 * @param {string} fsPath
 * @param {string} bashExe
 */
function pathForShellScriptExecutable(fsPath, bashExe) {
	if (process.platform !== "win32") {
		return fsPath;
	}
	const posix = fsPath.replace(/\\/g, "/").trim();
	const m = /^([a-zA-Z]):\/*(.*)$/i.exec(posix);
	const lowerBash = (bashExe || "").toLowerCase();
	const bashUsesMntCx =
		!lowerBash.includes("git\\") &&
		!lowerBash.includes("git/") &&
		!lowerBash.includes("msys") &&
		!lowerBash.includes("mingw");
	if (!m) {
		return posix;
	}
	if (bashUsesMntCx) {
		return `/mnt/${m[1].toLowerCase()}/${m[2]}`;
	}
	return `${m[1].toUpperCase()}:${m[2] ? `/${m[2]}` : ""}`;
}

/**
 * Absolute path as POST body line 1 (same as `curl --data-binary 'C:\...'` on Windows).
 * @param {import("vscode").Uri} fileUri
 * @param {import("vscode").WorkspaceConfiguration} conf
 * @returns {Promise<number | null>}
 */
function runMcPreviewCurl(fileUri, conf) {
	const body = pathForMcPreviewPayload(uriFsLikePath(fileUri), conf);
	const urls = previewPostUrlCandidates(conf);
	const curlBin = process.platform === "win32" ? "curl.exe" : "curl";
	const logL = (line) => dbg(line);
	const port = normalizePort(conf);
	return (async () => {
		if (
			process.platform !== "win32" &&
			isProbablyWslGuest() &&
			conf.get("wslInvokeWindowsCurl") !== false
		) {
			const winCurl = resolveWindowsCurlPathInWsl(conf);
			if (winCurl) {
				const loopUrl = `http://127.0.0.1:${port}/preview`;
				logL(
					`runMcPreviewCurl: try Windows curl first (TCP from Windows): ${winCurl} → ${loopUrl}`,
				);
				const wr = await curlPostPreviewOnce(
					winCurl,
					loopUrl,
					body,
					logL,
					true,
				);
				if (isMcPreviewCurlSuccess(wr)) {
					logL(
						"success via Windows curl.exe (browser opens on Windows mclocks)",
					);
					return 0;
				}
				logL(
					`Windows curl.exe path did not return OK (curlExit=${wr.code} http=${wr.httpCode} body=${clip(String(wr.responseBody || "").trim(), 200)}); falling back to Linux curl`,
				);
			} else {
				logL(
					"runMcPreviewCurl: no /mnt/c/.../curl.exe found (set mclocks.mcPreview.wslWindowsCurlPath)",
				);
			}
		}
		logL(`runMcPreviewCurl curlBin=${curlBin} urlCandidates=${JSON.stringify(urls)}`);
		let last = {
			code: /** @type {number|null} */ (1),
			out: "",
			url: urls[0] || "",
			httpCode: NaN,
			responseBody: "",
		};
		for (const url of urls) {
			const r = await curlPostPreviewOnce(curlBin, url, body, logL, false);
			last = {
				code: r.code,
				out: r.out,
				url: r.url,
				httpCode: r.httpCode,
				responseBody: r.responseBody,
			};
			if (r.spawnError) {
				logL(`abort: spawnError url=${r.url}`);
				vscode.window.showErrorMessage(`mclocks preview: ${r.out}`);
				return null;
			}
			if (isMcPreviewCurlSuccess(r)) {
				logL("success: HTTP OK and body is OK (browser should open on mclocks host)");
				return 0;
			}
			if (r.code === 0) {
				logL(
					`retry: ${r.url} http=${r.httpCode} body=${clip(String(r.responseBody || "").trim(), 400)}`,
				);
				continue;
			}
		}
		if (last.code === null) {
			return null;
		}
		if (last.code === 0) {
			const hint = String(last.responseBody || "").trim();
			logL(`fail: unexpected response after all URLs lastUrl=${last.url}`);
			vscode.window.showErrorMessage(
				`mclocks preview failed (${last.url}): HTTP ${Number.isFinite(last.httpCode) ? last.httpCode : "?"} ${hint ? clip(hint, 500) : (last.out || "").trim() || "(empty)"}`,
			);
			return 1;
		}
		const detail = friendlyFailureDetail(last.out, conf);
		const suffix = urls.length > 1 ? ` (tried: ${urls.join(" → ")})` : "";
		const msg = (detail || `exit ${last.code}`) + suffix;
		logL(`fail: ${clip(msg.replace(/\s+/g, " "), 800)}`);
		vscode.window.showErrorMessage(`mclocks preview failed: ${msg}`);
		return last.code;
	})();
}

/**
 * @param {import("vscode").Uri} fileUri
 * @param {string} scriptPath
 * @param {string} bashExe
 * @returns {Promise<number | null>}
 */
function runMcPreview(fileUri, scriptPath, bashExe, conf) {
	const filePath = uriFsLikePath(fileUri);
	const folder = vscode.workspace.getWorkspaceFolder(fileUri);
	const cwd = folder ? folder.uri.fsPath : path.dirname(filePath);
	const scriptForExec = pathForShellScriptExecutable(scriptPath, bashExe);
	const fileForPayload = pathForMcPreviewPayload(filePath, conf);
	let out = "";
	const env = buildSpawnEnv(conf);
	dbg(
		`runMcPreview bashExe=${bashExe} script=${scriptForExec} arg=${clip(fileForPayload, 240)} cwd=${cwd}`,
	);
	return new Promise((resolve) => {
		const child = spawn(bashExe, [scriptForExec, fileForPayload], {
			cwd,
			windowsHide: true,
			env,
		});
		child.stdout.on("data", (d) => {
			out += d.toString();
		});
		child.stderr.on("data", (d) => {
			out += d.toString();
		});
		child.on("error", (e) => {
			dbg(`bash spawn error: ${e.message}`);
			vscode.window.showErrorMessage(`mclocks preview: ${e.message}`);
			resolve(null);
		});
		child.on("close", (code) => {
			dbg(`bash exit code=${code} out=${clip((out || "").trim(), 500)}`);
			resolve(code);
		});
	}).then((code) => {
		if (code === null || code === 0) {
			return code;
		}
		const detail = friendlyFailureDetail(out, conf);
		const msg = detail || `exit ${code}`;
		vscode.window.showErrorMessage(`mclocks preview failed: ${msg}`);
		return code;
	});
}

/**
 * @param {import("vscode").ExtensionContext} context
 */
function activate(context) {
	previewOutput = vscode.window.createOutputChannel("mclocks Preview");
	context.subscriptions.push(previewOutput);
	context.subscriptions.push(
		vscode.commands.registerCommand("mclocks.mcPreview", async (uri) => {
			const conf = vscode.workspace.getConfiguration("mclocks.mcPreview");
			dbgReveal();
			dbg(`--- mclocks.mcPreview (${isoNow()}) ---`);
			const target = vscode.Uri.isUri(uri)
				? uri
				: vscode.window.activeTextEditor?.document.uri;
			dbg(
				`vscode.env.remoteName=${String(vscode.env.remoteName ?? "")} pid=${process.pid} platform=${process.platform}`,
			);
			dbg(`WSL_DISTRO_NAME=${process.env.WSL_DISTRO_NAME || "(unset)"}`);
			if (!target || !isDiskLikeResourceUri(target)) {
				dbg(`early exit: invalid target uri=${uri ? String(uri) : "(none)"}`);
				vscode.window.showWarningMessage(
					"mclocks preview: open or select a file on disk.",
				);
				return;
			}
			const fsProbe = uriFsLikePath(target);
			dbg(
				`target scheme=${target.scheme} fsPath=${clip(fsProbe, 400)}`,
			);
			const mode = resolvedTransport(conf);
			const pt = previewTarget(conf);
			dbg(
				`transport=${mode} previewTarget(host)=${pt.host} port=${pt.port} urls=${JSON.stringify(previewPostUrlCandidates(conf))}`,
			);
			dbg(
				`settings: webHost(conf)=${conf.get("webHost")} wslMapLh=${conf.get("wslMapLocalhostToWindowsHost") !== false} wslMapDrv=${conf.get("wslMapMntCToWindowsDrive") !== false}`,
			);
			if (mode === "curl") {
				await runMcPreviewCurl(target, conf);
				dbg("--- mcPreview curl path done ---");
				return;
			}
			let scriptPath = (conf.get("scriptPath") || "").trim();
			if (!scriptPath) {
				scriptPath = findBundledScript() || "";
			}
			if (!scriptPath) {
				dbg("bash transport but no scripts/mc-preview found");
				vscode.window.showErrorMessage(
					"mclocks preview: set mclocks.mcPreview.scriptPath, or open a workspace that contains scripts/mc-preview (transport is \"bash\"). Default transport is curl and does not need the script.",
				);
				return;
			}
			const bashExe = resolveBash(conf.get("bashExecutable") || "");
			await runMcPreview(target, scriptPath, bashExe, conf);
			dbg("--- mcPreview bash path done ---");
		}),
	);
}

function deactivate() {}

module.exports = { activate, deactivate };
