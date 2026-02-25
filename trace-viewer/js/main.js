/**
 * Entry point – wire up file loading, tabs, and initial render.
 */

let chart      = null;
let traceData  = null;

/* ---- DOM refs ---- */
const dropZone   = document.getElementById('drop-zone');
const fileInput  = document.getElementById('file-input');
const tabBar     = document.getElementById('tab-bar');
const waterfall  = document.getElementById('waterfall-panel');
const statsPanel = document.getElementById('stats-panel');
const canvas     = document.getElementById('waterfall-canvas');
const tooltip    = document.getElementById('tooltip');
const statsBody  = document.getElementById('stats-body');
const statusMsg  = document.getElementById('status-msg');

/* ---- tabs ---- */
tabBar.addEventListener('click', e => {
  const btn = e.target.closest('[data-tab]');
  if (!btn) return;
  tabBar.querySelectorAll('button').forEach(b => b.classList.remove('active'));
  btn.classList.add('active');
  const tab = btn.dataset.tab;
  waterfall.style.display  = tab === 'waterfall' ? 'block' : 'none';
  statsPanel.style.display = tab === 'stats'     ? 'block' : 'none';
  if (tab === 'waterfall' && chart) chart.render();
});

/* ---- file loading ---- */
function handleFile(file) {
  const reader = new FileReader();
  reader.onload = () => loadJson(reader.result);
  reader.readAsText(file);
}

dropZone.addEventListener('dragover', e => { e.preventDefault(); dropZone.classList.add('drag-over'); });
dropZone.addEventListener('dragleave', () => dropZone.classList.remove('drag-over'));
dropZone.addEventListener('drop', e => {
  e.preventDefault();
  dropZone.classList.remove('drag-over');
  if (e.dataTransfer.files.length) handleFile(e.dataTransfer.files[0]);
});
fileInput.addEventListener('change', () => { if (fileInput.files.length) handleFile(fileInput.files[0]); });

/* ---- core load ---- */
function loadJson(jsonString) {
  try {
    traceData = parseTraceData(jsonString);
  } catch (err) {
    statusMsg.textContent = 'Parse error: ' + err.message;
    return;
  }

  traceData.metadata._tree = traceData.tree;

  statusMsg.textContent = `Loaded ${traceData.metadata.spanCount} spans — total ${formatDuration(traceData.metadata.totalDurationNs)}`;

  if (!chart) chart = new WaterfallChart(canvas, tooltip);
  chart.load(traceData.tree, traceData.spans, traceData.metadata);

  renderStats();
}

function renderStats() {
  if (!traceData) return;
  const m = traceData.metadata;
  const spans = traceData.spans;

  const moduleCounts = {};
  const moduleDurations = {};
  spans.forEach(s => {
    moduleCounts[s.module] = (moduleCounts[s.module] || 0) + 1;
    moduleDurations[s.module] = (moduleDurations[s.module] || 0) + s.durationNs;
  });

  let html = `
    <div class="stat-card">
      <h3>Overview</h3>
      <table>
        <tr><td>Total spans</td><td>${m.spanCount}</td></tr>
        <tr><td>Total duration</td><td>${formatDuration(m.totalDurationNs)}</td></tr>
        <tr><td>TSC frequency</td><td>${(m.freqHz / 1e9).toFixed(1)} GHz</td></tr>
        <tr><td>Modules</td><td>${Object.keys(moduleCounts).length}</td></tr>
      </table>
    </div>
    <div class="stat-card">
      <h3>By Module</h3>
      <table>
        <tr><th>Module</th><th style="text-align:right">Spans</th><th style="text-align:right">Duration</th></tr>
  `;
  Object.keys(moduleCounts).sort().forEach(mod => {
    const color = getModuleColor(mod);
    html += `<tr>
      <td><span class="color-dot" style="background:${color}"></span>${mod}</td>
      <td style="text-align:right">${moduleCounts[mod]}</td>
      <td style="text-align:right">${formatDuration(moduleDurations[mod])}</td>
    </tr>`;
  });
  html += '</table></div>';

  const statusCounts = {};
  spans.forEach(s => { statusCounts[s.status] = (statusCounts[s.status] || 0) + 1; });
  html += '<div class="stat-card"><h3>By Status</h3><table>';
  Object.entries(statusCounts).forEach(([st, c]) => {
    html += `<tr><td>${st}</td><td style="text-align:right">${c}</td></tr>`;
  });
  html += '</table></div>';

  statsBody.innerHTML = html;
}

/* ---- auto-load sample ---- */
window.addEventListener('DOMContentLoaded', () => {
  fetch('sample/sample-trace.json')
    .then(r => { if (r.ok) return r.text(); throw new Error('not found'); })
    .then(loadJson)
    .catch(() => { statusMsg.textContent = 'Drop a trace JSON file to begin.'; });
});

window.addEventListener('resize', () => { if (chart) chart.render(); });
