#!/bin/bash
set -e

echo "==================================="
echo "  STRICT TDD WORKFLOW (SLOP-LESS)  "
echo "==================================="

echo ""
echo "[1/3] Checking formatting (cargo fmt)..."
if ! cargo fmt --check; then
  echo "Formatting failed. Run 'cargo fmt' to fix."
  exit 1
fi

echo ""
echo "[2/3] Checking lints (cargo clippy)..."
if ! cargo clippy; then
  echo "Clippy failed. Fix the warnings to proceed."
  exit 1
fi

echo ""
echo "[3/3] Running tests (cargo test)..."
# In true TDD fashion, the tests will fail until the compiler is complete.
# To make it usable iteratively, you can run specific tests like:
# cargo test --test official_suite -- <test_name>
if ! cargo test; then
  echo "Tests failed! Red -> Green -> Refactor."
  exit 1
fi

echo ""
echo "All checks passed! Great job!"
