/**
 * Waterfall / timeline chart rendered on a <canvas>.
 *
 * Spans are drawn as horizontal bars along a time axis (X) and stacked
 * vertically by hierarchy depth (Y).  Supports zoom (scroll-wheel),
 * pan (drag), hover tooltips and click-to-collapse.
 */

const ROW_HEIGHT   = 28;
const ROW_PAD      = 4;
const INDENT_PX    = 20;
const LABEL_LEFT   = 8;
const HEADER_H     = 32;
const MIN_BAR_W    = 2;

class WaterfallChart {
  constructor(canvas, tooltip) {
    this.canvas  = canvas;
    this.ctx     = canvas.getContext('2d');
    this.tooltip = tooltip;
    this.spans   = [];
    this.flat    = [];
    this.meta    = null;

    this.zoom    = 1;
    this.panX    = 0;
    this.dragging = false;
    this.dragStartX = 0;
    this.dragPanStart = 0;

    this._bindEvents();
  }

  load(tree, spans, metadata) {
    this.spans = spans;
    this.meta  = metadata;
    this.zoom  = 1;
    this.panX  = 0;
    this._flatten(tree);
    this.render();
  }

  /* ---- layout helpers ---- */

  _flatten(tree) {
    this.flat = [];
    const walk = (nodes) => {
      for (const s of nodes) {
        if (!s.visible) continue;
        this.flat.push(s);
        if (!s.collapsed) walk(s.children);
      }
    };
    walk(tree);
  }

  _tscToX(tsc) {
    const frac = (tsc - this.meta.globalStart) / (this.meta.globalEnd - this.meta.globalStart);
    const contentW = (this.canvas.width - INDENT_PX * 6) * this.zoom;
    return INDENT_PX * 3 + frac * contentW + this.panX;
  }

  _rowY(index) {
    return HEADER_H + index * (ROW_HEIGHT + ROW_PAD);
  }

  /* ---- rendering ---- */

  render() {
    const dpr = window.devicePixelRatio || 1;
    const rect = this.canvas.getBoundingClientRect();
    this.canvas.width  = rect.width * dpr;
    this.canvas.height = Math.max(rect.height, this._rowY(this.flat.length) + ROW_PAD + 8) * dpr;
    this.canvas.style.height = (this.canvas.height / dpr) + 'px';
    this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const ctx = this.ctx;
    const w = rect.width;
    const h = this.canvas.height / dpr;

    ctx.fillStyle = '#1e1e2e';
    ctx.fillRect(0, 0, w, h);

    this._drawTimeAxis(ctx, w);
    this._drawRows(ctx, w);
  }

  _drawTimeAxis(ctx, w) {
    ctx.fillStyle = '#313244';
    ctx.fillRect(0, 0, w, HEADER_H);

    const ticks = 10;
    ctx.fillStyle = '#a6adc8';
    ctx.font = '11px monospace';
    ctx.textAlign = 'center';

    for (let i = 0; i <= ticks; i++) {
      const frac = i / ticks;
      const tsc = this.meta.globalStart + frac * (this.meta.globalEnd - this.meta.globalStart);
      const x = this._tscToX(tsc);
      if (x < 0 || x > w) continue;

      ctx.beginPath();
      ctx.strokeStyle = '#45475a';
      ctx.moveTo(x, HEADER_H - 6);
      ctx.lineTo(x, HEADER_H);
      ctx.stroke();

      const ns = ((tsc - this.meta.globalStart) / this.meta.freqHz) * 1e9;
      ctx.fillText(formatDuration(ns), x, HEADER_H - 10);
    }
  }

  _drawRows(ctx, w) {
    this.flat.forEach((span, i) => {
      const y = this._rowY(i);
      const x0 = this._tscToX(span.startTsc);
      const x1 = this._tscToX(span.endTsc);
      const barW = Math.max(x1 - x0, MIN_BAR_W);

      const color = getModuleColor(span.module);

      ctx.fillStyle = color + '33';
      ctx.fillRect(0, y, w, ROW_HEIGHT);

      ctx.fillStyle = color;
      ctx.fillRect(x0, y + 2, barW, ROW_HEIGHT - 4);

      if (span.children.length > 0) {
        ctx.fillStyle = '#cdd6f4';
        ctx.font = '10px monospace';
        ctx.textAlign = 'left';
        const marker = span.collapsed ? '▸' : '▾';
        ctx.fillText(marker, LABEL_LEFT + span.depth * INDENT_PX, y + ROW_HEIGHT / 2 + 4);
      }

      ctx.fillStyle = '#cdd6f4';
      ctx.font = '12px monospace';
      ctx.textAlign = 'left';
      const label = `[${span.module}] ${span.name}  ${formatDuration(span.durationNs)}`;
      const labelX = Math.max(x0 + 4, LABEL_LEFT + span.depth * INDENT_PX + 14);
      ctx.fillText(label, labelX, y + ROW_HEIGHT / 2 + 4);
    });
  }

  /* ---- interaction ---- */

  _bindEvents() {
    this.canvas.addEventListener('wheel', e => {
      e.preventDefault();
      const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
      const rect = this.canvas.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;

      const oldZoom = this.zoom;
      this.zoom = clamp(this.zoom * factor, 0.5, 50);
      this.panX = mouseX - (mouseX - this.panX) * (this.zoom / oldZoom);
      this.render();
    }, { passive: false });

    this.canvas.addEventListener('mousedown', e => {
      this.dragging = true;
      this.dragStartX = e.clientX;
      this.dragPanStart = this.panX;
    });
    window.addEventListener('mousemove', e => {
      if (!this.dragging) {
        this._handleHover(e);
        return;
      }
      this.panX = this.dragPanStart + (e.clientX - this.dragStartX);
      this.render();
    });
    window.addEventListener('mouseup', () => { this.dragging = false; });

    this.canvas.addEventListener('click', e => {
      const rect = this.canvas.getBoundingClientRect();
      const y = e.clientY - rect.top;
      const idx = Math.floor((y - HEADER_H) / (ROW_HEIGHT + ROW_PAD));
      if (idx >= 0 && idx < this.flat.length) {
        const span = this.flat[idx];
        if (span.children.length > 0) {
          span.collapsed = !span.collapsed;
          this._flatten(this.meta._tree);
          this.render();
        }
      }
    });
  }

  _handleHover(e) {
    const rect = this.canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const idx = Math.floor((y - HEADER_H) / (ROW_HEIGHT + ROW_PAD));

    if (idx >= 0 && idx < this.flat.length) {
      const span = this.flat[idx];
      this.tooltip.style.display = 'block';
      this.tooltip.style.left = (e.clientX + 12) + 'px';
      this.tooltip.style.top  = (e.clientY + 12) + 'px';
      this.tooltip.innerHTML =
        `<strong>${span.name}</strong><br>` +
        `Module: ${span.module}<br>` +
        `Duration: ${formatDuration(span.durationNs)}<br>` +
        `Status: ${span.status}<br>` +
        `PID: ${span.pid}<br>` +
        `Span: ${span.spanId}`;
    } else {
      this.tooltip.style.display = 'none';
    }
  }
}
