#!/usr/bin/env bash
set -e
components=(Button ClipboardSync ConnectionManager FileTransfer RemoteScreen)
status=0
for comp in "${components[@]}"; do
  story="src/components/${comp}.stories.tsx"
  snap="tests/unit/components/${comp}.test.tsx"
  a11y="tests/unit/components/${comp}.a11y.test.tsx"
  [[ -f "$story" ]] || { echo "missing story for $comp"; status=1; }
  if [[ ! -f "$snap" ]] || ! grep -q composeStory "$snap"; then
    echo "missing snapshot test for $comp"; status=1
  fi
  [[ -f "$a11y" ]] || { echo "missing a11y test for $comp"; status=1; }
  if ! grep -q axe "$a11y"; then
    echo "a11y test does not use axe for $comp"; status=1
  fi
done
exit $status
