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

pub const FOX_AND_DOGS_DIMENSIONS: BoardDimensions = BoardDimensions {
    image_width: 1024.0,
    image_height: 1024.0,
    left_width: 0.0,
    right_width: 0.0,
};

pub const ARTHUR_DIMENSIONS: BoardDimensions = FOX_AND_DOGS_DIMENSIONS;
pub const RED_HUNT_DIMENSIONS: BoardDimensions = BoardDimensions {
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
    FoxAndDogs,
    TheRedHunt,
}

pub use BoardVariant::FoxAndDogs as FoxAndDogsSymmetric;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoardIntroFraming {
    pub playable_center: Vec2,
    pub playable_size: Vec2,
    pub max_target_zoom: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct VariantConfig {
    pub id: BoardVariant,
    pub name: &'static str,
    pub description: &'static str,
    pub allow_hound_retreat: bool,
    pub hounds_start_first: bool,
    pub allow_hounds_in_coop: bool,
    pub dimensions: BoardDimensions,
    pub intro_framing: BoardIntroFraming,
    pub board_image_bytes: &'static [u8],
    pub fox_start_node: &'static str,
    pub fox_free_entry: bool,
    pub hounds_start_nodes: &'static [&'static str],
    pub target_coop_node: &'static str,
    pub move_duration: f32,
    pub piece_base_size: f32,
    pub build_graph: fn() -> Graph,
}

pub type LevelConfig = VariantConfig;

pub const DEFAULT_MOVE_DURATION: f32 = 0.26;
pub const RED_HUNT_MOVE_SLOWNESS: f32 = 1.5;
pub const RED_HUNT_MOVE_DURATION: f32 = DEFAULT_MOVE_DURATION * RED_HUNT_MOVE_SLOWNESS;
pub const DEFAULT_PIECE_BASE_SIZE: f32 = 76.0;
pub const RED_HUNT_PIECE_BASE_SIZE: f32 = 58.0;

pub const CLASSIC_CONFIG: VariantConfig = VariantConfig {
    id: BoardVariant::Classic,
    name: "Classic",
    description: "Traditional rules on an 11-node spearhead board where hounds cannot retreat",
    allow_hound_retreat: false,
    hounds_start_first: false,
    allow_hounds_in_coop: false,
    dimensions: CLASSIC_DIMENSIONS,
    intro_framing: BoardIntroFraming {
        playable_center: Vec2::new(511.0, 510.0),
        playable_size: Vec2::new(660.0, 520.0),
        max_target_zoom: 1.30,
    },
    board_image_bytes: include_bytes!("../../assets/classic_board_image.png"),
    fox_start_node: "M4",
    fox_free_entry: true,
    hounds_start_nodes: &["M0", "T1", "B1"],
    target_coop_node: "M0",
    move_duration: DEFAULT_MOVE_DURATION,
    piece_base_size: DEFAULT_PIECE_BASE_SIZE,
    build_graph: build_classic_graph,
};

pub const RIVER_CROSSING_CONFIG: VariantConfig = VariantConfig {
    id: BoardVariant::RiverCrossing,
    name: "The river crossing",
    description: "3x9 board with a river bottleneck on Row 6 and free hound movement",
    allow_hound_retreat: true,
    hounds_start_first: false,
    allow_hounds_in_coop: false,
    dimensions: RIVER_CROSSING_DIMENSIONS,
    intro_framing: BoardIntroFraming {
        playable_center: Vec2::new(384.0, 600.0),
        playable_size: Vec2::new(380.0, 1080.0),
        max_target_zoom: 1.85,
    },
    board_image_bytes: include_bytes!("../../assets/board_image.png"),
    fox_start_node: "M9",
    fox_free_entry: false,
    hounds_start_nodes: &["L1", "M1", "R1"],
    target_coop_node: "M0",
    move_duration: DEFAULT_MOVE_DURATION,
    piece_base_size: DEFAULT_PIECE_BASE_SIZE,
    build_graph: build_river_crossing_graph,
};

