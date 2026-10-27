#!/bin/sh
# Prints how many unit, integration and e2e tests the project has.
# Each test is a `#[test]` attribute:
#   unit        -> src/**/*.rs
#   integration -> tests/*.rs that are not e2e files
#   e2e         -> tests/e2e*.rs

count() {
  # With no files, grep would read stdin and wait forever, so return 0 early.
  if [ "$#" -eq 0 ]; then
    echo 0
    return
  fi
  grep -h "#\[test\]" "$@" | wc -l | tr -d ' '
}

unit=$(count $(find src -name '*.rs'))
integration=$(count $(find tests -maxdepth 1 -name '*.rs' ! -name 'e2e*'))
e2e=$(count $(find tests -maxdepth 1 -name 'e2e*.rs'))

echo "🧪 Unit tests:        $unit"
echo "🔗 Integration tests: $integration"
echo "🎬 E2E tests:         $e2e"
echo "   Total:             $((unit + integration + e2e))"
