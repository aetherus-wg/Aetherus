#!/usr/bin/env bash
# Used to stash work before running underlying command
# From: https://github.com/KShivendu/smoldb/blob/c50fac26043601287e614223a63289a0d63a10b6/tools/stash-hook.sh

stash_unstaged_changes() {
  # Check if there are unstaged changes
  # NOTE: Add `--ignore-submodules`?
  # Add check condition: ` || [ -n "$(git ls-files --others --exclude-standard)" ]`?
  STASHED=0
  if ! git diff --quiet; then
      echo "🗃️  Stashing unstaged changes..."
      git stash push \
        -m "hook temporary stash" \
        --keep-index \
        --include-untracked

      STASHED=1
  fi
}

# Function to restore stash on exit
restore_stash() {
    if [ "${STASHED:-0}" -eq 1 ]; then
        echo "📦 Restoring stashed changes..."
        if git stash list | grep -q "hook temporary stash"; then
            git stash pop --index
        else
            echo "⚠️  Warning: Could not find stash to restore"
        fi
    fi
}