pub const FOX_AND_DOGS_CONFIG: VariantConfig = VariantConfig {
    id: BoardVariant::FoxAndDogs,
    name: "fox and dogs",
    description: "Fox and dogs board: dogs start on Row 7 and move first, fox must return to C8",
    allow_hound_retreat: true,
    hounds_start_first: true,
    allow_hounds_in_coop: true,
    dimensions: FOX_AND_DOGS_DIMENSIONS,
    intro_framing: BoardIntroFraming {
        playable_center: Vec2::new(510.0, 517.0),
        playable_size: Vec2::new(420.0, 780.0),
        max_target_zoom: 1.35,
    },
    board_image_bytes: include_bytes!("../../assets/fox_and_dogs_board.png"),
    fox_start_node: "C8",
    fox_free_entry: false,
    hounds_start_nodes: &["R7", "C7", "L7"],
    target_coop_node: "C8",
    move_duration: DEFAULT_MOVE_DURATION,
    piece_base_size: DEFAULT_PIECE_BASE_SIZE,
    build_graph: build_fox_and_dogs_graph,
};

pub const FOX_AND_DOGS_SYMMETRIC_CONFIG: VariantConfig = FOX_AND_DOGS_CONFIG;

pub const THE_RED_HUNT_CONFIG: VariantConfig = VariantConfig {
    id: BoardVariant::TheRedHunt,
    name: "The Red Hunt",
    description: "Martian crustal fault board: fox starts at C4 and races to C0, hounds start at R2, C1, L2 and can retreat",
    allow_hound_retreat: true,
    hounds_start_first: false,
    allow_hounds_in_coop: false,
    dimensions: RED_HUNT_DIMENSIONS,
    intro_framing: BoardIntroFraming {
        playable_center: Vec2::new(512.0, 514.0),
        playable_size: Vec2::new(400.0, 720.0),
        max_target_zoom: 1.40,
    },
    board_image_bytes: include_bytes!("../../assets/the_red_hunt_board.png"),
    fox_start_node: "C4",
    fox_free_entry: false,
    hounds_start_nodes: &["R2", "C1", "L2"],
    target_coop_node: "C0",
    move_duration: RED_HUNT_MOVE_DURATION,
    piece_base_size: RED_HUNT_PIECE_BASE_SIZE,
    build_graph: build_the_red_hunt_graph,
};

