#!/bin/bash
# Hotspot risk analysis — run at each checkpoint.
set -euo pipefail

echo "=== Change Frequency Top 20 (3 months) ==="
git log --since="3 months" --name-only --pretty=format: -- 'crates/**/*.rs' \
  | grep -v '^$' | sort | uniq -c | sort -rn | head -20

echo ""
echo "=== Bug-Fix Hotspots ==="
git log --since="3 months" --name-only --pretty=format: --grep="^fix" -- 'crates/**/*.rs' \
  | grep -v '^$' | sort | uniq -c | sort -rn | head -20

echo ""
echo "=== File Size Top 20 (LOC) ==="
find crates -name '*.rs' -exec wc -l {} + 2>/dev/null | sort -rn | head -20

echo ""
echo "=== Multi-Author Files ==="
for f in $(find crates -name '*.rs' 2>/dev/null); do
  authors=$(git shortlog -s -- "$f" 2>/dev/null | wc -l)
  if [ "$authors" -gt 2 ]; then
    echo "  $authors authors: $f"
  fi
done
echo "(done)"
