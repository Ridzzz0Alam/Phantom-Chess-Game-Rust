#!/usr/bin/env sh
# Prints the target line of every `file#L<n>` link in README.md so you can see
# at a glance whether each link still points at the right code.
# Usage: sh scripts/check-readme-links.sh
cd "$(dirname "$0")/.." || exit 1
grep -oE '\]\(([a-z-]+/[^)#]+)#L([0-9]+)\)' README.md \
  | sed -E 's/\]\((.*)#L([0-9]+)\)/\1 \2/' | sort -u \
  | while read -r file line; do
      printf '%-34s L%-4s %s\n' "$file" "$line" "$(sed -n "${line}p" "$file" | sed 's/^ *//' | cut -c1-70)"
    done
