#!/usr/bin/env bash
# snapshot-diff.sh — golden screenshot regression harness for the egui-shadcn
# gallery (plan §8). Invoked with no arguments from the repository root by
# the CI `snapshot-diff` job (see the contract comment at the top of
# .github/workflows/ci.yml).
#
# Modes:
#   diff mode (default): capture every listed gallery view and compare each
#     against tests/golden/. Exit 0 iff every capture matches. On a
#     mismatch, the view is re-captured ONCE (plan §8 flakiness rule); a
#     second mismatch fails with a report naming each differing capture,
#     its AE metric, and the actual/golden/diff image paths. Diff mode
#     NEVER writes into tests/golden/.
#   update mode (SNAPSHOT_UPDATE=1): capture every view and (over)write
#     tests/golden/. Refused unless explicitly requested. A golden is
#     written only after two captures agree byte-identically; on
#     disagreement the view is re-captured once more and written only
#     if the third capture matches one of the first two, otherwise the
#     run fails naming the page as unstable — a one-off transient frame
#     can never become the reference (plan §8 flake rule, defect D2).
#     Update mode also fails if any two goldens within a theme come out
#     identical (that means navigation failed and a wrong golden would
#     be baked in).
#
# Golden set: every gallery demo page reachable from the sidebar, resting
# state, nova (default) style, light + dark, one fixed viewport
# (1200x3300). Capture names are "<index>-<name>-<theme>.png" where <index>
# is the page's position in the gallery's demo_pages() list, which
# disambiguates the duplicate sidebar names (popover x2, tooltip x2,
# dropdown-menu x2, context-menu x2, menubar x2). The 5 `show: None` stub
# entries are included (they render "<name>" + "Demo not ported yet.").
# EXCLUDED (see the Golden Snapshots Engineer report for evidence):
#   5 spinner, 6 skeleton — continuous time-based animation (spinner arc
#   rotates 360 deg/sec, skeleton pulse 2s period); no settle wait can
#   stabilize them, and byte-stable baselines are impossible without code
#   changes.
#   56 command — the resting page oscillates between two states 38px
#   apart: the closed CommandDialog still renders its inner Command
#   (autofocus(true) at src/components/command.rs:1283), whose search
#   input holds keyboard focus and paints a blinking 2x19 text cursor
#   (x 212..213, y 618..636). Blink phase depends on time-since-start, so
#   no settle wait stabilizes it; re-add once the dialog/input focus
#   behavior is fixed.
#   61 data-table — navigating to the page PANICS the gallery
#   (egui layout.rs "frame_rect is NaN", via header_sort_cell at
#   src/components/data_table.rs:802); reproduced 4x, identical panic at
#   1200x3300 and 1200x2900. No golden is possible until the component
#   is fixed; re-add "61:data-table" to PAGES and regenerate when it is.
# Total: 68 pages x 2 themes = 136 goldens.
#
# Navigation (measured, not assumed; see the report): the gallery has no
# keyboard/CLI navigation, and xdotool wheel scrolling does not move its
# sidebar ScrollArea, so the window is resized tall (1200x3300) to expose
# all 72 sidebar labels at once and each label is clicked directly.
# Geometry, measured from screenshots of this tree (embedded fonts make
# layout identical on every machine running this binary):
#   label i click point: x=30, y=80+44*i (pitch fit: 43.997px, max
#   residual 1.7px over all 72 labels; click tests at i=0,20,45,71 landed
#   exactly). Theme toggle button: rect x 1108..1191, y 2..37 in both
#   themes; click (1150,20), verified by sampling the topbar background
#   pixel (light (255,255,255), dark (10,10,10)) with up to 3 retries.
#   CLOSED-LOOP navigation: xdotool clicks are occasionally lost (one
#   missed click baked a wrong golden during development: the capture
#   for 53-navigation-menu showed the previous page). Every capture is
#   therefore verified: the pixel at (15, label-y) must equal the
#   theme's selection-fill color (measured at runtime from a
#   known-selected row, never hardcoded). A failed check re-clicks up
#   to 3 times, then fails naming the page — a wrong-page capture can
#   never be baked silently.
# The tall viewport is a deliberate deviation from the gallery's default
# 1200x800: it is what makes scroll-free deterministic navigation
# possible. All goldens share this viewport, satisfying plan §8's
# same-viewport rule.
#
# Diff metric: exact byte match (cmp). The metric reported per capture is
# 0 for byte-identical captures, else the ImageMagick AE (absolute error)
# count from `compare`. No tolerance: static pages re-capture
# byte-identically (proven in the report), and any tolerance would hide
# real regressions.
#
# Outputs (all under target/snapshot-diff/, already gitignored):
#   actual/<name>.png   fresh captures
#   diff/<name>.png     red-highlight diff images, mismatches only
#   report.txt          per-capture results + metric values
#   probe.png           theme-verification probe (overwritten)
#   confirm.png         update-mode confirmation capture (overwritten)
#   confirm2.png        update-mode re-capture on disagreement (overwritten)
#   gallery.log         gallery stdout/stderr (panic diagnostics)
#
# Requirements: cargo, Xvfb (via xvfb-run), xdotool, ImageMagick
# (import, compare, identify). Missing tools fail fast with a message
# naming the tool. When DISPLAY is unset the harness re-execs itself
# under `xvfb-run -a` with a 1400x3400 screen. When DISPLAY is already
# set, the existing display is used and must be at least 1400x3400.

