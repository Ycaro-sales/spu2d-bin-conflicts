#!/usr/bin/env python3
"""Visualize a packed 2D bin-packing solution with matplotlib.patches.

Reads a ``.solution`` JSON file (the instance is the file stem, the method is the
parent directory name; neither is stored in the JSON). Draws one subplot per bin:
the bin outline plus every placed item as a Rectangle, colored per item index,
labeled with the index, and hatched when the item was rotated 90 degrees.

It also draws the instance's conflict graph as a node-link diagram (one node per
item, one edge per conflicting pair). The conflict graph is not stored in the
solution; it lives in the instance JSON (top-level ``"Conflicts"``), so the
instance is auto-located by file stem under ``instances_conflict/`` (then
``instances/``), or passed explicitly via ``--instance``. Item nodes that were
actually placed ("used") are colored to match their bin rectangle; unused items
are greyed.

Usage:
    python visualize_solution.py path/to/file.solution [--out OUTPUT.png] \\
        [--instance path/to/instance.json]
"""

import argparse
import json
import math
import os
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.lines import Line2D
from matplotlib.patches import Rectangle

# Repo root, so instance auto-location works regardless of the current directory.
REPO_ROOT = Path(__file__).resolve().parent.parent

USED_GREY = "#bdbdbd"


def identity(path):
    """(instance stem, method) derived from the path."""
    stem = os.path.splitext(os.path.basename(path))[0]
    method = os.path.basename(os.path.dirname(os.path.abspath(path)))
    return stem, method


def find_instance(stem, explicit=None):
    """Locate the instance JSON for a solution stem, or None if not found.

    An explicit path wins. Otherwise search ``instances_conflict/`` (the variants
    that carry conflict data) and then ``instances/`` for ``<stem>.json``.
    """
    if explicit:
        p = Path(explicit)
        return p if p.is_file() else None
    for root in ("instances_conflict", "instances"):
        matches = sorted((REPO_ROOT / root).rglob(f"{stem}.json"))
        if matches:
            return matches[0]
    return None


def load_conflicts(path, num_items):
    """Return the conflict edge list, keeping only in-range item-index pairs.

    Returns ``[]`` when the instance is missing/unreadable or has no conflicts.
    """
    if path is None:
        return []
    try:
        doc = json.loads(Path(path).read_text())
    except (OSError, json.JSONDecodeError):
        return []
    edges = []
    for pair in doc.get("Conflicts", []):
        if len(pair) != 2:
            continue
        a, b = pair
        if 0 <= a < num_items and 0 <= b < num_items and a != b:
            edges.append((a, b))
    return edges


def draw_conflict_graph(ax, num_items, edges, placed, cmap, instance_found):
    """Draw the conflict graph as a node-link diagram on a circular layout.

    Used (placed) nodes get their per-item bin color; unused nodes are greyed.
    Edges with both endpoints placed are drawn darker (the conflicts the packing
    actually had to honor).
    """
    n = max(num_items, 1)
    angles = np.linspace(0.0, 2.0 * np.pi, n, endpoint=False) + np.pi / 2.0
    pos = np.column_stack((np.cos(angles), np.sin(angles)))

    for a, b in edges:
        both_used = a in placed and b in placed
        ax.plot(
            [pos[a, 0], pos[b, 0]],
            [pos[a, 1], pos[b, 1]],
            color="#555555" if both_used else "#dddddd",
            linewidth=1.0 if both_used else 0.7,
            zorder=1,
        )

    for i in range(num_items):
        used = i in placed
        color = cmap((i % 20) / 20.0) if used else USED_GREY
        ax.scatter(
            pos[i, 0],
            pos[i, 1],
            s=260,
            facecolor=color,
            edgecolor="black",
            linewidth=0.6,
            alpha=0.9 if used else 0.6,
            zorder=2,
        )
        ax.text(
            pos[i, 0],
            pos[i, 1],
            str(i),
            ha="center",
            va="center",
            fontsize=8,
            fontweight="bold",
            zorder=3,
        )

    ax.set_xlim(-1.25, 1.25)
    ax.set_ylim(-1.25, 1.25)
    ax.set_aspect("equal")
    ax.axis("off")

    title = (
        f"conflict graph — {len(edges)} conflicts, {len(placed)}/{num_items} used"
    )
    if not instance_found:
        title += " (instance not found — nodes only)"
    ax.set_title(title, fontsize=11)

    ax.legend(
        handles=[
            Line2D([], [], marker="o", linestyle="", markersize=8,
                   markerfacecolor="#4878d0", markeredgecolor="black", label="used"),
            Line2D([], [], marker="o", linestyle="", markersize=8,
                   markerfacecolor=USED_GREY, markeredgecolor="black", label="unused"),
        ],
        loc="upper left",
        bbox_to_anchor=(1.0, 1.0),
        fontsize=8,
        framealpha=0.6,
    )


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("solution", help="path to a .solution JSON file")
    ap.add_argument("--out", help="output PNG path (default: alongside input)")
    ap.add_argument(
        "--instance",
        help="path to the instance JSON (default: auto-locate by file stem)",
    )
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

    # Items that were actually placed (used), derived from the bins themselves.
    placed = {it["index"] for b in bins for it in b["items"]}

    instance_path = find_instance(instance, args.instance)
    edges = load_conflicts(instance_path, n_items)

    fig = plt.figure(figsize=(4 * ncols, 4 * nrows + 4))
    top_sf, bottom_sf = fig.subfigures(2, 1, height_ratios=[1, 2])

    graph_ax = top_sf.subplots()
    # Leave headroom for the figure suptitle so it does not overlap the graph.
    top_sf.subplots_adjust(top=0.74)
    draw_conflict_graph(
        graph_ax, n_items, edges, placed, cmap, instance_path is not None
    )

    axes = bottom_sf.subplots(nrows, ncols, squeeze=False)
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
    fig.savefig(out, dpi=120, bbox_inches="tight")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
