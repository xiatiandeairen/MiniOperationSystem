#!/bin/bash
# Captures trace data from MiniOS serial output.
# Usage: ./scripts/capture-trace.sh [output_file]
set -e

OUTPUT=${1:-"trace-export.json"}

echo "Starting MiniOS in QEMU with trace capture..."
echo "Type 'trace export' in the MiniOS shell, then Ctrl+C to stop."
echo ""

# Build and create image
cargo make build-release 2>/dev/null
./tools/boot-image/target/x86_64-unknown-linux-gnu/release/boot-image \
    target/x86_64-unknown-none/release/minios-kernel 2>/dev/null

# Run QEMU, capture serial to file
timeout 60 qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/minios-bios.img \
    -serial file:serial-raw.log \
    -display none \
    -m 256M \
    -no-reboot \
    -no-shutdown 2>/dev/null || true

# Extract JSON between markers
if grep -q "TRACE-BEGIN" serial-raw.log 2>/dev/null; then
    sed -n '/---TRACE-BEGIN---/,/---TRACE-END---/p' serial-raw.log \
        | grep -v "TRACE-" \
        > "$OUTPUT"
    echo "Trace data saved to: $OUTPUT"
    echo "Open trace-viewer/index.html and load this file."
else
    echo "No trace data found in serial output."
    echo "Make sure to run 'trace export' in the MiniOS shell."
fi

rm -f serial-raw.log