set -uo pipefail

# ---------------------------------------------------------------- config
WIN_W=1200
WIN_H=3300
SCREEN_W=1400
SCREEN_H=3400
SIDEBAR_X=30
LABEL_Y0=80
LABEL_PITCH=44
TOGGLE_X=1150
TOGGLE_Y=20
SETTLE=0.6            # seconds after each navigation click (transitions <=150ms)
RESIZE_SETTLE=2       # seconds after resizing, for relayout
TOGGLE_RETRIES=3
NAV_RETRIES=3           # navigation click attempts per page (see header)
CLICK_DELAY_MS=100      # xdotool press->release gap (lost-click hardening)
GALLERY_TITLE="shadcn/ui for egui"

# Capture list: "<index>:<slug>" in demo_pages() order. Indices 5
# (spinner) and 6 (skeleton) are excluded: continuously animated, see the
# header note and the Golden Snapshots Engineer report. Index 56
# (command) is excluded: blinking text cursor in the resting state (see
# header). Index 61 (data-table) is excluded: navigating to it panics
# the gallery (NaN frame rect in header_sort_cell, data_table.rs:802);
# re-add it once the component is fixed.
PAGES=(
  "0:button" "1:badge" "2:label" "3:separator" "4:kbd"
  "7:typography" "8:aspect-ratio" "9:marker" "10:button-group"
  "11:input" "12:textarea" "13:checkbox" "14:radio-group" "15:switch"
  "16:slider" "17:toggle" "18:toggle-group" "19:input-otp"
  "20:native-select" "21:field" "22:input-group" "23:card" "24:alert"
  "25:avatar" "26:progress" "27:table" "28:empty" "29:item"
  "30:scroll-area" "31:resizable" "32:accordion" "33:collapsible"
  "34:tabs" "35:breadcrumb" "36:pagination" "37:popover"
  "38:dropdown-menu" "39:context-menu" "40:menubar" "41:dialog"
  "42:alert-dialog" "43:sheet" "44:drawer" "45:popover" "46:tooltip"
  "47:tooltip" "48:hover-card" "49:sonner" "50:dropdown-menu"
  "51:context-menu" "52:menubar" "53:navigation-menu" "54:select"
  "55:combobox" "57:calendar" "58:date-picker"
  "59:carousel" "60:chart" "62:form" "63:sidebar"
  "64:attachment" "65:bubble" "66:message" "67:message-scroller"
  "68:direction" "69:motion" "70:shadow" "71:icon"
)

# ---------------------------------------------------------------- setup
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

GOLDENS_DIR="tests/golden"
OUT_DIR="target/snapshot-diff"
ACTUAL_DIR="$OUT_DIR/actual"
DIFF_DIR="$OUT_DIR/diff"
REPORT="$OUT_DIR/report.txt"