impl BoardVariant {
    pub const fn move_duration(self) -> f32 {
        self.config().move_duration
    }
    pub const fn piece_base_size(self) -> f32 {
        self.config().piece_base_size
    }
    pub const fn config(self) -> &'static VariantConfig {
        match self {
            BoardVariant::Classic => &CLASSIC_CONFIG,
            BoardVariant::RiverCrossing => &RIVER_CROSSING_CONFIG,
            BoardVariant::FoxAndDogs => &FOX_AND_DOGS_CONFIG,
            BoardVariant::TheRedHunt => &THE_RED_HUNT_CONFIG,
        }
    }

    pub const fn all() -> &'static [BoardVariant] {
        &[
            BoardVariant::Classic,
            BoardVariant::RiverCrossing,
            BoardVariant::FoxAndDogs,
            BoardVariant::TheRedHunt,
        ]
    }

    pub const fn is_arthur(self) -> bool {
        matches!(self, BoardVariant::FoxAndDogs)
    }

    pub const fn is_red_hunt(self) -> bool {
        matches!(self, BoardVariant::TheRedHunt)
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

fn build_arthur_nodes() -> Vec<Node> {
    let raw_nodes = vec![
        ("C0", 0, 1, NodeType::Standard, Vec2::new(510., 184.4)),
        ("L1", 1, 0, NodeType::Standard, Vec2::new(389.3, 253.5)),
        ("C1", 1, 1, NodeType::Standard, Vec2::new(510., 254.5)),
        ("R1", 1, 2, NodeType::Standard, Vec2::new(630.1, 254.7)),
        ("L2", 2, 0, NodeType::Standard, Vec2::new(366.5, 328.5)),
        ("C2", 2, 1, NodeType::Bottleneck, Vec2::new(510., 323.8)),
        ("R2", 2, 2, NodeType::Standard, Vec2::new(649.2, 323.1)),
        ("C3", 3, 1, NodeType::Bottleneck, Vec2::new(511., 494.)),
        ("L4", 4, 0, NodeType::Standard, Vec2::new(386.8, 562.4)),
        ("C4", 4, 1, NodeType::Standard, Vec2::new(510., 565.)),
        ("R4", 4, 2, NodeType::Standard, Vec2::new(632., 565.)),
        ("L5", 5, 0, NodeType::Standard, Vec2::new(386., 640.)),
        ("C5", 5, 1, NodeType::Standard, Vec2::new(510., 640.)),
        ("R5", 5, 2, NodeType::Standard, Vec2::new(632., 640.)),
        ("C6", 6, 1, NodeType::Standard, Vec2::new(509.2, 710.)),
        ("L7", 7, 0, NodeType::Standard, Vec2::new(387.4, 780.)),
        ("C7", 7, 1, NodeType::Standard, Vec2::new(510., 780.)),
        ("R7", 7, 2, NodeType::Standard, Vec2::new(632., 780.)),
        ("C8", 8, 1, NodeType::FoxStart, Vec2::new(510., 850.)),
    ];

    raw_nodes
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
        .collect()
}

/// Edges for the Fox and Dogs board.
fn fox_and_dogs_raw_edges() -> Vec<(&'static str, &'static str)> {
    vec![
        // Central axis
        ("C0", "C1"),
        ("C1", "C2"),
        ("C2", "C3"),
        ("C3", "C4"),
        ("C4", "C5"),
        ("C5", "C6"),
        ("C6", "C7"),
        ("C7", "C8"),
        // 0 -> 1
        ("C0", "L1"),
        ("C0", "R1"),
        // Rows 1 and 2
        ("L1", "L2"),
        ("R1", "R2"),
        ("L1", "C1"),
        ("R1", "C1"),
        ("L1", "C2"),
        ("R1", "C2"),
        ("L2", "C2"),
        ("R2", "C2"),
        // 3 -> 4
        ("C3", "L4"),
        ("C3", "R4"),
        ("L4", "L5"),
        ("R4", "R5"),
        // Row 4 horizontal
        ("L4", "C4"),
        ("C4", "R4"),
        // 4 -> 5 diagonal
        ("C4", "L5"),
        ("C4", "R5"),
        // Row 5 horizontal
        ("L5", "C5"),
        ("C5", "R5"),
        // 5 -> 6 and outputs to dogs
        ("L5", "L7"),
        ("R5", "R7"),
        ("L5", "C6"),
        ("R5", "C6"),
        // 6 -> dogs
        ("C6", "L7"),
        ("C6", "R7"),
        // Row 7 horizontal
        ("L7", "C7"),
        ("C7", "R7"),
        // 7 -> C8 (Target Coop)
        ("L7", "C8"),
        ("R7", "C8"),
    ]
}

pub fn build_fox_and_dogs_graph() -> Graph {
    let nodes = build_arthur_nodes();
    let raw_edges = fox_and_dogs_raw_edges();

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

pub use build_fox_and_dogs_graph as build_arthur_symmetric_graph;

pub fn build_the_red_hunt_graph() -> Graph {
    let raw_nodes = vec![
        // Row 0: North row (Chicken Coop destination)
        ("L0", 0, 0, NodeType::Standard, Vec2::new(380., 220.)),
        ("C0", 0, 1, NodeType::TargetCoop, Vec2::new(512.0, 220.)),
        ("R0", 0, 2, NodeType::Standard, Vec2::new(645., 220.)),
        // Row 1
        ("C1", 1, 1, NodeType::Standard, Vec2::new(512.0, 284.)),
        // Row 2
        ("L2", 2, 0, NodeType::Standard, Vec2::new(380., 345.)),
        ("C2", 2, 1, NodeType::Standard, Vec2::new(512.0, 345.)),
        ("R2", 2, 2, NodeType::Standard, Vec2::new(645., 345.)),
        // Row 3: Top star node (upper hound start)
        ("C3", 3, 1, NodeType::Standard, Vec2::new(512.0, 406.)),
        // Row 4: Central line (stars at L4 & R4, Fox start at C4)
        ("L4", 4, 0, NodeType::Standard, Vec2::new(380., 470.)),
        ("C4", 4, 1, NodeType::FoxStart, Vec2::new(512.0, 470.)),
        ("R4", 4, 2, NodeType::Standard, Vec2::new(645., 470.)),
        // Row 5: Bottom diamond center
        ("C5", 5, 1, NodeType::Standard, Vec2::new(512.0, 536.0)),
        // Row 6: North bridge landing (Perekop)
        ("L6", 6, 0, NodeType::Standard, Vec2::new(380., 590.)),
        ("C6", 6, 1, NodeType::Bottleneck, Vec2::new(512.0, 590.)),
        ("R6", 6, 2, NodeType::Standard, Vec2::new(645., 590.)),
        // Row 7: South bridge landing (Perekop)
        ("L7", 7, 0, NodeType::Standard, Vec2::new(380., 683.)),
        ("C7", 7, 1, NodeType::Bottleneck, Vec2::new(512.0, 683.)),
        ("R7", 7, 2, NodeType::Standard, Vec2::new(645., 683.)),
        // Row 8
        ("L8", 8, 0, NodeType::Standard, Vec2::new(381., 744.)),
        ("C8", 8, 1, NodeType::Standard, Vec2::new(512.0, 744.)),
        ("R8", 8, 2, NodeType::Standard, Vec2::new(645., 744.)),
        // Row 9: South apex
        ("C9", 9, 1, NodeType::Standard, Vec2::new(512.0, 808.0)),
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
        // Central vertical axis
        ("C0", "C1"),
        ("C1", "C2"),
        ("C2", "C3"),
        ("C3", "C4"),
        ("C4", "C5"),
        ("C5", "C6"),
        ("C6", "C7"), // Perekop bridge across the fault
        ("C7", "C8"),
        ("C8", "C9"),
        // Horizontal connections
        ("L0", "C0"),
        ("C0", "R0"),
        ("L2", "C2"),
        ("C2", "R2"),
        ("L4", "C4"),
        ("C4", "R4"),
        ("L6", "C6"),
        ("C6", "R6"),
        ("L7", "C7"),
        ("C7", "R7"),
        ("L8", "C8"),
        ("C8", "R8"),
        // Lateral vertical connections
        ("L0", "L2"),
        ("R0", "R2"),
        ("L2", "L4"),
        ("R2", "R4"),
        ("L4", "L6"),
        ("R4", "R6"),
        ("L7", "L8"),
        ("R7", "R8"),
        // Diagonal: North field (Rows 0-2 through C1)
        ("L0", "C1"),
        ("R0", "C1"),
        ("C1", "L2"),
        ("C1", "R2"),
        // Diagonal: Upper diamond (Rows 2-4 through C3)
        ("L2", "C3"),
        ("R2", "C3"),
        ("C3", "L4"),
        ("C3", "R4"),
        // Diagonal: Lower diamond (Rows 4-6 through C5)
        ("L4", "C5"),
        ("R4", "C5"),
        ("C5", "L6"),
        ("C5", "R6"),
        // Diagonal: South triangle (Rows 8-9 to C9)
        ("L8", "C9"),
        ("R8", "C9"),
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
