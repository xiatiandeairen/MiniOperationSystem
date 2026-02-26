#!/bin/bash
# Release quality gate — run before tagging a new version.
# Usage: ./scripts/release-check.sh v0.X.0
set -e

VERSION=${1:?"Usage: $0 <version>"}
echo "=== Release Check: $VERSION ==="
echo ""

PASS=0
FAIL=0

check() {
    if eval "$2" > /dev/null 2>&1; then
        echo "  ✅ $1"
        PASS=$((PASS + 1))
    else
        echo "  ❌ $1"
        FAIL=$((FAIL + 1))
    fi
}

echo "--- Code Quality ---"
check "cargo fmt" "cargo fmt --all --check"
check "cargo clippy (0 warnings)" "cargo clippy --workspace -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem -- -D warnings"
check "debug build" "cargo build --workspace -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem"
check "release build" "cargo build --workspace --release -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem"

echo ""
echo "--- Tests ---"
check "unit tests (>=59)" "test \$(cargo test -p minios-common -p minios-trace -p minios-scheduler -p minios-ipc --target x86_64-unknown-linux-gnu 2>&1 | grep -oP '\d+ passed' | awk '{s+=\$1} END {print s}') -ge 59"

echo ""
echo "--- Documentation ---"
check "release notes exist" "test -f docs/releases/${VERSION}.md"
check "changelog exists" "test -f docs/changelogs/${VERSION}-changelog.md"
check "tasks doc exists" "test -f docs/dev-logs/${VERSION}-tasks.md"
check "insights doc exists" "test -f docs/dev-logs/${VERSION}-insights.md"

echo ""
echo "--- Boot Image ---"
check "boot-image tool built" "test -f tools/boot-image/target/x86_64-unknown-linux-gnu/release/boot-image"
check "kernel ELF exists" "test -f target/x86_64-unknown-none/release/minios-kernel"

echo ""
echo "=== Result: $PASS passed, $FAIL failed ==="
if [ $FAIL -gt 0 ]; then
    echo "❌ RELEASE BLOCKED — fix failures before tagging $VERSION"
    exit 1
else
    echo "✅ All checks pass — safe to release $VERSION"
fi
