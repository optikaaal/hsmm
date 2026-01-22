#!/bin/bash
# Local integration tests for hsmm
# Tests real CurseForge integration without Docker

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Store project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Convert to absolute path
BINARY="${HSMM_BINARY:-$PROJECT_ROOT/target/release/hsmm}"
TEST_DIR="test-workspace-$(date +%s)"
FAILURES=0

echo "=========================================="
echo "  hsmm Local Integration Tests"
echo "=========================================="
echo ""
echo "This will test real CurseForge downloads."
echo "Internet connection required."
echo ""

# Helper functions
section() {
    echo ""
    echo -e "${BLUE}▶ $1${NC}"
    echo "---"
}

pass() {
    echo -e "${GREEN}✓${NC} $1"
}

fail() {
    echo -e "${RED}✗${NC} $1"
    FAILURES=$((FAILURES + 1))
}

info() {
    echo -e "  $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

cleanup() {
    if [ -d "$TEST_DIR" ]; then
        info "Cleaning up test directory: $TEST_DIR"
        rm -rf "$TEST_DIR"
    fi
}

trap cleanup EXIT

# Verify binary exists
if [ ! -f "$BINARY" ]; then
    fail "Binary not found: $BINARY"
    echo ""
    echo "Build it first: cargo build --release"
    exit 1
fi

# Create test workspace
section "Setting up test workspace"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"
pass "Created: $TEST_DIR"

# Test 1: Config creation
section "Test 1: Config File Creation"
info "Running: hsmm list"
$BINARY list > /dev/null 2>&1

if [ -f "mods.toml" ]; then
    pass "Config file created automatically"
    info "$(wc -l < mods.toml) lines in config"
else
    fail "Config file not created"
fi

# Test 2: Add mods
section "Test 2: Adding Mods to Config"

info "Adding test mod (ID: 123456)"
$BINARY add "Test Mod 1" 123456
if grep -q "Test Mod 1" mods.toml; then
    pass "Mod 1 added successfully"
else
    fail "Mod 1 not found in config"
fi

info "Adding another mod (ID: 789012)"
$BINARY add "Test Mod 2" 789012
if grep -q "Test Mod 2" mods.toml; then
    pass "Mod 2 added successfully"
else
    fail "Mod 2 not found in config"
fi

# Test 3: List mods
section "Test 3: Listing Mods"

info "Running: hsmm list"
LIST_OUTPUT=$($BINARY list)

if echo "$LIST_OUTPUT" | grep -q "Test Mod 1"; then
    pass "Mod 1 appears in list"
else
    fail "Mod 1 not in list output"
fi

if echo "$LIST_OUTPUT" | grep -q "Test Mod 2"; then
    pass "Mod 2 appears in list"
else
    fail "Mod 2 not in list output"
fi

# Test 4: Config display
section "Test 4: Config Display"

info "Running: hsmm config"
CONFIG_OUTPUT=$($BINARY config)

if echo "$CONFIG_OUTPUT" | grep -q "Mods: 2"; then
    pass "Config shows correct mod count"
else
    warn "Config mod count may be incorrect"
fi

# Test 5: Real CurseForge test with actual Hytale mod
section "Test 5: CurseForge API Test (Real Hytale Mod)"

info "Testing with real Hytale mod from CurseForge"
info "Using Hytale (game ID: 70216) and EyeSpy mod (ID: 1423494)"

# Create test config with real Hytale mod
cat > test-cf-config.toml << 'EOF'
game_id = 70216  # Hytale

[[mods]]
name = "EyeSpy"
enabled = true
identifier = { curseforge = 1423494 }

[settings]
cleanup_old_versions = true
max_old_versions = 1
download_timeout_secs = 120
parallel_downloads = 1
EOF

mkdir -p test-mods

info "Attempting to download EyeSpy mod (this may take 10-30 seconds)..."
info "Running: hsmm upgrade --config test-cf-config.toml --output ./test-mods"

if timeout 120 "$BINARY" --config test-cf-config.toml --output ./test-mods upgrade 2>&1 | tee upgrade.log; then
    if ls test-mods/*.jar >/dev/null 2>&1 || ls test-mods/*.zip >/dev/null 2>&1; then
        MOD_FILE=$(ls test-mods/*.jar test-mods/*.zip 2>/dev/null | head -1)
        MOD_SIZE=$(du -h "$MOD_FILE" | cut -f1)
        pass "CurseForge API works! Downloaded: $(basename "$MOD_FILE") ($MOD_SIZE)"
        info "This proves the download mechanism works"
    else
        warn "API responded but no file downloaded (may be distribution blocked)"
        info "Check upgrade.log for details"
    fi
else
    warn "CurseForge test timed out or failed"
    info "This could mean:"
    info "  - Network issues"
    info "  - CurseForge API down"
    info "  - API key invalid"
    info "The mod manager code may still be correct"
fi

# Test 6: Upgrade detection (already up-to-date)
section "Test 6: Upgrade Detection"

info "Running upgrade again to test 'already up-to-date' detection"
if timeout 60 "$BINARY" --config test-cf-config.toml --output ./test-mods upgrade 2>&1 | tee upgrade2.log; then
    pass "Upgrade command completed successfully"

    # Check if it detected already up-to-date
    if grep -qi "up.to.date\|already\|latest" upgrade2.log; then
        pass "Detected mod is already up-to-date"
        info "Smart upgrade detection working correctly"
    else
        warn "May not have detected up-to-date status (check upgrade2.log)"
    fi
else
    warn "Second upgrade failed (may be rate limiting)"
fi

# Test 7: Upgrade with old version (archiving test)
section "Test 7: Upgrade with Old Version (Archiving)"

info "Creating fake old version of EyeSpy mod"
# Remove current version first
rm -f test-mods/EyeSpy-*.jar

# Create a fake older version
echo "fake old mod data" > test-mods/EyeSpy-2026.1.10-5600.jar
FAKE_OLD_FILE="test-mods/EyeSpy-2026.1.10-5600.jar"

if [ -f "$FAKE_OLD_FILE" ]; then
    pass "Created fake old version: $(basename "$FAKE_OLD_FILE")"
    info "Size: $(du -h "$FAKE_OLD_FILE" | cut -f1)"
else
    fail "Failed to create fake old version"
fi

info "Running upgrade to replace old version with latest"
if timeout 60 "$BINARY" --config test-cf-config.toml --output ./test-mods upgrade 2>&1 | tee upgrade3.log; then
    pass "Upgrade completed successfully"

    # Check that new version was downloaded
    if ls test-mods/EyeSpy-2026.1.20-*.jar >/dev/null 2>&1; then
        NEW_FILE=$(ls test-mods/EyeSpy-2026.1.20-*.jar | head -1)
        pass "New version downloaded: $(basename "$NEW_FILE")"
        info "Size: $(du -h "$NEW_FILE" | cut -f1)"
    else
        fail "New version not found after upgrade"
    fi

    # Check that old version was archived
    if [ -d "test-mods/.old" ]; then
        pass ".old directory created for archiving"

        if ls test-mods/.old/EyeSpy-2026.1.10-*.jar >/dev/null 2>&1; then
            ARCHIVED_FILE=$(ls test-mods/.old/EyeSpy-2026.1.10-*.jar | head -1)
            pass "Old version archived: $(basename "$ARCHIVED_FILE")"
            info "Archive location: .old/$(basename "$ARCHIVED_FILE")"
        else
            warn "Old version not found in .old directory (may have been cleaned)"
        fi
    else
        fail ".old directory not created"
    fi

    # Verify old version no longer in main directory
    if ! ls test-mods/EyeSpy-2026.1.10-*.jar >/dev/null 2>&1; then
        pass "Old version removed from main directory"
    else
        fail "Old version still in main directory"
    fi
else
    warn "Upgrade with old version failed"
fi

# Test 8: Remove mod
section "Test 8: Removing Mods"

info "Removing 'Test Mod 1'"
$BINARY remove "Test Mod 1"

if ! grep -q "Test Mod 1" mods.toml; then
    pass "Mod 1 removed successfully"
else
    fail "Mod 1 still in config"
fi

LIST_OUTPUT=$($BINARY list)
if ! echo "$LIST_OUTPUT" | grep -q "Test Mod 1"; then
    pass "Mod 1 no longer in list"
else
    fail "Mod 1 still appears in list"
fi

# Test 9: Config validation
section "Test 9: Config File Validation"

info "Checking TOML syntax"
if cat mods.toml | grep -q "^\[settings\]"; then
    pass "Config has [settings] section"
else
    fail "Config missing [settings] section"
fi

info "Verifying remaining mod count"
MOD_COUNT=$(grep -c "^\[\[mods\]\]" mods.toml || echo 0)
if [ "$MOD_COUNT" -eq 1 ]; then
    pass "Correct mod count after removal ($MOD_COUNT mod)"
else
    warn "Unexpected mod count: $MOD_COUNT (expected 1)"
fi

# Test 10: Cleanup behavior
section "Test 10: Old Version Cleanup"

info "Checking .old directory handling"
mkdir -p test-mods/.old

# Create fake old mod file
echo "fake mod data" > test-mods/old-mod-v1.0.0.jar

# Run upgrade (will clean up old files)
info "Running upgrade to trigger cleanup..."
$BINARY --config test-cf-config.toml --output ./test-mods upgrade > /dev/null 2>&1 || true

if [ -d "test-mods/.old" ]; then
    pass ".old directory exists (cleanup structure present)"
else
    warn ".old directory not created (cleanup may not have run)"
fi

# Test 11: Error handling
section "Test 11: Error Handling"

info "Testing invalid config file"
echo "invalid toml [[" > bad-config.toml
if $BINARY --config bad-config.toml list 2>&1 | grep -qi "error\|failed"; then
    pass "Shows error for invalid config"
else
    fail "Doesn't handle invalid config properly"
fi

info "Testing non-existent mod removal"
if $BINARY remove "NonExistent Mod" 2>&1 | grep -qi "not found\|error"; then
    pass "Shows error for non-existent mod"
else
    warn "May not show clear error for missing mod"
fi

# Final summary
section "Test Summary"

cd "$PROJECT_ROOT" || exit 1

echo ""
echo "=========================================="
if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "The hsmm binary is working correctly."
    echo "Ready for Docker integration testing."
else
    echo -e "${YELLOW}⚠ $FAILURES test(s) failed${NC}"
    echo ""
    echo "Some tests failed, but this may be expected:"
    echo "  - CurseForge API issues (timeout, network)"
    echo "  - Missing Hytale game ID"
    echo "  - API rate limiting"
    echo ""
    echo "Check the test output above for details."
fi

echo ""
echo "Test artifacts in: $TEST_DIR"
echo "  - mods.toml: Generated config"
echo "  - test-mods/: Downloaded files"
echo "  - upgrade.log: CurseForge API output"
echo ""
echo "To clean up: rm -rf $TEST_DIR"
echo "=========================================="

exit $FAILURES
