#!/bin/bash
# Smoke tests for hsmm - quick validation that binary works

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Convert to absolute path
BINARY="${HSMM_BINARY:-./target/release/hsmm}"
BINARY=$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")
FAILURES=0

echo "=========================================="
echo "  hsmm Smoke Tests"
echo "=========================================="
echo ""

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
}

fail() {
    echo -e "${RED}✗${NC} $1"
    FAILURES=$((FAILURES + 1))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

test_binary_exists() {
    echo -n "Binary exists: "
    if [ -f "$BINARY" ]; then
        pass "$BINARY"
    else
        fail "$BINARY not found"
        echo ""
        echo "Build it first: cargo build --release"
        exit 1
    fi
}

test_binary_executable() {
    echo -n "Binary is executable: "
    if [ -x "$BINARY" ]; then
        pass "yes"
    else
        fail "not executable"
        echo "Fix: chmod +x $BINARY"
    fi
}

test_help_command() {
    echo -n "Help command: "
    if $BINARY --help > /dev/null 2>&1; then
        pass "works"
    else
        fail "failed"
    fi
}

test_version_command() {
    echo -n "Version command: "
    VERSION=$($BINARY --version 2>&1 || echo "ERROR")
    if [[ "$VERSION" =~ ^hsmm\ [0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        pass "$VERSION"
    else
        fail "unexpected output: $VERSION"
    fi
}

test_subcommands_exist() {
    echo -n "Subcommands present: "
    HELP_OUTPUT=$($BINARY --help 2>&1)

    EXPECTED_COMMANDS=("add" "remove" "list" "upgrade" "config")
    MISSING=()

    for cmd in "${EXPECTED_COMMANDS[@]}"; do
        if ! echo "$HELP_OUTPUT" | grep -q "^  $cmd"; then
            MISSING+=("$cmd")
        fi
    done

    if [ ${#MISSING[@]} -eq 0 ]; then
        pass "all found (${EXPECTED_COMMANDS[*]})"
    else
        fail "missing: ${MISSING[*]}"
    fi
}

test_no_crash_on_invalid_args() {
    echo -n "Handles invalid args: "
    if $BINARY --invalid-flag 2>&1 | grep -q "error:"; then
        pass "shows error message"
    else
        fail "unexpected behavior"
    fi
}

test_temp_workspace() {
    echo -n "Can create config: "

    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Run list command (may return non-zero with no mods, that's OK)
    $BINARY list > /dev/null 2>&1 || true

    if [ -f "mods.toml" ]; then
        pass "mods.toml created"
    else
        fail "config not created"
    fi

    cd - > /dev/null
    rm -rf "$TEMP_DIR"
}

test_add_mod_cli() {
    echo -n "Can add mod: "

    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Add command should succeed and show success message
    ADD_OUTPUT=$($BINARY add "Test Mod" 123456 2>&1 || echo "FAILED")

    if echo "$ADD_OUTPUT" | grep -q "Added mod"; then
        if grep -q "Test Mod" mods.toml; then
            pass "mod added to config"
        else
            fail "mod not in config"
        fi
    else
        fail "add command failed: $ADD_OUTPUT"
    fi

    cd - > /dev/null
    rm -rf "$TEMP_DIR"
}

test_list_mod_cli() {
    echo -n "Can list mods: "

    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    $BINARY add "Test Mod" 123456 > /dev/null 2>&1 || true

    LIST_OUTPUT=$($BINARY list 2>&1 || true)

    if echo "$LIST_OUTPUT" | grep -q "Test Mod"; then
        pass "mod listed correctly"
    else
        fail "list output incorrect"
    fi

    cd - > /dev/null
    rm -rf "$TEMP_DIR"
}

test_config_command() {
    echo -n "Config command works: "

    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    CONFIG_OUTPUT=$($BINARY config 2>&1 || true)

    if echo "$CONFIG_OUTPUT" | grep -q "Configuration:"; then
        pass "shows configuration"
    else
        fail "config command failed"
    fi

    cd - > /dev/null
    rm -rf "$TEMP_DIR"
}

# Run all tests
test_binary_exists
test_binary_executable
test_help_command
test_version_command
test_subcommands_exist
test_no_crash_on_invalid_args
test_temp_workspace
test_add_mod_cli
test_list_mod_cli
test_config_command

echo ""
echo "=========================================="

if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}All smoke tests passed!${NC}"
    echo "=========================================="
    exit 0
else
    echo -e "${RED}$FAILURES test(s) failed${NC}"
    echo "=========================================="
    exit 1
fi
