use crate::grid::Grid;
use crate::path::{Node, PathResult};
use std::collections::HashSet;

/// Affiche la grille hex avec couleurs simples.
/// Si min/max fournis, surligne leurs cases.
pub fn print_grid(
    grid: &Grid,
    min_path: Option<&PathResult>,
    max_path: Option<&PathResult>,
) {
    let mut min_set: HashSet<(usize, usize)> = HashSet::new();
    let mut max_set: HashSet<(usize, usize)> = HashSet::new();

    if let Some(p) = min_path {
        for n in &p.nodes {
            min_set.insert((n.x, n.y));
        }
    }
    if let Some(p) = max_path {
        for n in &p.nodes {
            max_set.insert((n.x, n.y));
        }
    }

    for y in 0..grid.height {
        for x in 0..grid.width {
            let v = grid.get(x, y);
            let coord = (x, y);
            let is_start = x == 0 && y == 0;
            let is_end = x == grid.width - 1 && y == grid.height - 1;

            let in_min = min_set.contains(&coord);
            let in_max = max_set.contains(&coord);

            // Déterminer couleurs
            if is_start || is_end {
                // start / end → blanc vif
                print!("\x1b[1;97m{:02X}\x1b[0m ", v);
            } else if in_min && in_max {
                // dans les deux chemins → magenta
                print!("\x1b[1;95m{:02X}\x1b[0m ", v);
            } else if in_min {
                // chemin min → blanc
                print!("\x1b[1;97m{:02X}\x1b[0m ", v);
            } else if in_max {
                // chemin max → rouge
                print!("\x1b[1;91m{:02X}\x1b[0m ", v);
            } else {
                // coloration simple : 6 couleurs selon la valeur
                let idx = (v as usize * 6) / 256; // 0..5
                let color_code = 31 + idx as u8;  // 31..36
                print!("\x1b[1;{}m{:02X}\x1b[0m ", color_code, v);
            }
        }
        println!();
    }
}

/// Affiche un rapport détaillé comme dans l’énoncé
pub fn print_path_report(title: &str, grid: &Grid, res: &PathResult) {
    println!("{title}:");
    println!("{}", "=".repeat(title.len()));
    println!(
        "Total cost: 0x{:X} ({} decimal)",
        res.total_cost, res.total_cost
    );
    println!("Path length: {} steps", res.nodes.len());

    // Affichage du chemin complet
    print!("Path: ");
    for (i, node) in res.nodes.iter().enumerate() {
        if i > 0 {
            print!("→");
        }
        print!("({},{})", node.x, node.y);
    }
    println!();
    println!();

    print_step_by_step(grid, &res.nodes);
}

/// Affiche étape par étape les coûts
fn print_step_by_step(grid: &Grid, nodes: &[Node]) {
    println!("Step-by-step costs:");
    if nodes.is_empty() {
        return;
    }

    let start = nodes[0];
    let start_val = grid.get(start.x, start.y);
    println!(
        "  Start  0x{:02X} ({},{})",
        start_val, start.x, start.y
    );

    let mut total: u32 = 0;

    for window in nodes.windows(2) {
        let to = window[1];
        let val = grid.get(to.x, to.y) as u32;
        total += val;
        println!(
            "    →    0x{:02X} ({},{})  +{}",
            val,
            to.x,
            to.y,
            val
        );
    }

    println!("  Total: 0x{:X} ({})", total, total);
}

/// Simple placeholder pour --animate
pub fn animate_placeholder() {
    println!("Animation mode is not implemented yet.");
    println!("(Add Dijkstra step-by-step visualization here.)");
}

