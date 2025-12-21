/**
 * Default configuration for the application
 * Can be overridden by setting window.__defaultConfig before initialization
 * @returns {Object} Default configuration object
 */
export const getDefaultConfig = () => {
  return window.__defaultConfig || {
    clocks: [
      { name: 'UTC', timezone: 'UTC' },
      { name: 'JST', timezone: 'Asia/Tokyo' }
    ],
    epochClockName: 'Epoch',
    format: 'HH:mm:ss',
    timerIcon: '⏱',
    withoutNotification: false,
    maxTimerClockNumber: 10,
    usetz: false,
    convtz: null,
    disableHover: false,
    forefront: false,
    font: 'Courier, monospace',
    color: '#fff',
    size: '14px',
    locale: 'en',
    margin: '10px'
  };
};

