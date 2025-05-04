import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';
import { ask, message } from '@tauri-apps/plugin-dialog';

const escapeTarget = {
  '&': '&amp;',
  "'": '&#x27;',
  '`': '&#x60;',
  '"': '&quot;',
  '<': '&lt;',
  '>': '&gt;',
};

export function escapeHTML(str) {
  return (str || '').replace(/[&'`"<>]/g, function (match) {
    return escapeTarget[match]
  });
}

export function pad(n) {
  if (n >= 0 && n < 10) {
    return "0" + n
  }

  return n
}

export function trim(str) {
  str = str.replace(/^[\s\t]+/, '');
  str = str.replace(/[\s\t]+$/, '');
  return str.replace(/\r?\n/g, '');
}

// return the list of timezones of clocks with no duplication
export function uniqueTimezones(clocks) {
  let flag = {};
  let list = [];
  clocks.getAllClocks().map((clock) => {
    if (clock.timezone && clock.timezone !== "" && !flag[clock.timezone]) {
      flag[clock.timezone] = true;
      list.push(clock.timezone);
    }
  });

  return list;
}


async function checkPermission() {
  if (!(await isPermissionGranted())) {
    return (await requestPermission()) === 'granted'
  }
  return true
}

export async function enqueueNotification(title, body) {
  if (!(await checkPermission())) {
    return
  }
  sendNotification({ title, body })
}

export async function writeClipboardText(text) {
  await writeText(text);
}

export async function readClipboardText() {
  return await readText()
}

export async function openAskDialog(body, title, kind) {
  const answer = await ask(body, {
    title: title ?? "mclocks",
    kind: kind ?? "info",
  });

  return answer;
}

export async function openMessageDialog(body, title, kind) {
  const ret = await message(body, {
    title: title ?? "mclocks",
    kind: kind ?? "info",
  });

  return ret;
}
