export class Clocks {
  #clocks = [];
  #timerClocks = [];

  constructor(clocks, epochClockName) {
    const epochClock = {
      name: epochClockName ?? "Epoch",
      timezone: "UTC",
      isEpoch: true,
    };

    this.#clocks = [...clocks, epochClock];
  }

  pushTimerClock(timerClock) {
    this.#timerClocks.push(timerClock);
    return this;
  }

  getClocks() {
    return this.#clocks;
  }

  getAllClocks() {
    return [...this.#clocks, ...this.#timerClocks];
  }

  getTimerClocks() {
    return this.#timerClocks;
  }

  removeTimerRight() {
    const timerClock = this.#timerClocks.pop();
    this.#cleanupTimer(timerClock);
    return this;
  }

  removeTimerLeft() {
    const timerClock = this.#timerClocks.shift();
    this.#cleanupTimer(timerClock);
    return this;
  }

  /**
   * @param {Object} timerClock - Timer clock to cleanup
   */
  #cleanupTimer(timerClock) {
    clearTimeout(timerClock.timeoutId);
    document.getElementById(timerClock.id).remove();
  }
}
