#!/usr/bin/env python3
"""Render the share card with Pillow (authoring only, never part of the app build).

Usage: python3 web/scripts/render-social-preview.py --font /path/to/font.ttf
For a font collection, select faces with --regular-index and --bold-index.
The checked-in image uses Avenir Next, regular index 7 and bold index 0.
"""

import argparse
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


BACKGROUND = '#edf0f8'
INK = '#202c49'
MUTED = '#596580'
GRID = '#cbd3e5'
ACCENT = '#5949b7'
TEAL = '#32877c'
AMBER = '#c48a34'
SCALE = 2


def render(font_path, regular_index, bold_index, output):
    image = Image.new('RGB', (1200 * SCALE, 630 * SCALE), BACKGROUND)
    draw = ImageDraw.Draw(image)

    def line(points, fill, width=1):
        draw.line([(x * SCALE, y * SCALE) for x, y in points], fill=fill, width=width * SCALE)

    def circle(x, y, radius, fill):
        draw.ellipse(((x - radius) * SCALE, (y - radius) * SCALE,
                      (x + radius) * SCALE, (y + radius) * SCALE), fill=fill)

    def text(x, y, label, size, fill=INK, bold=False):
        font = ImageFont.truetype(font_path, size * SCALE,
                                  index=bold_index if bold else regular_index)
        draw.text((x * SCALE, y * SCALE), label, font=font, fill=fill, anchor='lt')

    # The application's lattice mark, enlarged without adding interface chrome.
    for offset in (0, 9, 19, 28):
        line([(66 + offset, 65), (66 + offset, 93)], ACCENT, 3)
        line([(66, 65 + offset), (94, 65 + offset)], ACCENT, 3)

    text(62, 174, 'BottomUpLab', 82, bold=True)
    text(66, 301, 'Computational social science', 34)
    text(66, 352, '& AI alignment', 34)
    text(66, 511, 'Nate Douglas', 32, MUTED)

    # An illustrative agent lattice: relationships, not a claimed experiment.
    origin_x, origin_y, spacing, steps = 780, 116, 58, 6
    end_x = origin_x + steps * spacing
    end_y = origin_y + steps * spacing
    for index in range(steps + 1):
        line([(origin_x + index * spacing, origin_y),
              (origin_x + index * spacing, end_y)], GRID)
        line([(origin_x, origin_y + index * spacing),
              (end_x, origin_y + index * spacing)], GRID)

    # Deterministic positions keep the composition editable and reproducible.
    agents = [(0, 1, TEAL), (0, 5, ACCENT), (1, 0, ACCENT),
              (1, 3, AMBER), (1, 6, TEAL), (2, 1, TEAL),
              (2, 4, ACCENT), (3, 2, ACCENT), (3, 5, TEAL),
              (4, 0, AMBER), (4, 3, TEAL), (4, 6, ACCENT),
              (5, 1, ACCENT), (5, 4, AMBER), (6, 2, TEAL),
              (6, 5, ACCENT)]
    for column, row, color in agents:
        circle(origin_x + column * spacing, origin_y + row * spacing, 12, color)

    image.resize((1200, 630), Image.Resampling.LANCZOS).save(output, optimize=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--font', required=True, help='Path to an installed .ttf, .otf or .ttc font')
    parser.add_argument('--regular-index', type=int, default=0)
    parser.add_argument('--bold-index', type=int, default=0)
    parser.add_argument('--output', type=Path,
                        default=Path(__file__).resolve().parents[1] / 'public' / 'social-preview.png')
    args = parser.parse_args()
    render(args.font, args.regular_index, args.bold_index, args.output)
