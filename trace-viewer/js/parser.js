/**
 * Parse a MiniOS trace JSON string into a structured object.
 *
 * @param {string} jsonString  Raw JSON text
 * @returns {{ spans: Array, metadata: Object, tree: Array }}
 */
function parseTraceData(jsonString) {
  const raw = JSON.parse(jsonString);

  if (raw.format !== 'minios-trace-v1') {
    throw new Error('Unsupported trace format: ' + (raw.format || 'unknown'));
  }

  const freqHz = raw.tsc_frequency_hz || 2_400_000_000;

  const spans = (raw.spans || []).map(s => ({
    traceId:      s.trace_id,
    spanId:       s.span_id,
    parentSpanId: s.parent_span_id || null,
    name:         s.name,
    module:       s.module,
    startTsc:     s.start_tsc,
    endTsc:       s.end_tsc,
    durationNs:   s.duration_ns != null
                    ? s.duration_ns
                    : ((s.end_tsc - s.start_tsc) / freqHz) * 1e9,
    status:       s.status || 'unknown',
    pid:          s.pid != null ? s.pid : -1,
    attributes:   s.attributes || {},
    children:     [],
    depth:        0,
    collapsed:    false,
    visible:      true,
  }));

  const spanMap = new Map();
  spans.forEach(s => spanMap.set(s.spanId, s));

  const roots = [];
  spans.forEach(s => {
    if (s.parentSpanId && spanMap.has(s.parentSpanId)) {
      spanMap.get(s.parentSpanId).children.push(s);
    } else {
      roots.push(s);
    }
  });

  function setDepth(span, depth) {
    span.depth = depth;
    span.children.sort((a, b) => a.startTsc - b.startTsc);
    span.children.forEach(c => setDepth(c, depth + 1));
  }
  roots.forEach(r => setDepth(r, 0));

  const globalStart = spans.length ? Math.min(...spans.map(s => s.startTsc)) : 0;
  const globalEnd   = spans.length ? Math.max(...spans.map(s => s.endTsc))   : 0;

  const metadata = {
    format:       raw.format,
    exportedAt:   raw.exported_at,
    freqHz,
    spanCount:    spans.length,
    globalStart,
    globalEnd,
    totalDurationNs: ((globalEnd - globalStart) / freqHz) * 1e9,
  };

  return { spans, metadata, tree: roots };
}
