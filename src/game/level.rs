use super::graph::{Graph, Node, NodeType};
use macroquad::prelude::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoardDimensions {
    pub image_width: f32,
    pub image_height: f32,
    pub left_width: f32,
    pub right_width: f32,
}

impl BoardDimensions {
    pub const fn total_width(&self) -> f32 {
        self.left_width + self.image_width + self.right_width
    }

    pub const fn composition_center_x(&self) -> f32 {
        (self.image_width + self.right_width - self.left_width) / 2.0
    }
}

pub const RIVER_CROSSING_DIMENSIONS: BoardDimensions = BoardDimensions {
    image_width: 768.0,
    image_height: 1376.0,
    left_width: 384.0,
    right_width: 256.0,
};

pub const CLASSIC_DIMENSIONS: BoardDimensions = BoardDimensions {
    image_width: 1024.0,
    image_height: 1024.0,
    left_width: 0.0,
    right_width: 0.0,
};

pub const BOARD_IMAGE_WIDTH: f32 = RIVER_CROSSING_DIMENSIONS.image_width;
pub const BOARD_IMAGE_HEIGHT: f32 = RIVER_CROSSING_DIMENSIONS.image_height;
pub const BOARD_LEFT_WIDTH: f32 = RIVER_CROSSING_DIMENSIONS.left_width;
pub const BOARD_RIGHT_WIDTH: f32 = RIVER_CROSSING_DIMENSIONS.right_width;
pub const BOARD_TOTAL_WIDTH: f32 = RIVER_CROSSING_DIMENSIONS.total_width();
pub const BOARD_COMPOSITION_CENTER_X: f32 = RIVER_CROSSING_DIMENSIONS.composition_center_x();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BoardVariant {
    #[default]
    Classic,
    RiverCrossing,
}

#[derive(Debug, Clone, Copy)]
pub struct VariantConfig {
    pub id: BoardVariant,
    pub name: &'static str,
    pub description: &'static str,
    pub allow_hound_retreat: bool,
    pub dimensions: BoardDimensions,
    pub board_image_bytes: &'static [u8],
    pub fox_start_node: &'static str,
    pub hounds_start_nodes: &'static [&'static str],
    pub target_coop_node: &'static str,
    pub build_graph: fn() -> Graph,
}

pub type LevelConfig = VariantConfig;

pub const CLASSIC_CONFIG: VariantConfig = VariantConfig {
    id: BoardVariant::Classic,
    name: "Classic",
    description: "Traditional rules on an 11-node spearhead board where hounds cannot retreat",
    allow_hound_retreat: false,
    dimensions: CLASSIC_DIMENSIONS,
    board_image_bytes: include_bytes!("../../assets/classic_board_image.png"),
    fox_start_node: "M4",
    hounds_start_nodes: &["M0", "T1", "B1"],
    target_coop_node: "M0",
    build_graph: build_classic_graph,
};

pub const RIVER_CROSSING_CONFIG: VariantConfig = VariantConfig {
    id: BoardVariant::RiverCrossing,
    name: "The river crossing",
    description: "3x9 board with a river bottleneck on Row 6 and free hound movement",
    allow_hound_retreat: true,
    dimensions: RIVER_CROSSING_DIMENSIONS,
    board_image_bytes: include_bytes!("../../assets/board_image.png"),
    fox_start_node: "M9",
    hounds_start_nodes: &["L1", "M1", "R1"],
    target_coop_node: "M0",
    build_graph: build_river_crossing_graph,
};

impl BoardVariant {
    pub const fn config(self) -> &'static VariantConfig {
        match self {
            BoardVariant::Classic => &CLASSIC_CONFIG,
            BoardVariant::RiverCrossing => &RIVER_CROSSING_CONFIG,
        }
    }

    pub const fn all() -> &'static [BoardVariant] {
        &[BoardVariant::Classic, BoardVariant::RiverCrossing]
    }

    pub fn localized_name(self, locales: &crate::game::i18n::LocaleStrings) -> &str {
        locales.variant_name(self)
    }

    pub fn localized_sub(self, locales: &crate::game::i18n::LocaleStrings) -> &str {
        locales.variant_sub(self)
    }
}

