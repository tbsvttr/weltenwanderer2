#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIB_RS="$REPO_ROOT/crates/ww-dsl/src/lib.rs"
TEMPLATE="$REPO_ROOT/doc/README.tmpl.md"
OUTPUT="$REPO_ROOT/README.md"

# Extract //! doc comments from the top of lib.rs, stripping the prefix.
# Stops at the first line that is not a //! comment.
extract_docs() {
    while IFS= read -r line; do
        case "$line" in
            '//! '*)  printf '%s\n' "${line#//! }" ;;
            '//!'*)   printf '%s\n' "${line#//!}" ;;
            *)        break ;;
        esac
    done < "$LIB_RS"
}

# Read template line by line; substitute {{DSL_REFERENCE}} with extracted docs.
{
    while IFS= read -r line; do
        if [ "$line" = "{{DSL_REFERENCE}}" ]; then
            extract_docs
        else
            printf '%s\n' "$line"
        fi
    done < "$TEMPLATE"
} > "$OUTPUT"