UPDATE_MODE=0
if [ "${SNAPSHOT_UPDATE:-}" = "1" ]; then
    UPDATE_MODE=1
elif [ -n "${SNAPSHOT_UPDATE:-}" ]; then
    echo "error: SNAPSHOT_UPDATE must be exactly '1' to refresh goldens (got '$SNAPSHOT_UPDATE'); refusing." >&2
    exit 2
fi

die() { echo "error: $*" >&2; exit 1; }

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "required tool '$1' is missing ($2)"
}

# rustup's cargo is not always on PATH (bare shells, the xvfb-run
# re-exec below). Prefer a cargo already on PATH; otherwise fall back to
# the standard rustup bin dir so the harness stays self-contained. (In CI
# the toolchain action puts cargo on PATH already, so this is a no-op.)
if ! command -v cargo >/dev/null 2>&1; then
    for _cand in "${CARGO_HOME:-$HOME/.cargo}/bin" "$HOME/.cargo/bin"; do
        if [ -x "$_cand/cargo" ]; then PATH="$_cand:$PATH"; break; fi
    done
    unset _cand
fi

# Self-contained display: re-exec under xvfb-run when no display is set.
if [ -z "${DISPLAY:-}" ]; then
    require_cmd xvfb-run "install the 'xvfb' package (CI: snapshot-diff job apt step)"
    echo "no DISPLAY set; re-execing under xvfb-run"
    exec xvfb-run -a -s "-screen 0 ${SCREEN_W}x${SCREEN_H}x24" bash "$ROOT/scripts/snapshot-diff.sh"
fi

require_cmd cargo "install the Rust toolchain"
require_cmd xdotool "install the 'xdotool' package (CI: snapshot-diff job apt step)"
require_cmd import "install the 'imagemagick' package (CI: snapshot-diff job apt step)"
require_cmd compare "install the 'imagemagick' package (CI: snapshot-diff job apt step)"
require_cmd identify "install the 'imagemagick' package (CI: snapshot-diff job apt step)"
require_cmd timeout "GNU coreutils timeout (present on ubuntu-latest)"

# The existing display must fit the tall window.
read -r DW DH < <(xdotool getdisplaygeometry) || die "could not query display geometry"
if [ "$DW" -lt "$SCREEN_W" ] || [ "$DH" -lt "$SCREEN_H" ]; then
    die "display is ${DW}x${DH}; need at least ${SCREEN_W}x${SCREEN_H} (unset DISPLAY to use xvfb-run)"
fi

mkdir -p "$ACTUAL_DIR" "$DIFF_DIR" "$GOLDENS_DIR"
: > "$REPORT"

# ---------------------------------------------------------------- build
echo "building gallery..."
cargo build --example gallery >/dev/null 2>&1 || die "cargo build --example gallery failed"
GALLERY_BIN="target/debug/examples/gallery"
[ -x "$GALLERY_BIN" ] || die "gallery binary missing after build: $GALLERY_BIN"

# ---------------------------------------------------------------- launch
# Gallery stdout/stderr go to gallery.log (NOT /dev/null): if the gallery
# panics mid-run, the panic message is the diagnostic.
"$GALLERY_BIN" >"$OUT_DIR/gallery.log" 2>&1 &
GALLERY_PID=$!
cleanup() {
    if kill -0 "$GALLERY_PID" 2>/dev/null; then
        kill "$GALLERY_PID" 2>/dev/null
        wait "$GALLERY_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

WIN=""
for _ in $(seq 1 60); do
    WIN="$(xdotool search --name "$GALLERY_TITLE" 2>/dev/null | head -1)" || true
    [ -n "$WIN" ] && break
    sleep 1
done
[ -n "$WIN" ] || die "gallery window did not appear within 60s"
xdotool windowfocus "$WIN" || die "could not focus gallery window $WIN"
xdotool windowsize "$WIN" "$WIN_W" "$WIN_H" || die "could not resize gallery window"
sleep "$RESIZE_SETTLE"
GEOM="$(xdotool getwindowgeometry --shell "$WIN" | tr '\n' ' ')"
case "$GEOM" in
    *"WIDTH=$WIN_W"*"HEIGHT=$WIN_H"*) ;;
    *) die "gallery window geometry is not ${WIN_W}x${WIN_H}: $GEOM" ;;
