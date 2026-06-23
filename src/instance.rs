use std::path::Path;

use serde::Deserialize;

use crate::conflict_graph::ConflictGraph;
use crate::item::Item;

#[derive(Clone, Debug)]
pub struct Instance {
    pub items: Vec<Item>,
    pub width: u32,
    pub height: u32,
    /// Conflict graph keyed by item index (position in `items`).
    pub conflicts: ConflictGraph<usize>,
}

// --- Format A: { "Objects": [...], "Items": [...] } ---

#[derive(Deserialize)]
struct ObjectsFormat {
    #[serde(rename = "Objects")]
    objects: Vec<RawObject>,
    #[serde(rename = "Items")]
    items: Vec<RawDemandItem>,
    #[serde(rename = "Conflicts")]
    conflicts: Option<Vec<(usize, usize)>>,
}

#[derive(Deserialize)]
struct RawObject {
    #[serde(rename = "Length")]
    length: u32,
    #[serde(rename = "Height")]
    height: u32,
}

#[derive(Deserialize)]
struct RawDemandItem {
    #[serde(rename = "Length")]
    length: u32,
    #[serde(rename = "Height")]
    height: u32,
    #[serde(rename = "Demand")]
    demand: u32,
    #[serde(rename = "Client")]
    client: Option<u32>,
}

// --- Format B: 2D-OPP { "Container": {...}, "ItemTypes": [...] } ---

#[derive(Deserialize)]
struct ItemTypesFormat {
    #[serde(rename = "Container")]
    container: RawContainer,
    #[serde(rename = "ItemTypes")]
    item_types: Vec<RawItemType>,
    #[serde(rename = "Conflicts")]
    conflicts: Option<Vec<(usize, usize)>>,
}

#[derive(Deserialize)]
struct RawContainer {
    #[serde(rename = "Length")]
    length: u32,
    #[serde(rename = "Width")]
    width: u32,
}

#[derive(Deserialize)]
struct RawItemType {
    #[serde(rename = "Length")]
    length: u32,
    #[serde(rename = "Width")]
    width: u32,
    #[serde(rename = "Amount")]
    amount: u32,
    #[serde(rename = "Client")]
    client: Option<u32>,
}

/// Assemble an `Instance`, building the item-indexed conflict graph.
///
/// Every item index `0..items.len()` is added as a node (so conflict-free items
/// are still present for coloring), then each `(a, b)` edge is added.
fn assemble(width: u32, height: u32, items: Vec<Item>, edges: Vec<(usize, usize)>) -> Instance {
    let mut conflicts = ConflictGraph::new();
    for i in 0..items.len() {
        conflicts.add_node(i);
    }
    for (a, b) in edges {
        conflicts.add_conflict(a, b);
    }
    Instance {
        items,
        width,
        height,
        conflicts,
    }
}

/// Build an `Instance` from a parsed instance JSON document.
///
/// Two schemas are supported: the `Objects`/`Items` format and the 2D-OPP
/// `Container`/`ItemTypes` format. In both, `Length` maps to `width` and the
/// other planar dimension (`Height` / `Width`) maps to `height`. Each item is
/// expanded by its `Demand`/`Amount`.
///
/// When present, the `Client` field on each item populates `Item.client`, and the
/// top-level `Conflicts` (a list of item-index pairs) populates a
/// `ConflictGraph<usize>` keyed by item index. Absent fields default to
/// `client = 0` and a conflict-free graph (one node per item).
pub fn instance_from_json(json: serde_json::Value) -> Instance {
    if json.get("ItemTypes").is_some() {
        let raw: ItemTypesFormat =
            serde_json::from_value(json).expect("invalid 2D-OPP (ItemTypes) instance");
        let mut items = Vec::new();
        for it in &raw.item_types {
            for _ in 0..it.amount {
                items.push(Item {
                    width: it.length,
                    height: it.width,
                    client: it.client.unwrap_or(0),
                });
            }
        }
        assemble(
            raw.container.length,
            raw.container.width,
            items,
            raw.conflicts.unwrap_or_default(),
        )
    } else {
        let raw: ObjectsFormat =
            serde_json::from_value(json).expect("invalid (Objects/Items) instance");
        let object = raw.objects.first().expect("instance has no object");
        let (width, height) = (object.length, object.height);
        let mut items = Vec::new();
        for it in &raw.items {
            for _ in 0..it.demand {
                items.push(Item {
                    width: it.length,
                    height: it.height,
                    client: it.client.unwrap_or(0),
                });
            }
        }
        assemble(width, height, items, raw.conflicts.unwrap_or_default())
    }
}

