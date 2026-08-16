use macroquad::prelude::Vec2;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    TargetCoop,
    Standard,
    Bottleneck,
    FoxStart,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub row: usize,
    pub col: usize,
    pub node_type: NodeType,
    pub visual_pos: Vec2,
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub name_to_id: HashMap<String, usize>,
    pub adjacency: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: &[(usize, usize)]) -> Self {
        let name_to_id = nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| (node.name.clone(), idx))
            .collect();

        let mut adjacency = vec![Vec::new(); nodes.len()];
        for &(u, v) in edges {
            if u < nodes.len() && v < nodes.len() {
                if !adjacency[u].contains(&v) {
                    adjacency[u].push(v);
                }
                if !adjacency[v].contains(&u) {
                    adjacency[v].push(u);
                }
            }
        }

        Self {
            nodes,
            name_to_id,
            adjacency,
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn node(&self, id: usize) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn find_id_by_name(&self, name: &str) -> Option<usize> {
        self.name_to_id.get(name).copied()
    }

    pub fn neighbors(&self, node_id: usize) -> &[usize] {
        self.adjacency.get(node_id).map_or(&[], |n| n.as_slice())
    }

    /// Computes BFS shortest distance from `start` to `target`, avoiding any `obstacles`.
    pub fn shortest_distance(
        &self,
        start: usize,
        target: usize,
        obstacles: &[usize],
    ) -> Option<usize> {
        if start == target {
            return Some(0);
        }

        let mut distances = vec![None; self.nodes.len()];
        let mut queue = VecDeque::new();

        distances[start] = Some(0);
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            let curr_dist = distances[current].unwrap_or(0);
            if current == target {
                return Some(curr_dist);
            }

            for &next in self.neighbors(current) {
                if next == target {
                    return Some(curr_dist + 1);
                }
                if distances[next].is_none() && !obstacles.contains(&next) {
                    distances[next] = Some(curr_dist + 1);
                    queue.push_back(next);
                }
            }
        }

        distances[target]
    }
}
