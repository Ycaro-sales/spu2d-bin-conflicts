#!/usr/bin/env python3
"""Compare greedy vs BRKGA solutions on speed and quality.

Reads `.solution` JSON files from ``solutions/greedy/`` and ``solutions/brkga/``.
The instance is the file stem and the method is the parent directory name (neither
is stored inside the JSON). Solutions are joined per instance on the shared stem.

Produces two figures under ``solutions/``:
  * ``comparison_quality.png`` - greedy vs BRKGA bins-used scatter (with y=x line)
    plus a histogram of (greedy - brkga) bins.
  * ``comparison_speed.png``   - per-instance runtime for both methods (log scale).

and prints a summary table (mean bins, BRKGA win/tie/loss, mean runtime).
"""

import glob
import json
import os
import statistics
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

SOLUTIONS = "solutions"


def load(method):
    """Return {instance_stem: doc} for one method subfolder."""
    out = {}
    for path in glob.glob(os.path.join(SOLUTIONS, method, "*.solution")):
        stem = os.path.splitext(os.path.basename(path))[0]
        with open(path) as f:
            out[stem] = json.load(f)
    return out


def main():
    greedy = load("greedy")
    brkga = load("brkga")
    instances = sorted(set(greedy) & set(brkga))
    if not instances:
        raise SystemExit(
            "no matching solutions found; run the Rust binary first to populate "
            "solutions/greedy and solutions/brkga"
        )

    g_bins = [greedy[i]["bins_used"] for i in instances]
    b_bins = [brkga[i]["bins_used"] for i in instances]
    g_time = [greedy[i]["runtime_seconds"] for i in instances]
    b_time = [brkga[i]["runtime_seconds"] for i in instances]

    wins = sum(b < g for g, b in zip(g_bins, b_bins))
    ties = sum(b == g for g, b in zip(g_bins, b_bins))
    losses = sum(b > g for g, b in zip(g_bins, b_bins))

    print(f"instances compared: {len(instances)}")
    print(f"mean bins   - greedy: {statistics.mean(g_bins):.2f}  "
          f"brkga: {statistics.mean(b_bins):.2f}")
    print(f"BRKGA vs greedy (bins) - wins: {wins}  ties: {ties}  losses: {losses}")
    print(f"mean runtime - greedy: {statistics.mean(g_time):.4f}s  "
          f"brkga: {statistics.mean(b_time):.4f}s")

    # --- Quality figure ---
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    # jitter identical points slightly so overlaps are visible
    import random
    rng = random.Random(0)
    jx = [g + rng.uniform(-0.12, 0.12) for g in g_bins]
    jy = [b + rng.uniform(-0.12, 0.12) for b in b_bins]
    ax1.scatter(jx, jy, alpha=0.6, edgecolor="k", linewidth=0.3)
    lo = min(min(g_bins), min(b_bins)) - 1
    hi = max(max(g_bins), max(b_bins)) + 1
    ax1.plot([lo, hi], [lo, hi], "r--", label="y = x (tie)")
    ax1.set_xlim(lo, hi)
    ax1.set_ylim(lo, hi)
    ax1.set_xlabel("greedy bins used")
    ax1.set_ylabel("BRKGA bins used")
    ax1.set_title("Quality: bins used (below line = BRKGA better)")
    ax1.legend()
    ax1.set_aspect("equal")

    diffs = [g - b for g, b in zip(g_bins, b_bins)]
    bins_range = range(min(diffs), max(diffs) + 2)
    ax2.hist(diffs, bins=list(bins_range), align="left",
             color="steelblue", edgecolor="k")
    ax2.axvline(0, color="r", linestyle="--")
    ax2.set_xlabel("bins saved by BRKGA (greedy - brkga)")
    ax2.set_ylabel("instances")
    ax2.set_title("Quality improvement distribution")

    fig.tight_layout()
    qpath = os.path.join(SOLUTIONS, "comparison_quality.png")
    fig.savefig(qpath, dpi=120)
    print(f"wrote {qpath}")

    # --- Speed figure ---
    fig, ax = plt.subplots(figsize=(12, 5))
    order = sorted(range(len(instances)), key=lambda k: b_time[k])
    xs = range(len(instances))
    ax.plot(xs, [g_time[k] for k in order], "o-", ms=3, label="greedy")
    ax.plot(xs, [b_time[k] for k in order], "o-", ms=3, label="brkga")
    ax.set_yscale("log")
    ax.set_xlabel("instance (sorted by BRKGA runtime)")
    ax.set_ylabel("runtime (s, log scale)")
    ax.set_title("Speed: per-instance runtime")
    ax.legend()
    fig.tight_layout()
    spath = os.path.join(SOLUTIONS, "comparison_speed.png")
    fig.savefig(spath, dpi=120)
    print(f"wrote {spath}")


if __name__ == "__main__":
    main()
