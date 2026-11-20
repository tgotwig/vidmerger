#!/bin/sh
# Usage: is-newest-version.sh <TAG> [REMOTE]
# Exit 0 if TAG is the highest x.y.z tag on REMOTE (default: origin), 1 if not, 2 if the tags cannot be read
tag="$1"
remote="${2:-origin}"
if ! tags=$(git ls-remote --tags --refs "$remote"); then
  echo "Cannot read tags from $remote" >&2
  exit 2
fi
newest=$(printf '%s\n' "$tags" | sed 's#.*refs/tags/##' | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' | sort -t. -k1,1n -k2,2n -k3,3n | tail -n 1)
echo "Newest version tag on $remote: ${newest:-none}"
[ "$tag" = "$newest" ]
