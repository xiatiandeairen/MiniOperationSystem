function renderDashboard() {
    const input = document.getElementById('dashboard-input').value;
    const output = document.getElementById('dashboard-output');
    const lines = input.trim().split('\n');
    const data = {};
    lines.forEach(line => {
        const [key, val] = line.split(': ');
        if (key && val) data[key.trim()] = val.trim();
    });

    const uptimeSec = Math.floor((parseInt(data.uptime_ticks) || 0) / 100);
    const memUsed = (parseInt(data.memory_frames_total) || 0) - (parseInt(data.memory_frames_free) || 0);
    const memTotal = parseInt(data.memory_frames_total) || 1;
    const memPct = Math.round(memUsed * 100 / memTotal);

    output.innerHTML = `
        <div class="dashboard-grid">
            <div class="metric">
                <h3>Uptime</h3>
                <span class="value">${uptimeSec}s</span>
            </div>
            <div class="metric">
                <h3>Memory</h3>
                <div class="bar" style="width:${memPct}%"></div>
                <span class="value">${memPct}% used (${memUsed}/${memTotal} frames)</span>
            </div>
            <div class="metric">
                <h3>Heap</h3>
                <span class="value">${data.heap_used || 0} used / ${data.heap_free || 0} free</span>
            </div>
            <div class="metric">
                <h3>Scheduler</h3>
                <span class="value">${data.scheduler_switches || 0} switches in ${data.scheduler_ticks || 0} ticks</span>
            </div>
            <div class="metric">
                <h3>Interrupts</h3>
                <span class="value">Timer: ${data.irq_timer || 0} | Keyboard: ${data.irq_keyboard || 0}</span>
            </div>
        </div>
    `;
}
