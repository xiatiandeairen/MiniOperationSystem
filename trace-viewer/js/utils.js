/**
 * Module color palette for trace visualization.
 * Each module maps to an HSL color used in the waterfall chart.
 */
const MODULE_COLORS = {
  boot:      '#7aa2f7',
  hal:       '#9ece6a',
  trace:     '#89dceb',
  memory:    '#a6e3a1',
  interrupt: '#f9e2af',
  process:   '#fab387',
  scheduler: '#cba6f7',
  fs:        '#f38ba8',
  ipc:       '#eba0ac',
  syscall:   '#94e2d5',
  shell:     '#f5c2e7',
  driver:    '#b4befe',
};

const MODULE_COLOR_DEFAULT = '#cdd6f4';

/**
 * Return the color string for a given module name.
 */
function getModuleColor(mod) {
  return MODULE_COLORS[mod] || MODULE_COLOR_DEFAULT;
}

/**
 * Format a nanosecond duration into a human-friendly string.
 *   < 1 000 ns        →  "123 ns"
 *   1 000 – 999 999   →  "12.3 µs"
 *   >= 1 000 000      →  "1.23 ms"
 */
function formatDuration(ns) {
  if (ns < 1000) return ns.toFixed(0) + ' ns';
  if (ns < 1_000_000) return (ns / 1000).toFixed(1) + ' µs';
  return (ns / 1_000_000).toFixed(2) + ' ms';
}

/**
 * Format a TSC value relative to a base TSC into a time offset string.
 */
function formatTscOffset(tsc, baseTsc, freqHz) {
  const ns = ((tsc - baseTsc) / freqHz) * 1e9;
  return formatDuration(ns);
}

/**
 * Clamp a number between min and max.
 */
function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}
