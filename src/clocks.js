export class Clocks {
  constructor(clocks, epochClockName) {
    clocks.push({
      name: epochClockName ?? "Epoch",
      timezone: "UTC",
      isEpoch: true,
    });

    this._clocks = clocks;
    this._timerClocks = [];
  }

  pushTimerClock(timerClock) {
    this._timerClocks.push(timerClock);

    return this;
  }

  getClocks() {
    return this._clocks;
  }

  getAllClocks() {
    return [...this._clocks, ...this._timerClocks];
  }

  getTimerClocks() {
    return this._timerClocks;
  }

  removeTimerRight() {
    const timerClock = this._timerClocks.pop();
    clearTimeout(timerClock.timeoutId);
    document.getElementById(timerClock.id).remove();
    return this;
  }

  removeTimerLeft() {
    const timerClock = this._timerClocks.shift();
    clearTimeout(timerClock.timeoutId);
    document.getElementById(timerClock.id).remove();
    return this;
  }
}
