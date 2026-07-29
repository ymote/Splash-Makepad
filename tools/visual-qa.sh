#!/usr/bin/env bash
# Visual QA sweep: drive every route on a connected device and screenshot it.
#
# The route-sweep test in crates/splash-makepad proves each screen *translates*
# to its own dialect. It cannot tell you whether the widgets then render — a
# control whose shader never binds, a container that collapses to zero height,
# or a Label whose box clips its descenders all translate perfectly and look
# wrong. That is what this catches, and it is how the elevation-drops-children,
# page-collapses-to-zero and clipped-descender bugs were found.
#
#   tools/visual-qa.sh                  # every route in tools/qa-routes.txt
#   tools/visual-qa.sh cupertino        # only routes matching a substring
#   DARK=1 tools/visual-qa.sh           # dark palette
#
# Output: tools/qa-shots/<route>.png plus contact sheets sheet-NN.png.
set -euo pipefail

cd "$(dirname "$0")/.."
ADB="${ADB:-$HOME/Library/Android/sdk/platform-tools/adb}"
ROUTE_FILE=/data/local/tmp/flutter_samples.route
OUT=tools/qa-shots
FILTER="${1:-}"
SETTLE="${SETTLE:-1.0}"

command -v "$ADB" >/dev/null 2>&1 || { echo "adb not found at $ADB" >&2; exit 1; }
"$ADB" get-state >/dev/null 2>&1 || { echo "no device attached" >&2; exit 1; }

mkdir -p "$OUT"
rm -f "$OUT"/*.png

# Read the list up front: `adb shell` consumes stdin, so looping over a pipe
# with adb inside it swallows the remaining routes after the first iteration.
mapfile -t ROUTES < tools/qa-routes.txt

n=0
for route in "${ROUTES[@]}"; do
    [ -z "$route" ] && continue
    if [ -n "$FILTER" ] && [[ "$route" != *"$FILTER"* ]]; then continue; fi

    line="$route"
    [ "${DARK:-0}" = "1" ] && line="$route dark"
    "$ADB" shell "echo '$line' > $ROUTE_FILE" < /dev/null
    sleep "$SETTLE"

    # `/` is legal in a route but not in a filename.
    name="${route//\//__}"
    "$ADB" exec-out screencap -p < /dev/null > "$OUT/$name.png"
    n=$((n + 1))
    printf '%3d  %s\n' "$n" "$route"
done

echo "captured $n screens into $OUT"
python3 tools/contact-sheet.py "$OUT"
