#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
GAME_INPUT="${GAME:-}"
JOBS="${JOBS:-}"
OUTPUT_INPUT="${OUTPUT:-/mnt/nvme/MHGU}"

usage() {
    cat <<'EOF'
Usage: ./build-linux.sh [--game DIR] [--output DIR] [--jobs N]

Builds the Linux runtime, updater, and launcher into /mnt/nvme/MHGU/.
The game directory must contain exefs/main and romfs. If generated game code is missing or stale, it also needs
the exefs/sdk file so the script can regenerate the game code.

Defaults: game=/mnt/nvme/MHGU/data, output=/mnt/nvme/MHGU
Environment: GAME=DIR OUTPUT=DIR JOBS=N
EOF
}

fail() {
    printf 'build-linux: %s\n' "$*" >&2
    exit 1
}

while (($#)); do
    case "$1" in
        --game)
            (($# >= 2)) || fail "--game requires a directory"
            GAME_INPUT="$2"
            shift 2
            ;;
        --jobs)
            (($# >= 2)) || fail "--jobs requires a positive integer"
            JOBS="$2"
            shift 2
            ;;
        --output)
            (($# >= 2)) || fail "--output requires a directory"
            OUTPUT_INPUT="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --*)
            fail "unknown option: $1"
            ;;
        *)
            [[ -z "$GAME_INPUT" ]] || fail "game directory was specified more than once"
            GAME_INPUT="$1"
            shift
            ;;
    esac
done

mkdir -p "$OUTPUT_INPUT"
OUTPUT_DIR="$(cd -- "$OUTPUT_INPUT" && pwd -P)"
[[ -n "$GAME_INPUT" ]] || GAME_INPUT="$OUTPUT_DIR/data"
[[ -d "$GAME_INPUT" ]] || fail "game directory not found: $GAME_INPUT"
GAME_DIR="$(cd -- "$GAME_INPUT" && pwd -P)"
EXEFS_DIR="$GAME_DIR/exefs"
ROMFS_DIR="$GAME_DIR/romfs"
if [[ ! -d "$EXEFS_DIR" ]]; then
    EXEFS_DIR="$(find "$GAME_DIR" -mindepth 1 -maxdepth 1 -type d -iname exefs -print -quit)"
fi
if [[ ! -d "$ROMFS_DIR" ]]; then
    ROMFS_DIR="$(find "$GAME_DIR" -mindepth 1 -maxdepth 1 -type d -iname romfs -print -quit)"
fi
[[ -f "$EXEFS_DIR/main" ]] || fail "missing exefs/main (or ExeFS/main) in $GAME_DIR"
[[ -d "$ROMFS_DIR" ]] || fail "missing romfs (or RomFS) in $GAME_DIR"

for tool in make git clang clang++ lld python3 cmake curl sha256sum tar pkg-config npm node cargo rustc objdump; do
    command -v "$tool" >/dev/null 2>&1 || fail "required tool not found: $tool"
done
python3 -c 'import sys; sys.exit(0 if sys.version_info >= (3, 10) else 1)' \
    || fail "Python 3.10 or newer is required"
if ! NODE_VERSION="$(node --version 2>&1)"; then
    fail "Node.js could not start: $NODE_VERSION"
fi
NODE_MAJOR="${NODE_VERSION#v}"
NODE_MAJOR="${NODE_MAJOR%%.*}"
[[ "$NODE_MAJOR" =~ ^[0-9]+$ ]] || fail "could not parse Node.js version: $NODE_VERSION"
((NODE_MAJOR >= 22)) || fail "Node.js 22 or newer is required (found $NODE_VERSION)"

if ! pkg-config --exists gtk+-3.0 webkit2gtk-4.1; then
    fail "Tauri Linux development libraries are missing (need GTK 3 and WebKitGTK 4.1; see README.md, Linux)"
fi
if ! printf '#include <vulkan/vulkan.h>\n' | clang -E -x c - >/dev/null 2>&1; then
    fail "Vulkan headers are missing (install vulkan-headers; on Arch: pacman -S vulkan-headers)"
fi

if [[ -z "$JOBS" ]]; then
    JOBS="$(nproc 2>/dev/null || printf '4')"
    ((JOBS > 8)) && JOBS=8
fi
[[ "$JOBS" =~ ^[1-9][0-9]*$ ]] || fail "job count must be a positive integer"
GAME_REL="$(realpath --relative-to="$ROOT" "$GAME_DIR")"

cd "$ROOT"
if [[ ! -e "$GAME_DIR/exefs" ]]; then
    ln -s "$(basename -- "$EXEFS_DIR")" "$GAME_DIR/exefs"
fi
if [[ ! -e "$GAME_DIR/romfs" ]]; then
    ln -s "$(basename -- "$ROMFS_DIR")" "$GAME_DIR/romfs"
fi

GEN_SRCS=(tools/gen.py tools/lifter.py tools/neon.py tools/nso.py runtime/hle/hle_symbols.txt)
GEN_ID="$(cat "${GEN_SRCS[@]}" | tr -d '\r' | sha256sum | cut -c1-16)"
HAS_GENERATED_CODE=0
if [[ -f generated/gen.id && "$(tr -d '\r\n' < generated/gen.id)" == "$GEN_ID" ]] \
    && find generated/main -maxdepth 1 -type f -name '*.c' -print -quit 2>/dev/null | grep -q . \
    && find generated/sdk -maxdepth 1 -type f -name '*.c' -print -quit 2>/dev/null | grep -q .; then
    HAS_GENERATED_CODE=1
fi

PYTHON="$ROOT/tools/venv/bin/python"
if [[ ! -x "$PYTHON" ]]; then
    python3 -m venv tools/venv
fi
"$PYTHON" -m pip install -r tools/requirements.txt

if ((HAS_GENERATED_CODE)); then
    printf 'Using current generated game code in generated/ (gen.id %s)\n' "$GEN_ID"
else
    SDK_FILE="$EXEFS_DIR/sdk"
    if [[ ! -f "$SDK_FILE" ]]; then
        SDK_FILE="$(find "$EXEFS_DIR" -mindepth 1 -maxdepth 1 -type f -iname sdk -print -quit)"
    fi
    [[ -f "$SDK_FILE" ]] || fail "generated game code is missing or stale, and the exefs/sdk file was not found in $GAME_DIR"
    if [[ ! -e "$EXEFS_DIR/sdk" ]]; then
        ln -s "$(basename -- "$SDK_FILE")" "$EXEFS_DIR/sdk"
    fi
    printf 'Generating game code from %s\n' "$GAME_DIR"
    make gen GAME="$GAME_REL"
fi
if [[ -f "$OUTPUT_DIR/data/exefs/main" && -d "$OUTPUT_DIR/data/romfs" ]]; then
    touch "$OUTPUT_DIR/data/.complete"
fi
printf 'Building Linux runtime into %s (-j%s)\n' "$OUTPUT_DIR" "$JOBS"
make -j"$JOBS" PLATFORM=linux B="$OUTPUT_DIR" O=obj-linux GAME="$GAME_REL"

printf 'Building Linux launcher\n'
mkdir -p src-tauri/icons
cp backgrounds/launcher-icon.png src-tauri/icons/icon.png
npm ci
npm run build:launcher
cargo build --manifest-path src-tauri/Cargo.toml --release
install -D -m 755 src-tauri/target/release/mhgu-launcher "$OUTPUT_DIR/MHGU-Launcher"
install -D -m 644 backgrounds/launcher-icon.png "$OUTPUT_DIR/MHGU-Launcher.png"
cat > "$OUTPUT_DIR/MHGU-Launcher.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Monster Hunter Generations Ultimate Recomp
Exec="$OUTPUT_DIR/MHGU-Launcher"
Icon=$OUTPUT_DIR/MHGU-Launcher.png
Terminal=false
Categories=Game;
StartupWMClass=com.mhgu.recomp.launcher
EOF
chmod 755 "$OUTPUT_DIR/MHGU-Launcher.desktop"
cp README.md LICENSE THIRD_PARTY_NOTICES "$OUTPUT_DIR/"

printf '\nBuild complete: %s\n' "$OUTPUT_DIR"
printf 'Start the game with: %s/MHGU-Launcher or %s/mhgurecomp\n' "$OUTPUT_DIR" "$OUTPUT_DIR"
