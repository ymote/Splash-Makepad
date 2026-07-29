#!/usr/bin/env python3
"""Build labelled contact sheets from a directory of QA screenshots.

ImageMagick's `montage -label` needs a ghostscript delegate that is often not
installed; PIL is, and this gives control over the label and the blank-screen
flagging that actually matters when scanning 108 screens.

Also reports, per screen, the fraction of pixels that are not the background
colour — a screen that translated fine but rendered nothing comes out near 0
and is worth looking at before anything else.
"""

import sys
from collections import Counter
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

COLS, ROWS = 6, 2
THUMB_W = 300


def load_font(size):
    for p in (
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ):
        if Path(p).exists():
            try:
                return ImageFont.truetype(p, size)
            except OSError:
                pass
    return ImageFont.load_default()


def ink_fraction(img):
    """Fraction of pixels differing from the modal (background) colour."""
    small = img.convert("RGB").resize((80, 160))
    px = list(small.getdata())
    bg = Counter(px).most_common(1)[0][0]
    off = sum(
        1
        for p in px
        if abs(p[0] - bg[0]) + abs(p[1] - bg[1]) + abs(p[2] - bg[2]) > 24
    )
    return off / len(px)


def main():
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "tools/qa-shots")
    shots = sorted(p for p in out.glob("*.png") if not p.name.startswith("sheet-"))
    if not shots:
        print("no screenshots found", file=sys.stderr)
        return 1

    font = load_font(15)
    blank = []
    per_sheet = COLS * ROWS

    for sheet_i in range(0, len(shots), per_sheet):
        chunk = shots[sheet_i : sheet_i + per_sheet]
        thumbs = []
        for p in chunk:
            im = Image.open(p)
            frac = ink_fraction(im)
            if frac < 0.02:
                blank.append((p.stem, frac))
            h = int(im.height * THUMB_W / im.width)
            thumbs.append((p.stem, im.resize((THUMB_W, h)), frac))

        cell_h = max(t[1].height for t in thumbs) + 30
        sheet = Image.new(
            "RGB", (COLS * (THUMB_W + 10) + 10, ROWS * (cell_h + 10) + 10), "#222"
        )
        d = ImageDraw.Draw(sheet)
        for i, (name, im, frac) in enumerate(thumbs):
            cx = 10 + (i % COLS) * (THUMB_W + 10)
            cy = 10 + (i // COLS) * (cell_h + 10)
            sheet.paste(im, (cx, cy + 24))
            colour = "#ff6b6b" if frac < 0.02 else "white"
            d.text((cx, cy + 4), name[:42], fill=colour, font=font)

        name = out / f"sheet-{sheet_i // per_sheet + 1:02d}.png"
        sheet.save(name)
        print(name.name)

    if blank:
        print("\nnear-blank screens (ink < 2%):")
        for n, f in blank:
            print(f"  {f * 100:5.2f}%  {n}")
    else:
        print("\nno blank screens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