esac
echo "gallery window $WIN at ${WIN_W}x${WIN_H} (pid $GALLERY_PID)"

# ------------------------------------------------------- capture helpers
# Mean intensity (0..1) of the topbar background probe pixel (600,25).
# `timeout` guards each X call so a wedged X round-trip fails fast with a
# clear message instead of hanging the job forever (GNU coreutils timeout
# is present on ubuntu-latest and on the dev VM).
probe_theme() {
    timeout 60 import -window "$WIN" "$OUT_DIR/probe.png" || return 1
    identify -format "%[fx:mean]" "$OUT_DIR/probe.png[1x1+600+25]" 2>/dev/null || return 1
}

# Single pixel color as "R,G,B" integers. Uses fx channel math (not
# %[pixel:...], whose "srgb(...)" vs "gray(...)" spelling depends on the
# PNG type) so sRGB and grayscale captures compare equal.
pixel_at() {
    identify -format "%[fx:int(r*255+0.5)],%[fx:int(g*255+0.5)],%[fx:int(b*255+0.5)]" "$1[1x1+$2+$3]" 2>/dev/null
}

# Click a sidebar label by index, wait for settle, capture to $1.
# Verifies the navigation landed: the pixel at (15, label-y) — inside
# the label rect, left of the text — must equal the theme's
# selection-fill color in SEL_COLOR (measured at runtime, see below).
# Retries the click up to NAV_RETRIES times; a wrong-page capture must
# never pass silently (a lost click once baked a wrong golden).
capture_page() {
    local idx="$1" out="$2"
    # If the gallery died (e.g. a page panicked it), fail with a pointer
    # to the gallery log instead of a cryptic import/xdotool error.
    kill -0 "$GALLERY_PID" 2>/dev/null || die "gallery process (pid $GALLERY_PID) exited before capture of index $idx; see $OUT_DIR/gallery.log for the panic/error"
    local y=$((LABEL_Y0 + LABEL_PITCH * idx))
    local attempt=1
    while [ "$attempt" -le "$NAV_RETRIES" ]; do
        timeout 60 xdotool mousemove "$SIDEBAR_X" "$y" || return 1
        sleep 0.5
        timeout 60 xdotool click --delay "$CLICK_DELAY_MS" 1 || return 1
        sleep "$SETTLE"
        timeout 60 import -window "$WIN" "$out" || return 1
        local size
        size="$(identify -format "%wx%h" "$out")" || return 1
        [ "$size" = "${WIN_W}x${WIN_H}" ] || { echo "capture $out has size $size, expected ${WIN_W}x${WIN_H}" >&2; return 1; }
        local got
        got="$(pixel_at "$out" 15 "$y")" || return 1
        if [ "$got" = "$SEL_COLOR" ]; then
            return 0
        fi
        echo "navigation to row $idx did not land (attempt $attempt/$NAV_RETRIES: selection pixel is $got, expected $SEL_COLOR); re-clicking..." >&2
        attempt=$((attempt + 1))
    done
    echo "navigation to row $idx failed after $NAV_RETRIES attempts (selection never landed on the row)" >&2
    return 1
}

# AE metric between two images (stdout: integer). Assumes files exist.
ae_of() {
    compare -metric AE "$1" "$2" null: 2>&1 | tr -d '[:space:]'
}

# Verify the starting theme is light; fail loudly otherwise.
echo "verifying starting theme is light..."
MEAN="$(probe_theme)" || die "theme probe capture failed"
if ! awk -v m="$MEAN" 'BEGIN { exit !(m > 0.5) }'; then
    die "gallery did not start in light mode (probe mean $MEAN); refusing to guess"
