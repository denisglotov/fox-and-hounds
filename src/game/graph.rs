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
    pub dist_matrix: Vec<Option<usize>>,
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

        // Precompute All-Pairs Shortest Path (APSP) matrix with BFS
        let n = nodes.len();
        let mut dist_matrix = vec![None; n * n];
        for start in 0..n {
            dist_matrix[start * n + start] = Some(0);
            let mut queue = VecDeque::new();
            queue.push_back(start);
            while let Some(current) = queue.pop_front() {
                let curr_dist = dist_matrix[start * n + current].unwrap();
                for &next in &adjacency[current] {
                    let idx = start * n + next;
                    if dist_matrix[idx].is_none() {
                        dist_matrix[idx] = Some(curr_dist + 1);
                        queue.push_back(next);
                    }
                }
            }
        }

        Self {
            nodes,
            name_to_id,
            adjacency,
            dist_matrix,
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

    /// Returns precomputed shortest distance between two nodes in O(1) time without allocations.
    pub fn distance(&self, start: usize, target: usize) -> Option<usize> {
        let n = self.nodes.len();
        if start < n && target < n {
            self.dist_matrix[start * n + target]
        } else {
            None
        }
    }

    /// Computes BFS shortest distance from `start` to `target`, avoiding any `obstacles`.
    /// Fast-paths to O(1) precomputed lookup when obstacles slice is empty.
    pub fn shortest_distance(
        &self,
        start: usize,
        target: usize,
        obstacles: &[usize],
    ) -> Option<usize> {
        if obstacles.is_empty() {
            return self.distance(start, target);
        }

        if start == target {
            return Some(0);
        }

        let n = self.nodes.len();
        if start >= n || target >= n {
            return None;
        }

        // Fast zero-allocation path for typical game graphs (<= 32 nodes)
        if n <= 32 {
            let mut dist = [u8::MAX; 32];
            let mut queue = [0usize; 32];
            let mut head = 0;
            let mut tail = 0;

            dist[start] = 0;
            queue[tail] = start;
            tail += 1;

            while head < tail {
                let current = queue[head];
                head += 1;
                let curr_dist = dist[current];

                for &next in self.neighbors(current) {
                    if next == target {
                        return Some((curr_dist + 1) as usize);
                    }
                    if next < n && dist[next] == u8::MAX && !obstacles.contains(&next) {
                        dist[next] = curr_dist + 1;
                        if tail < 32 {
                            queue[tail] = next;
                            tail += 1;
                        }
                    }
                }
            }

            if dist[target] != u8::MAX {
                Some(dist[target] as usize)
            } else {
                None
            }
        } else {
            let mut distances = vec![None; n];
            let mut queue = VecDeque::new();

            distances[start] = Some(0);
            queue.push_back(start);

            while let Some(current) = queue.pop_front() {
                let curr_dist = distances[current].unwrap();
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

    /// Generates an iterator over legal destination nodes for the Fox.
    pub fn fox_legal_moves<'a>(
        &'a self,
        fox_pos: usize,
        hounds_pos: &'a [usize],
    ) -> impl Iterator<Item = usize> + 'a {
        self.neighbors(fox_pos)
            .iter()
            .copied()
            .filter(move |&target| !hounds_pos.contains(&target))
    }

    /// Generates an iterator over every vertex the Fox may enter the board on for its
    /// opening move: any vertex that is not occupied by a Hound and is not the Chicken Coop.
    pub fn fox_entry_moves<'a>(
        &'a self,
        hounds_pos: &'a [usize],
        coop_pos: usize,
    ) -> impl Iterator<Item = usize> + 'a {
        self.nodes
            .iter()
            .map(|node| node.id)
            .filter(move |&target| {
                !hounds_pos.contains(&target)
                    && target != coop_pos
                    && self
                        .node(target)
                        .is_none_or(|n| n.node_type != NodeType::TargetCoop)
            })
    }

    /// Generates an iterator over legal destination nodes for a Hound at `hound_pos`.
    pub fn hound_legal_moves<'a>(
        &'a self,
        hound_pos: usize,
        fox_pos: usize,
        coop_pos: usize,
        hounds_pos: &'a [usize],
        allow_retreat: bool,
    ) -> impl Iterator<Item = usize> + 'a {
        let current_row = self.node(hound_pos).map(|n| n.row);
        self.neighbors(hound_pos)
            .iter()
            .copied()
            .filter(move |&target| {
                if target == fox_pos || target == coop_pos || hounds_pos.contains(&target) {
                    return false;
                }
                let target_node = self.node(target);
                if target_node.is_none_or(|n| n.node_type == NodeType::TargetCoop) {
                    return false;
                }
                if !allow_retreat {
                    if let (Some(cur), Some(tgt)) = (current_row, target_node.map(|n| n.row)) {
                        if tgt < cur {
                            return false;
                        }
                    }
                }
                true
            })
    }
}
