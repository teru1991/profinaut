#!/usr/bin/env bash
set -euo pipefail

# artifact_guard.sh
#
# Detect generated artifacts accidentally committed in a PR diff.
#
# Usage:
#   scripts/security/artifact_guard.sh path1 path2 ...
#   git diff --name-only origin/main...HEAD | scripts/security/artifact_guard.sh

read_changed_files() {
  if [[ "$#" -gt 0 ]]; then
    printf '%s\n' "$@"
  else
    cat
  fi
}

is_forbidden_path() {
  local path="$1"
  local forbidden_regex='(^|/)(node_modules|\.next|dist|__pycache__)/|\.pyc$'

  if [[ "$path" =~ $forbidden_regex ]]; then
    return 0
  fi

  return 1
}

main() {
  local forbidden_files=()
  local changed_file=""
  local changed_count=0

  while IFS= read -r changed_file; do
    [[ -z "$changed_file" ]] && continue
    changed_count=$((changed_count + 1))
    if is_forbidden_path "$changed_file"; then
      forbidden_files+=("$changed_file")
    fi
  done < <(read_changed_files "$@")

  if [[ "$changed_count" -eq 0 ]]; then
    echo "✅ Artifact guard passed: no changed files detected (nothing to validate)."
    return 0
  fi

  if [[ "${#forbidden_files[@]}" -gt 0 ]]; then
    echo "❌ Generated artifact files are not allowed in PR commits."
    echo "Found forbidden paths:"
    printf ' - %s\n' "${forbidden_files[@]}"
    echo ""
    echo "Please remove generated artifacts from the commit and push again."
    return 1
  fi

  echo "✅ Artifact guard passed: no forbidden generated artifacts found in changed files."
}

main "$@"
