use crate::grid::Grid;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Node {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug)]
pub struct PathResult {
    pub total_cost: u32, // coût réel (somme des valeurs des cases, sans compter la case start)
    pub nodes: Vec<Node>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: u32,
    x: usize,
    y: usize,
}

// Pour BinaryHeap (min-heap via inversion)
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // on inverse pour avoir un min-heap
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.y.cmp(&other.y))
            .then_with(|| self.x.cmp(&other.x))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Dijkstra classique pour coût MINIMUM (sur coûts = valeur de la cellule)
pub fn find_min_path(grid: &Grid) -> Option<PathResult> {
    dijkstra(grid, false)
}

/// Variante MAXIMUM: on utilise des poids transformés (255 - valeur)
/// mais on recalcule la vraie somme après.
pub fn find_max_path(grid: &Grid) -> Option<PathResult> {
    dijkstra(grid, true)
}

fn dijkstra(grid: &Grid, maximize: bool) -> Option<PathResult> {
    let w = grid.width;
    let h = grid.height;
    let n = w * h;

    let start = Node { x: 0, y: 0 };
    let goal = Node {
        x: w - 1,
        y: h - 1,
    };

    let mut dist = vec![u32::MAX; n];
    let mut prev: Vec<Option<Node>> = vec![None; n];

    let start_idx = idx(start.x, start.y, w);
    dist[start_idx] = 0;

    let mut heap = BinaryHeap::new();
    heap.push(State {
        cost: 0,
        x: start.x,
        y: start.y,
    });

    while let Some(State { cost, x, y }) = heap.pop() {
        let idx_cur = idx(x, y, w);

        if cost > dist[idx_cur] {
            continue;
        }
        if x == goal.x && y == goal.y {
            break;
        }

        for (nx, ny) in grid.neighbors(x, y) {
            let idx_next = idx(nx, ny, w);
            let cell_value = grid.get(nx, ny) as u32;

            let weight = if maximize {
                // poids transformé pour "favoriser" les grandes valeurs
                255u32.saturating_sub(cell_value)
            } else {
                cell_value
            };

            let next_cost = cost.saturating_add(weight);

            if next_cost < dist[idx_next] {
                dist[idx_next] = next_cost;
                prev[idx_next] = Some(Node { x, y });
                heap.push(State {
                    cost: next_cost,
                    x: nx,
                    y: ny,
                });
            }
        }
    }

    let goal_idx = idx(goal.x, goal.y, w);
    if dist[goal_idx] == u32::MAX {
        return None;
    }

    // reconstruit le chemin
    let mut path_nodes: Vec<Node> = Vec::new();
    let mut cur = Some(goal);
    while let Some(node) = cur {
        path_nodes.push(node);
        if node.x == start.x && node.y == start.y {
            break;
        }
        let i = idx(node.x, node.y, w);
        cur = prev[i];
    }
    path_nodes.reverse();

    // calcule le coût réel (somme des valeurs des cellules, sans compter start)
    let mut real_cost: u32 = 0;
    for (i, node) in path_nodes.iter().enumerate() {
        if i == 0 {
            continue;
        }
        real_cost += grid.get(node.x, node.y) as u32;
    }

    Some(PathResult {
        total_cost: real_cost,
        nodes: path_nodes,
    })
}

fn idx(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}

