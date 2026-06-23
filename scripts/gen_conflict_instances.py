#!/usr/bin/env python3
"""Augment every OPP/BPP instance with a client per item and a conflict graph.

For each source instance in ``instances/`` this emits one variant per conflict
density in ``DENSITIES``, mirroring the source tree under ``instances_conflict/``.
Each variant keeps all original fields and adds:

  * a ``"Client"`` field on every item entry (in ``Items`` or ``ItemTypes``),
  * a top-level ``"NumClients"`` and ``"Conflicts"`` (an edge list over ITEM indices).

Items are grouped into ``ceil(n_items / 3)`` clients (a per-item attribute only).
Conflicts are between items by index, independent of client: each unordered item
pair conflicts with probability ``p`` (Erdos-Renyi over item indices). Item indices
refer to item entry position, which equals the expanded item index because every
entry in this dataset has unit Demand/Amount. The RNG is seeded per
(instance name, density) so output is reproducible.
"""

import json
import math
import random
from pathlib import Path

SRC = Path("instances")
DST = Path("instances_conflict")
DENSITIES = [0.1, 0.3, 0.5]


def item_list(doc):
    """Return the list of item entries for either supported schema."""
    return doc["ItemTypes"] if "ItemTypes" in doc else doc["Items"]


def augment(doc, name, p):
    out = json.loads(json.dumps(doc))  # deep copy so the source dict is untouched
    items = item_list(out)
    n = len(items)
    n_clients = max(1, math.ceil(n / 3))
    rng = random.Random(f"{name}|{p}")

    for it in items:
        it["Client"] = rng.randrange(n_clients)

    # Conflicts are between items by index, independent of client: each unordered
    # item pair conflicts with probability p (Erdos-Renyi over item indices).
    conflicts = [
        [i, j]
        for i in range(n)
        for j in range(i + 1, n)
        if rng.random() < p
    ]

    out["NumClients"] = n_clients
    out["Conflicts"] = conflicts
    return out


def main():
    files = sorted(SRC.rglob("*.json"))
    count = 0
    for f in files:
        doc = json.loads(f.read_text())
        rel = f.relative_to(SRC)
        for p in DENSITIES:
            aug = augment(doc, f.stem, p)
            dst = DST / rel.with_name(f"{f.stem}_c{p}.json")
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_text(json.dumps(aug, indent=2))
            count += 1
    print(f"wrote {count} instances from {len(files)} sources into {DST}/")


if __name__ == "__main__":
    main()
