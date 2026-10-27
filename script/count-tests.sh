#!/bin/sh
# Prints how many unit, integration and e2e tests the project has.
# Each test is a `#[test]` attribute:
#   unit        -> #[test] in src/**/*.rs, except modules named test_*_integration
#   integration -> #[test] in src/**/*.rs modules named test_*_integration,
#                  and in tests/*.rs files that are not e2e files
#   e2e         -> #[test] in tests/e2e*.rs

# Counts #[test] attributes in src/ files, split into unit and integration.
# An integration module is one whose name ends in `_integration`.
count_src() {
  find src -name '*.rs' -exec awk '
    /^[ \t]*mod [A-Za-z0-9_]+ *\{/ {
      name = $2; sub(/\{.*/, "", name)
      in_integration = (name ~ /_integration$/)
    }
    /#\[test\]/ {
      if (in_integration) integration++; else unit++
    }
    END { printf "%d %d\n", unit + 0, integration + 0 }
  ' {} \; | awk '{u += $1; i += $2} END {printf "%d %d\n", u + 0, i + 0}'
}

# Counts #[test] attributes in the given files. Prints 0 when there are none.
count() {
  # With no files, grep would read stdin and wait forever, so return 0 early.
  if [ "$#" -eq 0 ]; then
    echo 0
    return
  fi
  grep -h "#\[test\]" "$@" | wc -l | tr -d ' '
}

set -- $(count_src)
unit=$1
src_integration=$2
tests_integration=$(count $(find tests -maxdepth 1 -name '*.rs' ! -name 'e2e*'))
integration=$((src_integration + tests_integration))
e2e=$(count $(find tests -maxdepth 1 -name 'e2e*.rs'))

echo "🧪 Unit tests:        $unit"
echo "🔗 Integration tests: $integration"
echo "🎬 E2E tests:         $e2e"
echo "   Total:             $((unit + integration + e2e))"
