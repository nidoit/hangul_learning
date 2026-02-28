#!/usr/bin/env bash
#
# build.sh — Build and package DocConvert
#
# Usage:
#   ./build.sh [--dev | --release]
#
#   --dev       Start Tauri dev server (hot-reload)
#   --release   Build optimized release binary + .run installer (default)
#
set -euo pipefail

# ── Constants ───────────────────────────────────────────
APP_NAME="docconvert"
APP_DISPLAY="DocConvert"
APP_VERSION="1.0.0"
APP_DESC="Cross-platform document converter: PDF to Markdown, XLSX to CSV"
APP_ID="com.docconvert.app"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAURI_DIR="$SCRIPT_DIR/src-tauri"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ─────────────────────────────────────────────
info()  { printf "${CYAN}[info]${RESET}  %s\n" "$*"; }
ok()    { printf "${GREEN}[ok]${RESET}    %s\n" "$*"; }
warn()  { printf "${YELLOW}[warn]${RESET}  %s\n" "$*"; }
err()   { printf "${RED}[error]${RESET} %s\n" "$*" >&2; }
die()   { err "$@"; exit 1; }

# ── Parse arguments ─────────────────────────────────────
MODE="release"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dev)     MODE="dev";     shift ;;
        --release) MODE="release"; shift ;;
        -h|--help)
            echo "Usage: $0 [--dev | --release]"
            echo "  --dev       Start Tauri dev server"
            echo "  --release   Build release binary + .run installer (default)"
            exit 0
            ;;
        *) die "Unknown option: $1" ;;
    esac
done

# ── Detect host target triple ──────────────────────────
detect_target() {
    local arch kernel triple
    arch="$(uname -m)"
    kernel="$(uname -s)"

    case "$arch" in
        x86_64|amd64)   arch="x86_64"  ;;
        aarch64|arm64)   arch="aarch64" ;;
        *)               die "Unsupported architecture: $arch" ;;
    esac

    case "$kernel" in
        Linux)   triple="${arch}-unknown-linux-gnu" ;;
        Darwin)  triple="${arch}-apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) triple="${arch}-pc-windows-msvc" ;;
        *)       die "Unsupported OS: $kernel" ;;
    esac

    echo "$triple"
}

TARGET_TRIPLE="$(detect_target)"
info "Host target: ${BOLD}$TARGET_TRIPLE${RESET}"

# ── Dependency checks ──────────────────────────────────
MISSING=0

check_cmd() {
    local cmd="$1" hint="$2"
    if command -v "$cmd" &>/dev/null; then
        local ver
        ver="$("$cmd" --version 2>/dev/null | head -1)" || ver="found"
        ok "$cmd — $ver"
    else
        err "$cmd not found"
        warn "  Install: $hint"
        MISSING=$((MISSING + 1))
    fi
}

check_pkg_config_lib() {
    local lib="$1" hint="$2"
    if pkg-config --exists "$lib" 2>/dev/null; then
        local ver
        ver="$(pkg-config --modversion "$lib" 2>/dev/null)" || ver="found"
        ok "$lib — $ver"
    else
        err "System library '$lib' not found"
        warn "  Install: $hint"
        MISSING=$((MISSING + 1))
    fi
}

echo ""
info "Checking runtime dependencies..."
check_cmd node       "https://nodejs.org/ or: nvm install --lts"
check_cmd npm        "(included with Node.js)"
check_cmd rustc      "https://rustup.rs/ — curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_cmd cargo      "(included with rustup)"
check_cmd pkg-config "apt install pkg-config | dnf install pkgconf-pkg-config | pacman -S pkgconf"

echo ""
info "Checking system libraries (Linux)..."
if [[ "$(uname -s)" == "Linux" ]]; then
    check_pkg_config_lib "gtk+-3.0"        "apt install libgtk-3-dev | dnf install gtk3-devel | pacman -S gtk3"
    check_pkg_config_lib "webkit2gtk-4.1"  "apt install libwebkit2gtk-4.1-dev | dnf install webkit2gtk4.1-devel | pacman -S webkit2gtk-4.1"
    check_pkg_config_lib "gdk-3.0"         "apt install libgtk-3-dev | dnf install gtk3-devel | pacman -S gtk3"
    check_pkg_config_lib "libsoup-3.0"     "apt install libsoup-3.0-dev | dnf install libsoup3-devel | pacman -S libsoup3"
    check_pkg_config_lib "javascriptcoregtk-4.1" \
        "apt install libjavascriptcoregtk-4.1-dev | dnf install webkit2gtk4.1-devel | pacman -S webkit2gtk-4.1"
else
    info "Non-Linux host — skipping system library checks"
fi

if [[ $MISSING -gt 0 ]]; then
    echo ""
    die "$MISSING missing dependency(ies). Install them and re-run."
fi

echo ""
ok "All dependencies satisfied"

