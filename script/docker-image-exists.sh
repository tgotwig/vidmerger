#!/bin/sh
# Usage: docker-image-exists.sh <IMAGE:TAG>
# Exit 0 if the image is in the registry, 1 if it is confirmed missing, 2 if the registry check keeps failing
image="$1"
for try in 1 2 3; do
  if out=$(docker buildx imagetools inspect "$image" 2>&1); then
    exit 0
  fi
  if printf '%s\n' "$out" | grep -q ": not found$"; then
    exit 1
  fi
  echo "Check of $image failed (try $try): $out" >&2
  [ "$try" -lt 3 ] && sleep 10
done
echo "Cannot check if $image is in the registry" >&2
exit 2
