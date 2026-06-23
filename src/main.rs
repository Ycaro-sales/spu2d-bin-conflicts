mod coloring;
mod conflict_graph;
mod container;
mod decoder;
mod ems;
mod encoder;
mod greedy;
mod instance;
mod item;
mod serializer;

use std::path::{Path, PathBuf};
use std::time::Instant;

/// Recursively collect every `*.json` file under `dir`.
fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(json_files(&path));
            } else if path.extension().is_some_and(|e| e == "json") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "instances_conflict/OPP/CJCM08".to_string());

    let files = json_files(Path::new(&dir));
    if files.is_empty() {
        eprintln!("no *.json instances found under {dir}");
        std::process::exit(1);
    }
    println!("solving {} instances under {dir}", files.len());

    for path in files {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("instance path has a UTF-8 file stem");
        let inst = instance::instance_from_path(&path);
        let (w, h, n) = (inst.width, inst.height, inst.items.len());

        // Greedy baseline (first-fit-decreasing).
        let t = Instant::now();
        let g = greedy::solve(&inst);
        let g_secs = t.elapsed().as_secs_f64();
        serializer::save(
            &serializer::to_doc(&g, w, h, n, g_secs),
            Path::new("solutions/greedy"),
            stem,
        )
        .expect("write greedy solution");

        // BRKGA metaheuristic.
        let t = Instant::now();
        let b = decoder::solve(&inst);
        let b_secs = t.elapsed().as_secs_f64();
        serializer::save(
            &serializer::to_doc(&b, w, h, n, b_secs),
            Path::new("solutions/brkga"),
            stem,
        )
        .expect("write brkga solution");

        println!(
            "{stem}: greedy {} bins ({:.3}s) | brkga {} bins ({:.3}s)",
            g.bins_used(),
            g_secs,
            b.bins_used(),
            b_secs
        );
    }
}
