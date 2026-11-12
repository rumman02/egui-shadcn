#!/usr/bin/env python3
"""Generate `src/icons/data.rs` from the official Lucide SVG release.

The icon geometry in this crate is REAL Lucide data: each icon's `<path>`,
`<circle>`, `<line>` and `<rect>` elements are converted to `El` values, with
SVG paths flattened to polylines (arcs sampled at <=3 degrees, cubics
subdivided to 0.02 grid units, quadratics converted exactly to cubics).
Nothing is hand-drawn or approximated beyond the flattening tolerance,
which is sub-pixel at every size the crate paints.

To add an icon: append one line to ICONS below (crate variant name ->
Lucide kebab name), add the variant to the `Icon` enum in `icons.rs`, and
re-run this script. To update Lucide: download the new release zip,
re-point --icons-dir, and record the new version/hash in the header this
script emits.

Usage:
    python3 src/icons/gen_icons.py --icons-dir <unzipped lucide icons/> \
        --out src/icons/data.rs
"""

import argparse
import math
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

# (Crate variant, Lucide kebab name). Order defines Icon::all() order.
# Rename notes: Lucide 1.x renamed `circle-alert` -> `circle-exclamation-point`,
# `triangle-alert` -> `triangle-exclamation-point`, `more-horizontal` ->
# `ellipsis`, `more-vertical` -> `ellipsis-vertical`; the SVG geometry is
# unchanged (rename only), verified against the old paths quoted in the
# crate's previous hand-drawn code. `Loader2` (lucide-react alias) is
# `loader-circle`; `Message` is `message-circle` (the rounded bubble the
# previous approximation traced, not `message-square`).
ICONS = [
    ("Check", "check"),
    ("ChevronDown", "chevron-down"),
    ("ChevronUp", "chevron-up"),
    ("ChevronLeft", "chevron-left"),
    ("ChevronRight", "chevron-right"),
    ("ChevronsLeft", "chevrons-left"),
    ("ChevronsRight", "chevrons-right"),
    ("X", "x"),
    ("Minus", "minus"),
    ("Plus", "plus"),
    ("CircleAlert", "circle-exclamation-point"),
    ("CircleCheck", "circle-check"),
    ("CircleX", "circle-x"),
    ("TriangleAlert", "triangle-exclamation-point"),
    ("OctagonX", "octagon-x"),
    ("Info", "info"),
    ("LoaderCircle", "loader-circle"),
    ("Calendar", "calendar"),
    ("CalendarDays", "calendar-days"),
    ("Search", "search"),
    ("MoreHorizontal", "ellipsis"),
    ("MoreVertical", "ellipsis-vertical"),
    ("ArrowDown", "arrow-down"),
    ("ArrowUp", "arrow-up"),
    ("ArrowUpDown", "arrow-up-down"),
    ("ArrowLeft", "arrow-left"),
    ("ArrowRight", "arrow-right"),
    ("PanelLeft", "panel-left"),
    ("User", "user"),
    ("Paperclip", "paperclip"),
    ("Trash", "trash"),
    ("Pencil", "pencil"),
    ("Sparkles", "sparkles"),
    ("Copy", "copy"),
    ("CornerDownLeft", "corner-down-left"),
    ("Message", "message-circle"),
    ("Send", "send"),
    ("Circle", "circle"),
    ("Dot", "dot"),
    ("GripVertical", "grip-vertical"),
    ("GripHorizontal", "grip-horizontal"),
    ("FileText", "file-text"),
    ("RefreshCw", "refresh-cw"),
    ("Clock", "clock"),
]

LUCIDE_VERSION = "1.72.0"
LUCIDE_URL = "https://github.com/lucide-icons/lucide/releases/download/1.72.0/lucide-icons-1.72.0.zip"
LUCIDE_SHA256 = "402d838c5ecbd9d395c3824568317335440018d2ce581a828fffe16abcca1eb3"

FLAT_TOL = 0.02  # grid units; sub-pixel at every painted size
ARC_STEP_DEG = 3.0

