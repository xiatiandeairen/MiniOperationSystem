#!/bin/bash
set -e
echo "=== MiniOS Integration Test ==="
echo "Building workspace..."
cargo build --workspace --release -Z "build-std=core,compiler_builtins,alloc" -Z "build-std-features=compiler-builtins-mem"
echo "Creating boot image..."
./tools/boot-image/target/x86_64-unknown-linux-gnu/release/boot-image target/x86_64-unknown-none/release/minios-kernel
echo "Booting in QEMU (15s timeout)..."
timeout 15 qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-unknown-none/release/minios-bios.img \
  -nographic -m 256M -no-reboot -no-shutdown 2>&1 | tee /tmp/minios-boot.log
echo ""
echo "=== Verifying output ==="
grep -q "Memory:" /tmp/minios-boot.log && echo "✅ Memory subsystem" || echo "❌ Memory"
grep -q "heap works" /tmp/minios-boot.log && echo "✅ Heap allocator" || echo "❌ Heap"
grep -q "Filesystem initialized" /tmp/minios-boot.log && echo "✅ Filesystem" || echo "❌ Filesystem"
grep -q "Syscall subsystem" /tmp/minios-boot.log && echo "✅ Syscall" || echo "❌ Syscall"
grep -q "IPC subsystem" /tmp/minios-boot.log && echo "✅ IPC" || echo "❌ IPC"
grep -q "Process" /tmp/minios-boot.log && echo "✅ Process manager" || echo "❌ Process"
grep -q "Shell started" /tmp/minios-boot.log && echo "✅ Shell" || echo "❌ Shell"
grep -q "tick" /tmp/minios-boot.log && echo "✅ Timer interrupts" || echo "❌ Timer"
echo "=== Done ==="
