import { platform } from '@tauri-apps/plugin-os';
import { cdate } from 'cdate';

export class Ctx {
  #mainElement;
  #cdateUTC;
  #ignoreOnMoved = false;
  #pauseTimer = false;
  #lockKeyP = false;
  #displayEpoch = false;
  #format;
  #timerIcon;
  #withoutNotification;
  #maxTimerClockNumber;
  #usetz = false; // If true, use TZ to convert epoch time instead of utcOffset
  #convtz;

  constructor(mainElement) {
    this.#mainElement = mainElement;
    this.#cdateUTC = cdate().tz("UTC").cdateFn();
  }

  isMacOS() {
    return platform().toLowerCase() === 'macos';
  }

  cdateUTC(cdt) {
    return this.#cdateUTC(cdt);
  }

  mainElement() {
    return this.#mainElement;
  }

  ignoreOnMoved() {
    return this.#ignoreOnMoved;
  }
  setIgnoreOnMoved(ignoreOnMoved) {
    this.#ignoreOnMoved = ignoreOnMoved;
    return this;
  }

  format() {
    return this.#format;
  }
  setFormat(format) {
    this.#format = format;
    return this;
  }

  pauseTimer() {
    return this.#pauseTimer;
  }
  setPauseTimer(pauseTimer) {
    this.#pauseTimer = pauseTimer;
    return this;
  }

  lockKeyP() {
    return this.#lockKeyP;
  }
  setLockKeyP(lockKeyP) {
    this.#lockKeyP = lockKeyP;
    return this;
  }

  timerIcon() {
    return this.#timerIcon;
  }
  setTimerIcon(timerIcon) {
    this.#timerIcon = timerIcon;
    return this;
  }

  withoutNotification() {
    return this.#withoutNotification;
  }
  setWithoutNotification(withoutNotification) {
    this.#withoutNotification = withoutNotification;
    return this;
  }

  maxTimerClockNumber() {
    return this.#maxTimerClockNumber;
  }
  setMaxTimerClockNumber(maxTimerClockNumber) {
    this.#maxTimerClockNumber = maxTimerClockNumber;
    return this;
  }

  displayEpoch() {
    return this.#displayEpoch;
  }
  setDisplayEpoch(displayEpoch) {
    this.#displayEpoch = displayEpoch;
    return this;
  }

  useTZ() {
    return this.#usetz;
  }
  setUseTZ(usetz) {
    this.#usetz = usetz;
    return this;
  }

  convTZ() {
    return this.#convtz;
  }
  setConvTZ(convtz) {
    this.#convtz = convtz;
    return this;
  }
}
