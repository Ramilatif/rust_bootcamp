mod grid;
mod path;
mod visualize;

use clap::Parser;
use std::error::Error;

/// Find min/max cost paths in hexadecimal grid
#[derive(Parser, Debug)]
#[command(
    name = "hexpath",
    about = "Find min/max cost paths in hexadecimal grid"
)]
struct Cli {
    /// Map file (hex values, space separated)
    map: Option<String>,

    /// Generate random map (e.g., 8x4, 10x10)
    #[arg(long)]
    generate: Option<String>,

    /// Save generated map to file
    #[arg(long)]
    output: Option<String>,

    /// Show colored map
    #[arg(long)]
    visualize: bool,

    /// Show both min and max paths
    #[arg(long)]
    both: bool,

    /// Animate pathfinding (simple placeholder)
    #[arg(long)]
    animate: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Cas 1: génération d'une map
    if let Some(spec) = cli.generate.as_ref() {
        let (w, h) =
            grid::parse_dims(spec).map_err(|e| format!("Invalid --generate spec '{spec}': {e}"))?;

        println!("Generating {}x{} hexadecimal grid...", w, h);
        let grid = grid::generate_grid(w, h);

        if let Some(out) = cli.output.as_ref() {
            grid::save_map(out, &grid)?;
            println!("Map saved to: {out}");
            println!();
            println!("Generated map:");
        } else {
            println!("Generated map:");
        }

        visualize::print_grid(&grid, None, None);

        if cli.visualize || cli.both || cli.animate {
            println!("Finding optimal paths.");

            let min = path::find_min_path(&grid).ok_or("No path found for minimum cost")?;
            let max = path::find_max_path(&grid).ok_or("No path found for maximum cost")?;

            println!();
            visualize::print_path_report("MINIMUM COST PATH", &grid, &min);
            println!();
            visualize::print_path_report("MAXIMUM COST PATH", &grid, &max);

            if cli.visualize {
                println!();
                println!("Visualizing grid with paths:");
                visualize::print_grid(&grid, Some(&min), Some(&max));
            }

            if cli.animate {
                println!();
                visualize::animate_placeholder();
            }
        }

        return Ok(());
    }

    // Cas 2: analyse d'une map existante
    let map_path = cli
        .map
        .as_ref()
        .ok_or("You must provide a map file or use --generate")?;

    let grid = grid::load_map(map_path)?;

    println!("Analyzing hexadecimal grid...");
    println!("Grid size: {}×{}", grid.width, grid.height);
    println!("Start: (0,0) = 0x{:02X}", grid.get(0, 0));
    println!(
        "End: ({}, {}) = 0x{:02X}",
        grid.width - 1,
        grid.height - 1,
        grid.get(grid.width - 1, grid.height - 1)
    );
    println!();

    let min = path::find_min_path(&grid).ok_or("No path found for minimum cost")?;

    visualize::print_path_report("MINIMUM COST PATH", &grid, &min);

    let mut max_opt = None;
    if cli.both {
        println!();
        let max = path::find_max_path(&grid).ok_or("No path found for maximum cost")?;
        visualize::print_path_report("MAXIMUM COST PATH", &grid, &max);
        max_opt = Some(max);
    }

    if cli.visualize {
        println!();
        println!("HEXADECIMAL GRID (colored):");
        visualize::print_grid(&grid, Some(&min), max_opt.as_ref());
    }

    if cli.animate {
        println!();
        visualize::animate_placeholder();
    }

    Ok(())
}