fi
echo "starting theme: light (probe mean $MEAN)"
# Measure the light selection-fill color from the probe: row 0 (button)
# is selected by default, so (15,80) is inside its selection rect.
SEL_COLOR_LIGHT="$(pixel_at "$OUT_DIR/probe.png" 15 "$LABEL_Y0")" || die "could not sample light selection color"
[ -n "$SEL_COLOR_LIGHT" ] || die "could not sample light selection color"
echo "light selection color: $SEL_COLOR_LIGHT"
SEL_COLOR="$SEL_COLOR_LIGHT"

# ------------------------------------------------------------- main loop
FAILURES=0
FLAKES=0

run_theme() {
    local theme="$1"
    for entry in "${PAGES[@]}"; do
        local idx="${entry%%:*}"
        local slug="${entry##*:}"
        local name
        name="$(printf "%02d-%s-%s" "$idx" "$slug" "$theme")"
        local actual="$ACTUAL_DIR/$name.png"
        local golden="$GOLDENS_DIR/$name.png"

        capture_page "$idx" "$actual" || die "capture failed for $name"

        if [ "$UPDATE_MODE" -eq 1 ]; then
            # Plan §8 flake rule (defect D2): never write a golden from
            # a single unconfirmed capture. Capture a second time and
            # require byte-identical agreement; on disagreement,
            # re-capture once more and write only if the third capture
            # matches one of the first two; otherwise fail naming the
            # page as unstable (no golden is written for that page).
            local confirm="$OUT_DIR/confirm.png"
            local recaptured=0
            capture_page "$idx" "$confirm" || die "confirmation capture failed for $name"
            if ! cmp -s "$actual" "$confirm"; then
                echo "$name: update captures disagree; re-capturing once..." | tee -a "$REPORT"
                local confirm2="$OUT_DIR/confirm2.png"
                capture_page "$idx" "$confirm2" || die "re-capture failed for $name"
                recaptured=1
                if cmp -s "$confirm2" "$confirm"; then
                    # Third capture agrees with the confirmation
                    # capture: the first capture was the transient
                    # one; stage the agreed value.
                    cp "$confirm" "$actual" || die "could not stage agreed capture for $name"
                elif ! cmp -s "$confirm2" "$actual"; then
                    echo "$name: UNSTABLE (3 captures disagree; no golden written)" | tee -a "$REPORT"
                    die "update for $name unstable: 3 captures disagree; no golden written for $name"
                fi
                # Otherwise the third capture agrees with the first:
                # $actual already holds the agreed content.
            fi
            local metric=0
            if [ -f "$golden" ]; then
                if cmp -s "$actual" "$golden"; then
                    metric=0
                else
                    metric="$(ae_of "$actual" "$golden")"
                fi
            else
                metric="new"
            fi
            cp "$actual" "$golden" || die "could not write golden $golden"
            if [ "$recaptured" -eq 1 ]; then
                echo "$name: UPDATED after 1 re-capture (flake noted; metric vs previous: $metric)" | tee -a "$REPORT"
            else
                echo "$name: UPDATED (metric vs previous: $metric)" | tee -a "$REPORT"
            fi
            continue
        fi

        # Diff mode: byte match, else one re-capture, else failure.
        if [ ! -f "$golden" ]; then
            echo "$name: MISSING GOLDEN ($golden)" | tee -a "$REPORT"
            FAILURES=$((FAILURES + 1))
            continue
        fi
        if cmp -s "$actual" "$golden"; then
            echo "$name: MATCH (metric 0)" | tee -a "$REPORT"
            continue
        fi
        # Plan §8 flakiness rule: one re-capture before filing.
        echo "$name: mismatch on first capture; re-capturing once..."
        capture_page "$idx" "$actual" || die "re-capture failed for $name"
        if cmp -s "$actual" "$golden"; then
            echo "$name: MATCH after 1 re-capture (flake noted; metric 0)" | tee -a "$REPORT"
            FLAKES=$((FLAKES + 1))
            continue
        fi
        local metric
        metric="$(ae_of "$actual" "$golden")"
        compare -highlight-color red "$actual" "$golden" "$DIFF_DIR/$name.png" >/dev/null 2>&1 || true
        echo "$name: MISMATCH (metric AE=$metric) actual=$actual golden=$golden diff=$DIFF_DIR/$name.png" | tee -a "$REPORT"
        FAILURES=$((FAILURES + 1))
    done
}

