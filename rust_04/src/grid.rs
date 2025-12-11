use rand::Rng;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<u8>, // taille = width * height
}

impl Grid {
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.cells[self.index(x, y)]
    }

    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut n = Vec::with_capacity(4);
        if x > 0 {
            n.push((x - 1, y));
        }
        if x + 1 < self.width {
            n.push((x + 1, y));
        }
        if y > 0 {
            n.push((x, y - 1));
        }
        if y + 1 < self.height {
            n.push((x, y + 1));
        }
        n
    }
}

/// Parse "WxH" from --generate
pub fn parse_dims(spec: &str) -> Result<(usize, usize), String> {
    let parts: Vec<&str> = spec.split('x').collect();
    if parts.len() != 2 {
        return Err("expected format WxH".into());
    }
    let w = parts[0]
        .parse::<usize>()
        .map_err(|_| "invalid width".to_string())?;
    let h = parts[1]
        .parse::<usize>()
        .map_err(|_| "invalid height".to_string())?;
    if w == 0 || h == 0 {
        return Err("width and height must be > 0".into());
    }
    Ok((w, h))
}

/// Generate random grid, with (0,0)=00 and (w-1,h-1)=FF
pub fn generate_grid(width: usize, height: usize) -> Grid {
    let mut rng = rand::thread_rng();
    let mut cells = Vec::with_capacity(width * height);

    for _ in 0..(width * height) {
        // valeurs hex aléatoires 0..=255
        let v: u8 = rng.r#gen();
        cells.push(v);
    }

    // Forcer start/end
    cells[0] = 0x00;
    let last_idx = width * height - 1;
    cells[last_idx] = 0xFF;

    Grid {
        width,
        height,
        cells,
    }
}

/// Load map from file
pub fn load_map<P: AsRef<Path>>(path: P) -> Result<Grid, Box<dyn Error>> {
    let content = fs::read_to_string(&path)?;
    let mut cells: Vec<u8> = Vec::new();
    let mut width: Option<usize> = None;
    let mut height: usize = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut row: Vec<u8> = Vec::new();
        for tok in line.split_whitespace() {
            if tok.len() != 2 {
                return Err(format!("Invalid hex value '{tok}'").into());
            }
            let v = u8::from_str_radix(tok, 16)
                .map_err(|_| format!("Invalid hex number '{tok}'"))?;
            row.push(v);
        }

        if row.is_empty() {
            continue;
        }

        let row_len = row.len();
        if let Some(w) = width {
            if w != row_len {
                return Err("Inconsistent row length in map".into());
            }
        } else {
            width = Some(row_len);
        }

        cells.extend(row);
        height += 1;
    }

    let width = width.ok_or("Empty map")?;
    if height == 0 {
        return Err("Empty map".into());
    }

    // Vérifier start/end
    if cells[0] != 0x00 {
        return Err(format!(
            "Start cell must be 00, found {:02X}",
            cells[0]
        )
        .into());
    }
    let last_idx = width * height - 1;
    if cells[last_idx] != 0xFF {
        return Err(format!(
            "End cell must be FF, found {:02X}",
            cells[last_idx]
        )
        .into());
    }

    Ok(Grid {
        width,
        height,
        cells,
    })
}

/// Save map to file
pub fn save_map<P: AsRef<Path>>(path: P, grid: &Grid) -> Result<(), Box<dyn Error>> {
    let mut out = String::new();
    for y in 0..grid.height {
        for x in 0..grid.width {
            if x > 0 {
                out.push(' ');
            }
            out.push_str(&format!("{:02X}", grid.get(x, y)));
        }
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(())
}