/// Read and parse an instance file from disk.
pub fn instance_from_path(path: impl AsRef<Path>) -> Instance {
    let file = std::fs::File::open(path).expect("cannot open instance file");
    let json =
        serde_json::from_reader(std::io::BufReader::new(file)).expect("invalid instance json");
    instance_from_json(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_objects_items_format() {
        let inst = instance_from_path(
            "instances/OPP/BPP-Subproblems/\
             173e0w100h100n21dx-25-50-100_19_24_35dy-25-50-100_12_23_32t300.json",
        );
        assert_eq!(inst.width, 100);
        assert_eq!(inst.height, 100);
        assert_eq!(inst.items.len(), 21); // n21 in the file name
        // First item in the file: Length 33, Height 28.
        assert_eq!(inst.items[0].width, 33);
        assert_eq!(inst.items[0].height, 28);
        // Original file has no conflict data: one node per item, no edges.
        assert_eq!(inst.conflicts.adjacency.len(), inst.items.len());
        assert_eq!(inst.conflicts.degree(&0), 0);
    }

    #[test]
    fn parses_item_types_format() {
        let inst = instance_from_path(
            "instances/OPP/BPP-Subproblems/\
             195e0w100h100n40dx-25-50-100_7_13_35dy-25-50-100_11_16_35t300.json",
        );
        assert_eq!(inst.width, 100);
        assert_eq!(inst.height, 100);
        assert_eq!(inst.items.len(), 40); // NumberItemTypes, each Amount 1
        // First ItemType: Length 1, Width 3.
        assert_eq!(inst.items[0].width, 1);
        assert_eq!(inst.items[0].height, 3);
        assert_eq!(inst.conflicts.adjacency.len(), inst.items.len());
        assert_eq!(inst.conflicts.degree(&0), 0);
    }

    #[test]
    fn reads_clients_and_item_conflicts() {
        let doc = serde_json::json!({
            "Objects": [{ "Length": 100, "Height": 100 }],
            "Items": [
                { "Length": 10, "Height": 20, "Demand": 1, "Client": 0 },
                { "Length": 5,  "Height": 5,  "Demand": 1, "Client": 1 },
                { "Length": 7,  "Height": 3,  "Demand": 1, "Client": 1 },
            ],
            "NumClients": 2,
            "Conflicts": [[0, 1], [1, 2]],
        });
        let inst = instance_from_json(doc);

        assert_eq!(inst.items.len(), 3);
        assert_eq!(inst.items[0].client, 0);
        assert_eq!(inst.items[1].client, 1);
        assert_eq!(inst.items[2].client, 1);

        // Item-indexed conflict graph: nodes 0,1,2; edges (0,1) and (1,2).
        assert_eq!(inst.conflicts.adjacency.len(), 3);
        assert_eq!(inst.conflicts.degree(&1), 2);
        assert_eq!(inst.conflicts.degree(&0), 1);
        assert_eq!(inst.conflicts.degree(&2), 1);
        assert!(inst.conflicts.adjacency[&0].contains(&1));
        assert!(inst.conflicts.adjacency[&1].contains(&2));
    }

    #[test]
    fn defaults_when_conflict_fields_absent() {
        let doc = serde_json::json!({
            "Objects": [{ "Length": 50, "Height": 50 }],
            "Items": [
                { "Length": 10, "Height": 20, "Demand": 2 },
                { "Length": 5,  "Height": 5,  "Demand": 1 },
            ],
        });
        let inst = instance_from_json(doc);

        // Demand expands to 3 items, all client 0, conflict-free.
        assert_eq!(inst.items.len(), 3);
        assert!(inst.items.iter().all(|i| i.client == 0));
        assert_eq!(inst.conflicts.adjacency.len(), 3);
        assert!((0..3).all(|i| inst.conflicts.degree(&i) == 0));
    }
}
