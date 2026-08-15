use super::graph::{Graph, Node, NodeType};
use macroquad::prelude::Vec2;

pub const BOARD_IMAGE_WIDTH: f32 = 848.0;
pub const BOARD_IMAGE_HEIGHT: f32 = 1264.0;

pub struct LevelConfig {
    pub name: &'static str,
    pub description: &'static str,
    pub fox_start_node: &'static str,
    pub hounds_start_nodes: &'static [&'static str],
    pub target_coop_node: &'static str,
}

pub const RIVER_CROSSING_CONFIG: LevelConfig = LevelConfig {
    name: "The River Crossing",
    description: "3x10 board with a river bottleneck on Row 7 and diamond connectivity",
    fox_start_node: "M10",
    hounds_start_nodes: &["L1", "M1", "R1"],
    target_coop_node: "M0",
};

pub fn build_river_crossing_graph() -> Graph {
    let col_x = [190.0, 424.0, 658.0];
    let row_y = [
        95.0,   // Row 0
        198.0,  // Row 1
        302.0,  // Row 2
        406.0,  // Row 3
        510.0,  // Row 4
        614.0,  // Row 5
        718.0,  // Row 6
        822.0,  // Row 7
        926.0,  // Row 8
        1030.0, // Row 9
        1134.0, // Row 10
    ];

    let raw_nodes = vec![
        (
            "M0",
            0,
            1,
            NodeType::TargetCoop,
            Vec2::new(col_x[1], row_y[0]),
        ),
        (
            "L1",
            1,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[1]),
        ),
        (
            "M1",
            1,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[1]),
        ),
        (
            "R1",
            1,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[1]),
        ),
        (
            "L2",
            2,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[2]),
        ),
        (
            "M2",
            2,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[2]),
        ),
        (
            "R2",
            2,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[2]),
        ),
        (
            "L3",
            3,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[3]),
        ),
        (
            "M3",
            3,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[3]),
        ),
        (
            "R3",
            3,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[3]),
        ),
        (
            "L4",
            4,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[4]),
        ),
        (
            "M4",
            4,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[4]),
        ),
        (
            "R4",
            4,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[4]),
        ),
        (
            "L5",
            5,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[5]),
        ),
        (
            "M5",
            5,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[5]),
        ),
        (
            "R5",
            5,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[5]),
        ),
        (
            "L6",
            6,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[6]),
        ),
        (
            "M6",
            6,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[6]),
        ),
        (
            "R6",
            6,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[6]),
        ),
        (
            "M7",
            7,
            1,
            NodeType::Bottleneck,
            Vec2::new(col_x[1], row_y[7]),
        ),
        (
            "L8",
            8,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[8]),
        ),
        (
            "M8",
            8,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[8]),
        ),
        (
            "R8",
            8,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[8]),
        ),
        (
            "L9",
            9,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[9]),
        ),
        (
            "M9",
            9,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[9]),
        ),
        (
            "R9",
            9,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[9]),
        ),
        (
            "M10",
            10,
            1,
            NodeType::FoxStart,
            Vec2::new(col_x[1], row_y[10]),
        ),
    ];

    let nodes: Vec<Node> = raw_nodes
        .into_iter()
        .enumerate()
        .map(|(id, (name, row, col, node_type, visual_pos))| Node {
            id,
            name: name.to_string(),
            row,
            col,
            node_type,
            visual_pos,
        })
        .collect();

    let raw_edges = vec![
        ("M0", "L1"),
        ("M0", "M1"),
        ("M0", "R1"),
        ("L1", "M1"),
        ("M1", "R1"),
        ("L1", "L2"),
        ("M1", "M2"),
        ("R1", "R2"),
        ("L1", "M2"),
        ("R1", "M2"),
        ("M1", "L2"),
        ("M1", "R2"),
        ("L2", "M2"),
        ("M2", "R2"),
        ("L2", "L3"),
        ("M2", "M3"),
        ("R2", "R3"),
        ("L2", "M3"),
        ("R2", "M3"),
        ("M2", "L3"),
        ("M2", "R3"),
        ("L3", "M3"),
        ("M3", "R3"),
        ("L3", "L4"),
        ("M3", "M4"),
        ("R3", "R4"),
        ("L3", "M4"),
        ("R3", "M4"),
        ("M3", "L4"),
        ("M3", "R4"),
        ("L4", "M4"),
        ("M4", "R4"),
        ("L4", "L5"),
        ("M4", "M5"),
        ("R4", "R5"),
        ("L4", "M5"),
        ("R4", "M5"),
        ("M4", "L5"),
        ("M4", "R5"),
        ("L5", "M5"),
        ("M5", "R5"),
        ("L5", "L6"),
        ("M5", "M6"),
        ("R5", "R6"),
        ("L5", "M6"),
        ("R5", "M6"),
        ("M5", "L6"),
        ("M5", "R6"),
        ("L6", "M6"),
        ("M6", "R6"),
        ("L6", "M7"),
        ("M6", "M7"),
        ("R6", "M7"),
        ("M7", "L8"),
        ("M7", "M8"),
        ("M7", "R8"),
        ("L8", "M8"),
        ("M8", "R8"),
        ("L8", "L9"),
        ("M8", "M9"),
        ("R8", "R9"),
        ("L8", "M9"),
        ("R8", "M9"),
        ("M8", "L9"),
        ("M8", "R9"),
        ("L9", "M9"),
        ("M9", "R9"),
        ("L9", "M10"),
        ("M9", "M10"),
        ("R9", "M10"),
    ];

    let name_to_id: std::collections::HashMap<&str, usize> = nodes
        .iter()
        .map(|node| (node.name.as_str(), node.id))
        .collect();

    let edges: Vec<(usize, usize)> = raw_edges
        .into_iter()
        .filter_map(|(u_name, v_name)| {
            let u = name_to_id.get(u_name)?;
            let v = name_to_id.get(v_name)?;
            Some((*u, *v))
        })
        .collect();

    Graph::new(nodes, &edges)
}