pub fn build_river_crossing_graph() -> Graph {
    let col_x = [230.0, 384.0, 538.0];
    let row_y = [
        150.0,  // Row 0 (Coop)
        226.0,  // Row 1
        328.0,  // Row 2
        438.0,  // Row 3
        554.0,  // Row 4
        660.0,  // Row 5
        755.0,  // Row 6 (Bridge Bottleneck)
        852.0,  // Row 7
        955.0,  // Row 8
        1052.0, // Row 9 (Fox Den)
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
            "M6",
            6,
            1,
            NodeType::Bottleneck,
            Vec2::new(col_x[1], row_y[6]),
        ),
        (
            "L7",
            7,
            0,
            NodeType::Standard,
            Vec2::new(col_x[0], row_y[7]),
        ),
        (
            "M7",
            7,
            1,
            NodeType::Standard,
            Vec2::new(col_x[1], row_y[7]),
        ),
        (
            "R7",
            7,
            2,
            NodeType::Standard,
            Vec2::new(col_x[2], row_y[7]),
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
            "M9",
            9,
            1,
            NodeType::FoxStart,
            Vec2::new(col_x[1], row_y[9]),
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
        ("L5", "M6"),
        ("M5", "M6"),
        ("R5", "M6"),
        ("M6", "L7"),
        ("M6", "M7"),
        ("M6", "R7"),
        ("L7", "M7"),
        ("M7", "R7"),
        ("L7", "L8"),
        ("M7", "M8"),
        ("R7", "R8"),
        ("L7", "M8"),
        ("R7", "M8"),
        ("M7", "L8"),
        ("M7", "R8"),
        ("L8", "M8"),
        ("M8", "R8"),
        ("L8", "M9"),
        ("M8", "M9"),
        ("R8", "M9"),
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

pub fn build_classic_graph() -> Graph {
    let raw_nodes = vec![
        ("M0", 0, 1, NodeType::TargetCoop, Vec2::new(243.0, 511.0)),
        ("T1", 1, 0, NodeType::Standard, Vec2::new(376.0, 329.0)),
        ("M1", 1, 1, NodeType::Standard, Vec2::new(375.0, 510.0)),
        ("B1", 1, 2, NodeType::Standard, Vec2::new(376.0, 689.0)),
        ("T2", 2, 0, NodeType::Standard, Vec2::new(511.0, 331.0)),
        ("M2", 2, 1, NodeType::Standard, Vec2::new(510.0, 510.0)),
        ("B2", 2, 2, NodeType::Standard, Vec2::new(510.0, 689.0)),
        ("T3", 3, 0, NodeType::Standard, Vec2::new(646.0, 331.0)),
        ("M3", 3, 1, NodeType::Standard, Vec2::new(645.0, 510.0)),
        ("B3", 3, 2, NodeType::Standard, Vec2::new(646.0, 690.0)),
        ("M4", 4, 1, NodeType::FoxStart, Vec2::new(778.0, 510.0)),
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
        ("M0", "T1"),
        ("M0", "M1"),
        ("M0", "B1"),
        ("T1", "M1"),
        ("M1", "B1"),
        ("T2", "M2"),
        ("M2", "B2"),
        ("T3", "M3"),
        ("M3", "B3"),
        ("T1", "T2"),
        ("T2", "T3"),
        ("M1", "M2"),
        ("M2", "M3"),
        ("B1", "B2"),
        ("B2", "B3"),
        ("T1", "M2"),
        ("M2", "B3"),
        ("B1", "M2"),
        ("M2", "T3"),
        ("T3", "M4"),
        ("M3", "M4"),
        ("B3", "M4"),
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
