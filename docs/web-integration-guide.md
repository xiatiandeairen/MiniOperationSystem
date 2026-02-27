# MiniOS Web Integration Guide

## Trace Viewer

1. Boot MiniOS: `cargo make run-gui`
2. In the shell: `trace export`
3. Capture serial: `cargo make run-trace` then `scripts/capture-trace.sh`
4. Open `trace-viewer/index.html`
5. Load the captured JSON file

## Dashboard

1. In the shell: `cat /proc/dashboard`
2. Copy the output
3. Open `trace-viewer/index.html` → Dashboard tab
4. Paste the output → click Render

## API Format

### /proc/dashboard (key-value pairs)
```
uptime_ticks: 5000
memory_frames_free: 63000
memory_frames_total: 65456
heap_used: 1024
heap_free: 1047552
scheduler_switches: 150
scheduler_ticks: 5000
irq_timer: 5000
irq_keyboard: 42
```

### trace export (JSON)
```json
{"format":"minios-trace-v1","spans":[...]}
```
