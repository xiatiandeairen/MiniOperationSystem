#!/bin/bash
set -e
echo "=== MiniOS Smoke Test ==="
cargo build --workspace --release -Z "build-std=core,compiler_builtins,alloc" -Z "build-std-features=compiler-builtins-mem" 2>/dev/null
./tools/boot-image/target/x86_64-unknown-linux-gnu/release/boot-image target/x86_64-unknown-none/release/minios-kernel 2>/dev/null
timeout 30 qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/release/minios-bios.img -nographic -m 256M -no-reboot -no-shutdown > /tmp/smoke.log 2>&1 || true

PASS=0; FAIL=0
check() { if grep -q "$2" /tmp/smoke.log; then echo "  ✅ $1"; PASS=$((PASS+1)); else echo "  ❌ $1"; FAIL=$((FAIL+1)); fi; }
check "Kernel boots" "boot sequence started"
check "Memory init" "Memory:"
check "Filesystem init" "Filesystem"
check "Shell starts" "Shell started"
check "Interrupts enabled" "interrupts enabled"
echo "=== $PASS passed, $FAIL failed ==="
[ $FAIL -eq 0 ]