# ── Frontend: install npm packages (if package.json exists) ──
if [[ -f "$SCRIPT_DIR/package.json" ]]; then
    echo ""
    info "Installing frontend dependencies..."
    (cd "$SCRIPT_DIR" && npm install --no-audit --no-fund)
    ok "npm install complete"
else
    info "No package.json found — vanilla JS frontend, skipping npm install"
fi

# ── Dev mode ────────────────────────────────────────────
if [[ "$MODE" == "dev" ]]; then
    echo ""
    info "Starting Tauri dev server..."
    exec npx tauri dev
    # exec replaces this process; code below is not reached
fi

# ── Release build ───────────────────────────────────────
echo ""
info "Building ${BOLD}$APP_DISPLAY v$APP_VERSION${RESET} (release)..."
BUILD_EXIT=0
(cd "$SCRIPT_DIR" && npx tauri build 2>&1) || BUILD_EXIT=$?

# Check if the binary was produced even if bundling partially failed
# (e.g., AppImage can fail on systems without xdg-open while .deb/.rpm succeed)
RELEASE_BIN="$TAURI_DIR/target/release/$APP_NAME"
if [[ $BUILD_EXIT -ne 0 ]] && [[ ! -f "$RELEASE_BIN" ]]; then
    die "Build failed — no release binary produced"
elif [[ $BUILD_EXIT -ne 0 ]]; then
    warn "Tauri build exited with code $BUILD_EXIT (some bundle targets may have failed)"
    warn "Release binary was produced — continuing with packaging"
else
    ok "Tauri build complete"
fi

# ── Strip the release binary ───────────────────────────
ORIG_SIZE="$(du -h "$RELEASE_BIN" | cut -f1)"
strip "$RELEASE_BIN" 2>/dev/null || warn "strip failed — continuing with unstripped binary"
STRIP_SIZE="$(du -h "$RELEASE_BIN" | cut -f1)"
info "Binary: $ORIG_SIZE → $STRIP_SIZE (stripped)"

# ── Prepare packaging staging area ─────────────────────
STAGING="$TAURI_DIR/target/installer-staging"
rm -rf "$STAGING"
mkdir -p "$STAGING"

# Copy binary
cp "$RELEASE_BIN" "$STAGING/$APP_NAME"
chmod 755 "$STAGING/$APP_NAME"

# Copy icons
cp "$TAURI_DIR/icons/128x128.png" "$STAGING/icon.png"
cp "$TAURI_DIR/icons/32x32.png"   "$STAGING/32x32.png"

# ── Generate .desktop entry ────────────────────────────
cat > "$STAGING/$APP_NAME.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=$APP_DISPLAY
Comment=$APP_DESC
Exec=${APP_NAME}
Icon=${APP_NAME}
Terminal=false
Categories=Utility;Office;
MimeType=application/pdf;application/vnd.openxmlformats-officedocument.spreadsheetml.sheet;
StartupNotify=true
DESKTOP

# ── Generate install.sh ───────────────────────────────
cat > "$STAGING/install.sh" <<'INSTALL_SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

APP_NAME="docconvert"
APP_DISPLAY="DocConvert"

# Allow user-local install via PREFIX env var
PREFIX="${DOCCONVERT_PREFIX:-/usr/local}"
BIN_DIR="$PREFIX/bin"
SHARE_DIR="$PREFIX/share"
ICON_DIR="$SHARE_DIR/icons/hicolor"
APP_DIR="$SHARE_DIR/applications"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; RESET='\033[0m'
info()  { printf "${CYAN}[info]${RESET}  %s\n" "$*"; }
ok()    { printf "${GREEN}[ok]${RESET}    %s\n" "$*"; }
err()   { printf "${RED}[error]${RESET} %s\n" "$*" >&2; }
die()   { err "$@"; exit 1; }

# Determine script directory (inside the extracted archive)
SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Uninstall mode ──────────────────────────────────────
if [[ "${1:-}" == "--uninstall" ]]; then
    info "Uninstalling $APP_DISPLAY from $PREFIX ..."
    rm -f  "$BIN_DIR/$APP_NAME"
    rm -f  "$ICON_DIR/128x128/apps/$APP_NAME.png"
    rm -f  "$ICON_DIR/32x32/apps/$APP_NAME.png"
    rm -f  "$APP_DIR/$APP_NAME.desktop"
    ok "Uninstalled $APP_DISPLAY"
    exit 0
fi

# ── Install ─────────────────────────────────────────────
info "Installing $APP_DISPLAY to $PREFIX ..."

# Check write access
if [[ ! -w "$PREFIX" ]] && [[ -d "$PREFIX" ]]; then
    die "No write permission to $PREFIX. Run with sudo or set ${APP_NAME^^}_PREFIX=~/.local"
fi

mkdir -p "$BIN_DIR"
mkdir -p "$ICON_DIR/128x128/apps"
mkdir -p "$ICON_DIR/32x32/apps"
mkdir -p "$APP_DIR"

