//! Pathfinding A* rápido y acotado sobre la grilla dinámica de la colonia.
//!
//! # Lógica del módulo
//! Calcula rutas libres de obstáculos (árboles, rocas y paredes construidas de
//! madera) evitando cortar esquinas diagonales. Limita las iteraciones máximas
//! para garantizar rendimiento fluido en dispositivos móviles de 32 y 64 bits.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Eq, PartialEq)]
struct OpenNode {
    f_cost: i32,
    g_cost: i32,
    x: i32,
    y: i32,
}

impl Ord for OpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for OpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[inline]
fn heuristic(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    let dx = (ax - bx).abs();
    let dy = (ay - by).abs();
    let min_d = dx.min(dy);
    let max_d = dx.max(dy);
    min_d * 14 + (max_d - min_d) * 10
}

#[inline]
fn is_walkable(w: i32, h: i32, blocked: &[i32], x: i32, y: i32) -> bool {
    if x < 1 || y < 1 || x >= w - 1 || y >= h - 1 {
        return false;
    }
    let idx = (y * w + x) as usize;
    blocked.get(idx).copied().unwrap_or(1) == 0
}

/// Encuentra un camino desde `(sx, sy)` hasta `(gx, gy)` o una casilla adyacente si
/// `allow_adjacent` es verdadero (ideal para talar, minar o construir una pared).
pub fn find_path(
    w: i32,
    h: i32,
    blocked: &[i32],
    sx: i32,
    sy: i32,
    gx: i32,
    gy: i32,
    allow_adjacent: bool,
) -> Option<Vec<(i32, i32)>> {
    if w <= 2 || h <= 2 {
        return None;
    }
    let start_x = sx.clamp(1, w - 2);
    let start_y = sy.clamp(1, h - 2);
    let goal_x = gx.clamp(1, w - 2);
    let goal_y = gy.clamp(1, h - 2);

    if allow_adjacent {
        let dx = (start_x - goal_x).abs();
        let dy = (start_y - goal_y).abs();
        if dx <= 1 && dy <= 1 && (dx + dy) > 0 {
            return Some(Vec::new());
        }
    } else if start_x == goal_x && start_y == goal_y {
        return Some(Vec::new());
    }

    if !allow_adjacent && !is_walkable(w, h, blocked, goal_x, goal_y) {
        return None;
    }

    let total = (w * h) as usize;
    let mut g_scores = vec![i32::MAX; total];
    let mut came_from = vec![-1i32; total];
    let mut closed = vec![false; total];
    let mut heap = BinaryHeap::new();

    let start_idx = (start_y * w + start_x) as usize;
    g_scores[start_idx] = 0;
    heap.push(OpenNode {
        f_cost: heuristic(start_x, start_y, goal_x, goal_y),
        g_cost: 0,
        x: start_x,
        y: start_y,
    });

    const DIRS: [(i32, i32, i32); 8] = [
        (1, 0, 10),
        (-1, 0, 10),
        (0, 1, 10),
        (0, -1, 10),
        (1, 1, 14),
        (1, -1, 14),
        (-1, 1, 14),
        (-1, -1, 14),
    ];

    let mut expansions = 0i32;
    let max_expansions = 1800;
    let mut reached_idx: Option<usize> = None;

    while let Some(curr) = heap.pop() {
        let c_idx = (curr.y * w + curr.x) as usize;
        if closed[c_idx] {
            continue;
        }
        closed[c_idx] = true;

        if allow_adjacent {
            let dx = (curr.x - goal_x).abs();
            let dy = (curr.y - goal_y).abs();
            if dx <= 1 && dy <= 1 && (dx + dy) > 0 {
                reached_idx = Some(c_idx);
                break;
            }
        } else if curr.x == goal_x && curr.y == goal_y {
            reached_idx = Some(c_idx);
            break;
        }

        expansions += 1;
        if expansions >= max_expansions {
            break;
        }

        for (dx, dy, step_cost) in DIRS {
            let nx = curr.x + dx;
            let ny = curr.y + dy;
            if !is_walkable(w, h, blocked, nx, ny) {
                continue;
            }
            // Evitar cortar esquinas diagonales si un lado ortogonal está bloqueado.
            if dx != 0
                && dy != 0
                && (!is_walkable(w, h, blocked, curr.x + dx, curr.y)
                    || !is_walkable(w, h, blocked, curr.x, curr.y + dy))
            {
                continue;
            }

            let n_idx = (ny * w + nx) as usize;
            if closed[n_idx] {
                continue;
            }
            let tentative_g = curr.g_cost + step_cost;
            if tentative_g < g_scores[n_idx] {
                g_scores[n_idx] = tentative_g;
                came_from[n_idx] = c_idx as i32;
                heap.push(OpenNode {
                    f_cost: tentative_g + heuristic(nx, ny, goal_x, goal_y),
                    g_cost: tentative_g,
                    x: nx,
                    y: ny,
                });
            }
        }
    }

    let mut end_idx = reached_idx?;
    let mut path = Vec::new();
    while end_idx != start_idx {
        let px = (end_idx as i32) % w;
        let py = (end_idx as i32) / w;
        path.push((px, py));
        let prev = came_from[end_idx];
        if prev < 0 {
            break;
        }
        end_idx = prev as usize;
    }
    path.reverse();
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_path_around_wooden_wall() {
        let mut blocked = vec![0i32; 16 * 16];
        // Muro vertical bloqueando el paso directo en x = 8, y = 4..=10
        for y in 4..=10 {
            blocked[y * 16 + 8] = 1;
        }
        let path = find_path(16, 16, &blocked, 5, 7, 11, 7, false);
        assert!(path.is_some());
        let steps = path.unwrap();
        assert!(!steps.is_empty());
        for (px, py) in steps {
            assert_eq!(blocked[(py * 16 + px) as usize], 0);
        }
    }

    #[test]
    fn finds_adjacent_tile_to_blocked_tree() {
        let mut blocked = vec![0i32; 16 * 16];
        blocked[8 * 16 + 8] = 1;
        let path = find_path(16, 16, &blocked, 4, 8, 8, 8, true);
        assert!(path.is_some());
        let steps = path.unwrap();
        let last = steps.last().copied().unwrap();
        assert!((last.0 - 8).abs() <= 1 && (last.1 - 8).abs() <= 1);
        assert_ne!(last, (8, 8));
    }
}
