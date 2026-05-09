#!/usr/bin/env bash
set -euo pipefail

MODULE_MAX_LINES="${MODULE_MAX_LINES:-500}"

find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} + \
  | awk -v max="$MODULE_MAX_LINES" '$2 != "total" && $1 > max { print; bad=1 } END { exit bad }'