NUM_RE = re.compile(r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?")
CMD_RE = re.compile(r"[MmLlHhVvCcSsQqTtAaZz]")
ARITY = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6, "S": 4, "Q": 4, "T": 2, "A": 7, "Z": 0}


def fail(msg):
    raise SystemExit(f"gen_icons: error: {msg}")


def tokenize(d):
    """Split path data into a stream of ('cmd', letter) / ('num', float)."""
    i, n = 0, len(d)
    while i < n:
        ch = d[i]
        if ch in " ,\t\r\n":
            i += 1
        elif ch.isalpha():
            if not CMD_RE.fullmatch(ch):
                fail(f"unsupported path command {ch!r} in {d!r}")
            yield ("cmd", ch)
            i += 1
        else:
            m = NUM_RE.match(d, i)
            if not m:
                fail(f"cannot parse number at {d[i:i+12]!r} in {d!r}")
            yield ("num", float(m.group(0)))
            i = m.end()


def parse_commands(d):
    """Yield (cmd, [args...]) with implicit repetition expanded."""
    cmd = None
    args = []
    pending = []
    for kind, val in tokenize(d):
        if kind == "cmd":
            if cmd is not None:
                yield from expand(cmd, args)
            cmd, args = val, []
        else:
            args.append(val)
    if cmd is not None:
        yield from expand(cmd, args)


def expand(cmd, args):
    arity = ARITY[cmd.upper()]
    if arity == 0:
        if args:
            fail(f"arguments for {cmd!r}")
        yield (cmd, [])
        return
    if len(args) % arity:
        fail(f"{cmd!r} has {len(args)} args, not a multiple of {arity}")
    first = True
    for i in range(0, len(args), arity):
        c = cmd
        # After the first moveto pair, further pairs are implicit lineto.
        if cmd in ("M", "m") and not first:
            c = "L" if cmd == "M" else "l"
        first = False
        yield (c, args[i : i + arity])


def cubic_points(p0, p1, p2, p3, tol=FLAT_TOL):
    """Adaptive subdivision; returns points excluding p0, including p3."""
    # Flatness: max distance of control points from the chord.
    dx, dy = p3[0] - p0[0], p3[1] - p0[1]
    chord = math.hypot(dx, dy)
    if chord < 1e-9:
        dev = max(math.hypot(p1[0] - p0[0], p1[1] - p0[1]),
                  math.hypot(p2[0] - p0[0], p2[1] - p0[1]))
    else:
        dev = 0.0
        for px, py in (p1, p2):
            dev = max(dev, abs((px - p0[0]) * dy - (py - p0[1]) * dx) / chord)
    if dev <= tol:
        return [p3]
    m01 = ((p0[0] + p1[0]) / 2, (p0[1] + p1[1]) / 2)
    m12 = ((p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2)
    m23 = ((p2[0] + p3[0]) / 2, (p2[1] + p3[1]) / 2)
    m012 = ((m01[0] + m12[0]) / 2, (m01[1] + m12[1]) / 2)
    m123 = ((m12[0] + m23[0]) / 2, (m12[1] + m23[1]) / 2)
    mid = ((m012[0] + m123[0]) / 2, (m012[1] + m123[1]) / 2)
    return cubic_points(p0, m01, m012, mid, tol) + cubic_points(
        mid, m123, m23, p3, tol
    )


def arc_points(p0, rx, ry, phi_deg, large, sweep, p1):
    """SVG endpoint->center arc (W3C F.6.5), sampled at <=ARC_STEP_DEG.

    Returns points excluding p0, including p1. Handles elliptical and
    rotated arcs; all arcs in the current set are circular with phi=0.
    """
    if rx == 0 or ry == 0:
        return [p1]
    rx, ry = abs(rx), abs(ry)
    phi = math.radians(phi_deg)
    cos_phi, sin_phi = math.cos(phi), math.sin(phi)
    dx = (p0[0] - p1[0]) / 2
    dy = (p0[1] - p1[1]) / 2
    x1p = cos_phi * dx + sin_phi * dy
    y1p = -sin_phi * dx + cos_phi * dy
    lam = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry)
    if lam > 1:
        s = math.sqrt(lam)
        rx, ry = rx * s, ry * s
    num = rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p
    den = rx * rx * y1p * y1p + ry * ry * x1p * x1p
    co = math.sqrt(max(0.0, num / den)) if den else 0.0
    if large == sweep:
        co = -co
    cxp = co * rx * y1p / ry
    cyp = -co * ry * x1p / rx
    cx = cos_phi * cxp - sin_phi * cyp + (p0[0] + p1[0]) / 2
    cy = sin_phi * cxp + cos_phi * cyp + (p0[1] + p1[1]) / 2

    def angle(ux, uy, vx, vy):
        dot = ux * vx + uy * vy
        det = ux * vy - uy * vx
        return math.atan2(det, dot)

    ux = (x1p - cxp) / rx
    uy = (y1p - cyp) / ry
    vx = (-x1p - cxp) / rx
    vy = (-y1p - cyp) / ry
    start = angle(1, 0, ux, uy)
    delta = angle(ux, uy, vx, vy)
    if not sweep and delta > 0:
        delta -= 2 * math.pi
    if sweep and delta < 0:
        delta += 2 * math.pi
    steps = max(2, math.ceil(abs(math.degrees(delta)) / ARC_STEP_DEG))
    pts = []
    for i in range(1, steps + 1):
        t = start + delta * i / steps
        ct, st = math.cos(t), math.sin(t)
        pts.append(
            (cx + rx * ct * cos_phi - ry * st * sin_phi,
             cy + rx * ct * sin_phi + ry * st * cos_phi)
        )
    pts[-1] = p1  # land exactly on the endpoint
    return pts