install -m 755 "$SELF_DIR/$APP_NAME"          "$BIN_DIR/$APP_NAME"
install -m 644 "$SELF_DIR/icon.png"            "$ICON_DIR/128x128/apps/$APP_NAME.png"
install -m 644 "$SELF_DIR/32x32.png"           "$ICON_DIR/32x32/apps/$APP_NAME.png"

# Update .desktop Exec and Icon paths for the target prefix
sed "s|^Exec=.*|Exec=$BIN_DIR/$APP_NAME|;s|^Icon=.*|Icon=$APP_NAME|" \
    "$SELF_DIR/$APP_NAME.desktop" > "$APP_DIR/$APP_NAME.desktop"
chmod 644 "$APP_DIR/$APP_NAME.desktop"

# Update icon cache if available
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t "$ICON_DIR" 2>/dev/null || true
fi
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

ok "Installed $APP_DISPLAY to $PREFIX"
echo ""
echo "  Binary:  $BIN_DIR/$APP_NAME"
echo "  Desktop: $APP_DIR/$APP_NAME.desktop"
echo ""
INSTALL_SCRIPT
chmod 755 "$STAGING/install.sh"

# ── Build .run self-extracting installer ───────────────
INSTALLER_NAME="${APP_NAME}-${APP_VERSION}-${TARGET_TRIPLE}.run"
INSTALLER_PATH="$TAURI_DIR/target/$INSTALLER_NAME"

echo ""
info "Packaging .run installer..."

if command -v makeself &>/dev/null; then
    # Use makeself for a proper self-extracting archive
    makeself --gzip --nox11 \
        "$STAGING" \
        "$INSTALLER_PATH" \
        "$APP_DISPLAY v$APP_VERSION Installer" \
        ./install.sh
    ok "Created installer with makeself: $INSTALLER_NAME"
else
    warn "makeself not found — using built-in tar self-extractor"

    # Build a tar.gz payload
    PAYLOAD_FILE="$(mktemp)"
    (cd "$STAGING" && tar czf "$PAYLOAD_FILE" .)

    # Write a self-extracting shell script
    cat > "$INSTALLER_PATH" <<'SFX_HEADER'
#!/usr/bin/env bash
set -euo pipefail
APP_DISPLAY="DocConvert"
RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

echo -e "${BOLD}${CYAN}$APP_DISPLAY Installer${RESET}"
echo ""

# Find the payload marker line number
ARCHIVE_LINE=$(awk '/^__ARCHIVE_BELOW__$/{print NR + 1; exit 0}' "$0")
if [[ -z "$ARCHIVE_LINE" ]]; then
    echo -e "${RED}Error: corrupt installer — archive marker not found${RESET}" >&2
    exit 1
fi

# Extract to a temp directory
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

tail -n +"$ARCHIVE_LINE" "$0" | tar xzf - -C "$TMPDIR"

# Run install.sh, forwarding all arguments (e.g. --uninstall)
exec bash "$TMPDIR/install.sh" "$@"

# Payload follows — do not edit below this line
__ARCHIVE_BELOW__
SFX_HEADER

    # Append binary payload
    cat "$PAYLOAD_FILE" >> "$INSTALLER_PATH"
    rm -f "$PAYLOAD_FILE"
    chmod 755 "$INSTALLER_PATH"
    ok "Created self-extracting installer: $INSTALLER_NAME"
fi

INSTALLER_SIZE="$(du -h "$INSTALLER_PATH" | cut -f1)"

# ── Also list Tauri-native packages if produced ────────
echo ""
info "Tauri-native packages:"
find "$TAURI_DIR/target/release/bundle" -type f \( -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" -o -name "*.msi" -o -name "*.exe" -o -name "*.dmg" \) 2>/dev/null \
    | while read -r pkg; do
        echo "  $(du -h "$pkg" | cut -f1)  $pkg"
    done || info "  (none found)"

# ── Summary ─────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}══════════════════════════════════════════${RESET}"
echo -e "${BOLD}  $APP_DISPLAY v$APP_VERSION — Build Complete${RESET}"
echo -e "${BOLD}${GREEN}══════════════════════════════════════════${RESET}"
echo ""
echo "  Installer: $INSTALLER_PATH ($INSTALLER_SIZE)"
echo "  Target:    $TARGET_TRIPLE"
echo ""
echo -e "  ${BOLD}Install (system-wide):${RESET}"
echo "    sudo ./$INSTALLER_NAME"
echo ""
echo -e "  ${BOLD}Install (user-local):${RESET}"
echo "    DOCCONVERT_PREFIX=~/.local ./$INSTALLER_NAME"
echo ""
echo -e "  ${BOLD}Uninstall:${RESET}"
echo "    sudo ./$INSTALLER_NAME --uninstall"
echo ""
