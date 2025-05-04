import { platform } from '@tauri-apps/plugin-os';

import { cdate } from 'cdate';

export class Ctx {
  constructor(mainElement) {
    this._mainElement = mainElement;

    this._cdateUTC = cdate().tz("UTC").cdateFn();

    this._ignoreOnMoved = false;
    this._pauseTimer = false;
    this._lockKeyP = false;
    this._displayEpoch = false;
  }

  static isMacOS = platform() === 'macOS';

  cdateUTC(cdt) {
    return this._cdateUTC(cdt);
  }

  mainElement() {
    return this._mainElement;
  }

  ignoreOnMoved() {
    return this._ignoreOnMoved;
  }
  setIgnoreOnMoved(ignoreOnMoved) {
    this._ignoreOnMoved = ignoreOnMoved;
    return this;
  }

  format() {
    return this._format;
  }
  setFormat(format) {
    this._format = format;
    return this;
  }

  pauseTimer() {
    return this._pauseTimer;
  }
  setPauseTimer(pauseTimer) {
    this._pauseTimer = pauseTimer;
    return this;
  }

  lockKeyP() {
    return this._lockKeyP;
  }
  setLockKeyP(lockKeyP) {
    this._lockKeyP = lockKeyP;
    return this;
  }

  timerIcon() {
    return this._timerIcon;
  }
  setTimerIcon(timerIcon) {
    this._timerIcon = timerIcon;
    return this;
  }

  withoutNotification() {
    return this._withoutNotification;
  }
  setWithoutNotification(withoutNotification) {
    this._withoutNotification = withoutNotification;
    return this;
  }

  maxTimerClockNumber() {
    return this._maxTimerClockNumber;
  }
  setMaxTimerClockNumber(maxTimerClockNumber) {
    this._maxTimerClockNumber = maxTimerClockNumber;
    return this;
  }

  displayEpoch() {
    return this._displayEpoch;
  }
  setDisplayEpoch(displayEpoch) {
    this._displayEpoch = displayEpoch;
    return this;
  }
}