def flatten_path(d):
    """Convert one path `d` to a list of (closed, [points]) subpaths."""
    subpaths = []
    pts = None
    cur = (0.0, 0.0)
    start = (0.0, 0.0)
    prev_cmd = None
    prev_c2 = None  # second control point of previous cubic (for S)
    prev_q = None  # control point of previous quadratic (for T)

    def begin_subpath(p):
        nonlocal pts, start, cur
        if pts is not None:
            subpaths.append((False, pts))
        pts = [p]
        start = p
        cur = p

    for cmd, a in parse_commands(d):
        rel = cmd.islower()
        c = cmd.upper()
        if c == "M":
            p = (cur[0] + a[0], cur[1] + a[1]) if rel else (a[0], a[1])
            begin_subpath(p)
            prev_c2 = prev_q = None
        elif c == "Z":
            if pts is None:
                fail(f"Z with no subpath in {d!r}")
            subpaths.append((True, pts))
            pts = None
            cur = start
            prev_c2 = prev_q = None
        elif c == "L":
            p = (cur[0] + a[0], cur[1] + a[1]) if rel else (a[0], a[1])
            pts.append(p)
            cur = p
            prev_c2 = prev_q = None
        elif c == "H":
            p = (cur[0] + a[0], cur[1]) if rel else (a[0], cur[1])
            pts.append(p)
            cur = p
            prev_c2 = prev_q = None
        elif c == "V":
            p = (cur[0], cur[1] + a[0]) if rel else (cur[0], a[0])
            pts.append(p)
            cur = p
            prev_c2 = prev_q = None
        elif c == "C":
            if rel:
                p1 = (cur[0] + a[0], cur[1] + a[1])
                p2 = (cur[0] + a[2], cur[1] + a[3])
                p3 = (cur[0] + a[4], cur[1] + a[5])
            else:
                p1, p2, p3 = (a[0], a[1]), (a[2], a[3]), (a[4], a[5])
            pts.extend(cubic_points(cur, p1, p2, p3))
            prev_c2 = p2
            prev_q = None
            cur = p3
        elif c == "S":
            p1 = (2 * cur[0] - prev_c2[0], 2 * cur[1] - prev_c2[1]) \
                if prev_cmd in ("C", "S") and prev_c2 else cur
            if rel:
                p2 = (cur[0] + a[0], cur[1] + a[1])
                p3 = (cur[0] + a[2], cur[1] + a[3])
            else:
                p2, p3 = (a[0], a[1]), (a[2], a[3])
            pts.extend(cubic_points(cur, p1, p2, p3))
            prev_c2 = p2
            prev_q = None
            cur = p3
        elif c == "Q":
            if rel:
                q = (cur[0] + a[0], cur[1] + a[1])
                p2 = (cur[0] + a[2], cur[1] + a[3])
            else:
                q, p2 = (a[0], a[1]), (a[2], a[3])
            p1 = (cur[0] + 2 / 3 * (q[0] - cur[0]), cur[1] + 2 / 3 * (q[1] - cur[1]))
            p2c = (p2[0] + 2 / 3 * (q[0] - p2[0]), p2[1] + 2 / 3 * (q[1] - p2[1]))
            pts.extend(cubic_points(cur, p1, p2c, p2))
            prev_q = q
            prev_c2 = None
            cur = p2
        elif c == "T":
            q = (2 * cur[0] - prev_q[0], 2 * cur[1] - prev_q[1]) \
                if prev_cmd in ("Q", "T") and prev_q else cur
            p2 = (cur[0] + a[0], cur[1] + a[1]) if rel else (a[0], a[1])
            p1 = (cur[0] + 2 / 3 * (q[0] - cur[0]), cur[1] + 2 / 3 * (q[1] - cur[1]))
            p2c = (p2[0] + 2 / 3 * (q[0] - p2[0]), p2[1] + 2 / 3 * (q[1] - p2[1]))
            pts.extend(cubic_points(cur, p1, p2c, p2))
            prev_q = q
            prev_c2 = None
            cur = p2
        elif c == "A":
            rx, ry, phi, large, sweep = a[0], a[1], a[2], int(a[3]), int(a[4])
            p1 = (cur[0] + a[5], cur[1] + a[6]) if rel else (a[5], a[6])
            pts.extend(arc_points(cur, rx, ry, phi, large, sweep, p1))
            prev_c2 = prev_q = None
            cur = p1
        prev_cmd = c
    if pts is not None:
        subpaths.append((False, pts))
    return subpaths


