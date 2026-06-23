use std::path::Path;

use serde::Serialize;

use crate::decoder::Solution;

/// A serializable solution document. The instance is identified by the file name
/// (`<stem>.solution`) and the method by the parent subfolder (`solutions/greedy`
/// vs `solutions/brkga`), so neither is stored in the body.
#[derive(Serialize)]
pub struct SolutionDoc {
    pub bin_width: u32,
    pub bin_height: u32,
    pub num_items: usize,
    pub bins_used: usize,
    pub unplaced: Vec<usize>,
    pub runtime_seconds: f64,
    pub bins: Vec<BinDoc>,
}

#[derive(Serialize)]
pub struct BinDoc {
    pub items: Vec<PlacementDoc>,
}

#[derive(Serialize)]
pub struct PlacementDoc {
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rotated: bool,
}

/// Build a serializable document from a decoded `Solution`.
pub fn to_doc(
    sol: &Solution,
    bin_w: u32,
    bin_h: u32,
    num_items: usize,
    runtime_seconds: f64,
) -> SolutionDoc {
    let bins = sol
        .bins
        .iter()
        .map(|bin| {
            let mut items: Vec<PlacementDoc> = bin
                .placements()
                .iter()
                .map(|(&index, p)| PlacementDoc {
                    index,
                    x: p.origin.x,
                    y: p.origin.y,
                    width: p.rect.width(),
                    height: p.rect.height(),
                    rotated: p.rotated,
                })
                .collect();
            items.sort_by_key(|p| p.index);
            BinDoc { items }
        })
        .collect();

    SolutionDoc {
        bin_width: bin_w,
        bin_height: bin_h,
        num_items,
        bins_used: sol.bins_used(),
        unplaced: sol.unplaced.clone(),
        runtime_seconds,
        bins,
    }
}

/// Write `doc` to `dir/<instance>.solution` as pretty JSON, creating `dir` if needed.
pub fn save(doc: &SolutionDoc, dir: &Path, instance: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{instance}.solution"));
    let json = serde_json::to_string_pretty(doc).expect("solution serializes");
    std::fs::write(path, json)
}
