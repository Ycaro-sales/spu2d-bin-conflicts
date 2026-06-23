#!/usr/bin/env python3
"""Visualize a packed 2D bin-packing solution with matplotlib.patches.

Reads a ``.solution`` JSON file (the instance is the file stem, the method is the
parent directory name; neither is stored in the JSON). Draws one subplot per bin:
the bin outline plus every placed item as a Rectangle, colored per item index,
labeled with the index, and hatched when the item was rotated 90 degrees.

Usage:
    python visualize_solution.py path/to/file.solution [--out OUTPUT.png]
"""

import argparse
import json
import math
import os
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle


def identity(path):
    """(instance stem, method) derived from the path."""
    stem = os.path.splitext(os.path.basename(path))[0]
    method = os.path.basename(os.path.dirname(os.path.abspath(path)))
    return stem, method


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("solution", help="path to a .solution JSON file")
    ap.add_argument("--out", help="output PNG path (default: alongside input)")
    args = ap.parse_args()

    with open(args.solution) as f:
        doc = json.load(f)

    instance, method = identity(args.solution)
    out = args.out or os.path.splitext(args.solution)[0] + ".png"

    bins = doc["bins"]
    bw, bh = doc["bin_width"], doc["bin_height"]
    n_bins = max(len(bins), 1)
    ncols = math.ceil(math.sqrt(n_bins))
    nrows = math.ceil(n_bins / ncols)

    # Stable color per item index across all bins.
    cmap = plt.get_cmap("tab20")
    n_items = max(doc.get("num_items", 1), 1)

    fig, axes = plt.subplots(
        nrows, ncols, figsize=(4 * ncols, 4 * nrows), squeeze=False
    )
    flat = [ax for row in axes for ax in row]

    for b, ax in enumerate(flat):
        if b >= len(bins):
            ax.axis("off")
            continue
        ax.add_patch(
            Rectangle((0, 0), bw, bh, fill=False, edgecolor="black", linewidth=1.5)
        )
        for it in bins[b]["items"]:
            color = cmap((it["index"] % 20) / 20.0)
            hatch = "//" if it.get("rotated") else None
            ax.add_patch(
                Rectangle(
                    (it["x"], it["y"]),
                    it["width"],
                    it["height"],
                    facecolor=color,
                    edgecolor="black",
                    linewidth=0.6,
                    alpha=0.75,
                    hatch=hatch,
                )
            )
            ax.text(
                it["x"] + it["width"] / 2,
                it["y"] + it["height"] / 2,
                str(it["index"]),
                ha="center",
                va="center",
                fontsize=8,
                fontweight="bold",
            )
        ax.set_xlim(0, bw)
        ax.set_ylim(0, bh)
        ax.set_aspect("equal")
        ax.set_title(f"bin {b}", fontsize=10)

    unplaced = doc.get("unplaced", [])
    title = f"{instance} — {method} — {doc.get('bins_used', len(bins))} bins"
    if unplaced:
        title += f" ({len(unplaced)} unplaced)"
    fig.suptitle(title, fontsize=13)
    fig.tight_layout(rect=[0, 0, 1, 0.97])
    fig.savefig(out, dpi=120)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