def clean_points(pts):
    """Drop consecutive near-duplicates; keep at least the endpoints."""
    out = [pts[0]]
    for p in pts[1:]:
        if abs(p[0] - out[-1][0]) > 1e-4 or abs(p[1] - out[-1][1]) > 1e-4:
            out.append(p)
    return out


def fmt(v):
    s = f"{v:.3f}".rstrip("0").rstrip(".")
    if s in ("-0", ""):
        s = "0"
    return s + ".0" if "." not in s else s


def el_for_subpath(closed, pts):
    pts = clean_points(pts)
    if len(pts) == 1:
        pts = [pts[0], pts[0]]
    body = ", ".join(f"[{fmt(x)}, {fmt(y)}]" for x, y in pts)
    if not closed and len(pts) == 2:
        return f"El::Line([{fmt(pts[0][0])}, {fmt(pts[0][1])}], [{fmt(pts[1][0])}, {fmt(pts[1][1])}])"
    kind = "El::PolyClosed" if closed else "El::Poly"
    return f"{kind}(&[{body}])"


def convert_svg(path):
    root = ET.parse(path).getroot()
    els = []
    for el in root:
        tag = el.tag.split("}")[-1]
        if tag == "path":
            for closed, pts in flatten_path(el.get("d", "")):
                els.append(el_for_subpath(closed, pts))
        elif tag == "line":
            els.append(
                f"El::Line([{fmt(float(el.get('x1')))}, {fmt(float(el.get('y1')))}], "
                f"[{fmt(float(el.get('x2')))}, {fmt(float(el.get('y2')))}])"
            )
        elif tag == "circle":
            els.append(
                f"El::Circle([{fmt(float(el.get('cx')))}, {fmt(float(el.get('cy')))}], "
                f"{fmt(float(el.get('r')))})"
            )
        elif tag == "rect":
            rx = float(el.get("rx", "0"))
            ry = el.get("ry")
            if ry is not None and abs(float(ry) - rx) > 1e-6:
                fail(f"{path}: rect with rx != ry ({rx} vs {ry})")
            els.append(
                f"El::Rect([{fmt(float(el.get('x', '0')))}, {fmt(float(el.get('y', '0')))}], "
                f"[{fmt(float(el.get('width')))}, {fmt(float(el.get('height')))}], {fmt(rx)})"
            )
        elif tag in ("polygon", "polyline"):
            raw = [float(v) for v in re.findall(r"[-+]?(?:\d+\.?\d*|\.\d+)", el.get("points", ""))]
            pts = [(raw[i], raw[i + 1]) for i in range(0, len(raw) - 1, 2)]
            els.append(el_for_subpath(tag == "polygon", pts))
        else:
            fail(f"{path}: unsupported element <{tag}>")
        if el.get("fill") not in (None, "none"):
            fail(f"{path}: filled element <{tag}> fill={el.get('fill')!r} (fills unsupported)")
    if not els:
        fail(f"{path}: no drawable elements")
    return els


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--icons-dir", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    icons_dir = Path(args.icons_dir)
    if not icons_dir.is_dir():
        fail(f"icons dir not found: {icons_dir}")

    chunks = []
    total_els = 0
    for variant, name in ICONS:
        svg = icons_dir / f"{name}.svg"
        if not svg.is_file():
            fail(f"missing {svg} for variant {variant}")
        els = convert_svg(svg)
        total_els += len(els)
        body = ",\n        ".join(els)
        chunks.append(f"pub(crate) static {variant.upper()}: &[El] = &[\n        {body},\n    ];")

    header = f"""//! Generated Lucide icon geometry — DO NOT EDIT BY HAND.
//!
//! Source: Lucide {LUCIDE_VERSION} official release
//!   {LUCIDE_URL}
//!   zip SHA256: {LUCIDE_SHA256}
//! Licence: ISC (see `LICENSE-Lucide.txt` beside this file; the licence text
//! also covers the Feather-derived icons under MIT).
//!
//! Regenerate with:
//!   python3 src/icons/gen_icons.py --icons-dir <unzipped lucide icons/> \\
//!       --out src/icons/data.rs && cargo fmt -- src/icons/data.rs
//!
//! Each icon's SVG elements were converted to `El` values by the generator
//! (paths flattened to polylines, arcs sampled, cubics subdivided). See
//! `gen_icons.py` for the conversion rules and the variant -> Lucide name
//! mapping. To add an icon, add one line to `ICONS` in the generator, add
//! the variant to `Icon`, and re-run it.

// Generated data: coordinates may coincidentally approximate float
// constants (clippy's approx_constant lint).
#![allow(clippy::approx_constant)]

use super::icons::{{El, Icon}};

"""
    matches = "\n".join(
        f"            Icon::{v} => {v.upper()}," for v, _ in ICONS
    )
    names = "\n".join(
        f'            Icon::{v} => "{n}",' for v, n in ICONS
    )
    from_names = "\n".join(
        f'        "{n}" => Icon::{v},' for v, n in ICONS
    )
    all_list = ", ".join(f"Icon::{v}" for v, _ in ICONS)
    body = header + "\n\n".join(chunks) + f"""

/// Static element data for one icon (no allocation).
pub(crate) fn elements(icon: Icon) -> &'static [El] {{
    match icon {{
{matches}
    }}
}}

/// The Lucide kebab-case name this icon's data comes from.
pub(crate) fn lucide_name(icon: Icon) -> &'static str {{
    match icon {{
{names}
    }}
}}

/// Look up an icon by its Lucide kebab-case name.
pub(crate) fn from_lucide_name(name: &str) -> Option<Icon> {{
    Some(match name {{
{from_names}
        _ => return None,
    }})
}}

/// Every icon in the crate, in enum order.
pub(crate) const ALL: &[Icon] = &[{all_list}];
"""
    out = Path(args.out)
    out.write_text(body)
    print(f"wrote {out}: {len(ICONS)} icons, {total_els} elements, {len(body)} bytes")


if __name__ == "__main__":
    main()
