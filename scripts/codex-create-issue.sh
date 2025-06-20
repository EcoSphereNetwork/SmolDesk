#!/bin/bash
# usage: ./codex-create-issue.sh "Title" "Body"

gh issue create --title "$1" --body "$2" --label "conflict"
