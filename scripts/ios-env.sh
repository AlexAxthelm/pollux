# Shared by the iOS tasks in mise-tasks/. Source it; do not run it.
# Values are computed when a task runs, not when mise loads, so tasks that don't
# need a simulator never call `xcrun simctl`.

XCODE_PROJECT="iOS/Pollux.xcodeproj"
XCODE_SCHEME="Pollux"

# Pin DerivedData to a repo-local path (build/ is gitignored) so the built app is
# always at a known location. xcodegen regenerates the .xcodeproj, which shifts
# Xcode's default per-project DerivedData hash and leaves stale Pollux-<hash>/
# builds behind, so a naive `find | head -1` would install an old binary.
DERIVED_DATA="$MISE_PROJECT_ROOT/build/DerivedData"
APP_PATH="$DERIVED_DATA/Build/Products/Debug-iphonesimulator/$XCODE_SCHEME.app"
DEVICE_APP_PATH="$DERIVED_DATA/Build/Products/Debug-iphoneos/$XCODE_SCHEME.app"

# Override with SIM_DEVICE_NAME="iPhone 16" mise run ios-sim
SIM_DEVICE_NAME="${SIM_DEVICE_NAME:-iPhone 14 Pro Max}"

# Prints the udid of the first available simulator named $SIM_DEVICE_NAME, or fails with
# a hint. Use as `SIM_ID="$(sim_id)"` under `set -e`.
sim_id() {
  local id
  id="$(xcrun simctl list devices available -j \
    | jq -r --arg name "$SIM_DEVICE_NAME" '.devices
        | to_entries[].value[]
        | select(.name == $name and .isAvailable == true)
        | .udid' \
    | head -1)"
  if [ -z "$id" ]; then
    echo "Simulator '$SIM_DEVICE_NAME' not found. See \`xcrun simctl list devices available\`, then set SIM_DEVICE_NAME." >&2
    return 1
  fi
  echo "$id"
}

bundle_id() { /usr/libexec/PlistBuddy -c "Print CFBundleIdentifier" "$1/Info.plist"; }