echo "=== capturing light theme (${#PAGES[@]} pages) ==="
run_theme light

# Toggle to dark, verified by probe pixel, with retries.
echo "toggling to dark theme..."
TOGGLED=0
for _ in $(seq 1 "$TOGGLE_RETRIES"); do
    timeout 60 xdotool mousemove "$TOGGLE_X" "$TOGGLE_Y" || die "mousemove to toggle failed"
    sleep 0.5
    timeout 60 xdotool click --delay "$CLICK_DELAY_MS" 1 || die "toggle click failed"
    sleep 0.8
    MEAN="$(probe_theme)" || die "theme probe capture failed"
    if awk -v m="$MEAN" 'BEGIN { exit !(m < 0.5) }'; then
        TOGGLED=1
        break
    fi
    echo "toggle attempt did not change theme (probe mean $MEAN); retrying..."
done
[ "$TOGGLED" -eq 1 ] || die "theme toggle did not switch to dark after $TOGGLE_RETRIES attempts"
echo "theme now dark (probe mean $MEAN)"
# Measure the dark selection-fill color from the toggle probe: the last
# light page (row 71) is still selected, so (15,3204) is inside its
# selection rect. It must differ from the light color (sanity check).
DARK_Y=$((LABEL_Y0 + LABEL_PITCH * 71))
SEL_COLOR_DARK="$(pixel_at "$OUT_DIR/probe.png" 15 "$DARK_Y")" || die "could not sample dark selection color"
[ -n "$SEL_COLOR_DARK" ] || die "could not sample dark selection color"
[ "$SEL_COLOR_DARK" != "$SEL_COLOR_LIGHT" ] || die "dark selection color equals light ($SEL_COLOR_DARK); theme probe state unexpected"
echo "dark selection color: $SEL_COLOR_DARK"
SEL_COLOR="$SEL_COLOR_DARK"

echo "=== capturing dark theme (${#PAGES[@]} pages) ==="
run_theme dark

# ------------------------------------------------------------- finish
if [ "$UPDATE_MODE" -eq 1 ]; then
    # Guard against baked-in navigation failures: within a theme, every
    # golden must be distinct (all pages have distinct headings).
    echo "checking goldens are pairwise distinct within each theme..."
    for theme in light dark; do
        DUPES="$(md5sum "$GOLDENS_DIR"/*-"$theme".png | awk '{print $1}' | sort | uniq -d)"
        if [ -n "$DUPES" ]; then
            echo "error: duplicate goldens in $theme theme (identical md5):" >&2
            for d in $DUPES; do
                md5sum "$GOLDENS_DIR"/*-"$theme".png | grep "^$d" >&2
            done
            echo "error: refusing to leave duplicate goldens; navigation likely failed for those pages." >&2
            exit 1
        fi
    done
    COUNT="$(find "$GOLDENS_DIR" -name '*.png' | wc -l)"
    BYTES="$(du -sb "$GOLDENS_DIR" | cut -f1)"
    echo "goldens refreshed: $COUNT files, $BYTES bytes in $GOLDENS_DIR"
    echo "report: $REPORT"
    exit 0
fi

# Warn (do not fail) about goldens with no capture in the list.
for g in "$GOLDENS_DIR"/*.png; do
    [ -e "$g" ] || continue
    base="$(basename "$g")"
    if [ ! -f "$ACTUAL_DIR/$base" ]; then
        echo "warning: golden $base has no capture in the list (stale?)" | tee -a "$REPORT"
    fi
done

echo "---"
echo "captures: $(( ${#PAGES[@]} * 2 )), mismatches: $FAILURES, flakes recovered: $FLAKES"
echo "report: $REPORT"
if [ "$FAILURES" -gt 0 ]; then
    echo "SNAPSHOT-DIFF: FAIL ($FAILURES mismatches; see $REPORT)" >&2
    exit 1
fi
echo "SNAPSHOT-DIFF: PASS"
exit 0
