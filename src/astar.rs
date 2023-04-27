use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
};

use crate::math::*;

#[derive(PartialEq, Eq)]
struct Node {
    pos: IVec2,
    priority: Reverse<i32>,
}

impl Node {
    fn new(pos: IVec2, priority: i32) -> Self {
        Self {
            pos,
            priority: Reverse(priority),
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn navigate(map: &crate::map::Map, start: Vec2, target: Vec2) -> Vec<Vec2> {
    let i_start = start.as_ivec2();
    let i_target = target.as_ivec2();

    let mut frontier = BinaryHeap::new();
    frontier.push(Node::new(i_start, 0));

    let mut came_from = HashMap::new();
    let mut g_cost = HashMap::new();
    g_cost.insert(i_start, 0);

    while !frontier.is_empty() {
        let Some(current) = frontier.pop() else {
          break;
      };

        if current.pos == i_target {
            break;
        }

        for neighbor in neighbor_points(current.pos) {
            if neighbor.x.is_negative()
                || neighbor.y.is_negative()
                || map.get_tile(neighbor.x as u32, neighbor.y as u32)
                    != Some(&crate::map::Tile::Empty)
            {
                continue;
            }

            let cost_to_neighbor = g_cost[&current.pos] + heuristic(&current.pos, &neighbor);

            // Initialize neighbor
            if !g_cost.contains_key(&neighbor) || cost_to_neighbor < g_cost[&neighbor] {
                g_cost.insert(neighbor, cost_to_neighbor);

                let priority = cost_to_neighbor + heuristic(&i_target, &neighbor);
                came_from.insert(neighbor, current.pos);
                frontier.push(Node::new(neighbor, priority));
            }
        }
    }
    let mut path = Vec::new();

    let mut current = target;

    while current != start {
        let par = came_from.get(&current.as_ivec2());
        path.push(current);

        if let Some(par) = par {
            current = par.as_vec2();
        } else {
            break;
        }
    }
    path.reverse();
    path
}

fn neighbor_points(point: IVec2) -> Vec<IVec2> {
    vec![
        ivec2(-1, 0) + point,
        ivec2(1, 0) + point,
        ivec2(0, -1) + point,
        ivec2(0, 1) + point,
    ]
}

fn heuristic(a: &IVec2, b: &IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
