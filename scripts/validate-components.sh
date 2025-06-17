#!/usr/bin/env bash
set -e
components=(ClipboardSync ConnectionManager FileTransfer RemoteScreen)
status=0
for comp in "${components[@]}"; do
  [[ -f "src/components/${comp}.tsx" ]] || { echo "missing source for $comp"; status=1; }
  if [[ ! -f "tests/unit/${comp}.test.tsx" && ! -f "tests/unit/${comp}.test.ts" ]]; then
    echo "missing tests for $comp"; status=1;
  fi
  [[ -f "docs/docs/components/${comp}.md" ]] || { echo "missing docs for $comp"; status=1; }
  [[ -f "src/components/${comp}.demo.tsx" ]] || { echo "missing demo for $comp"; status=1; }
done
exit $status
