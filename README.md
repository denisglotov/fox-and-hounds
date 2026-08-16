# Fox and Hounds

A turn-based asymmetric pursuit-evasion game played on arbitrary graph topologies.

One clever **Fox** ($\text{F}$) matches wits against a pack of **$N$ Hounds** ($\text{H}_1, \text{H}_2, \dots, \text{H}_N$). The Fox aims to slip past the pack and infiltrate the **Chicken Coop**, while the Hounds must coordinate as a cohesive unit to encircle and immobilize the Fox.

---

## 📖 Game Rules & Mechanics

### 1. Board as a Graph
- The game is played on an undirected graph $G = (V, E)$, where:
  - **Vertices ($V$)**: Discrete positions where pieces stand.
  - **Edges ($E$)**: Valid pathways connecting adjacent vertices.

### 2. Turn Order & Flow
- The game is strictly **turn-based**.
- **Fox Moves First**: The Fox takes turn 1, followed by the Hounds.
- **Single Piece Movement**:
  - On the **Fox's turn**: The Fox moves to an adjacent, unoccupied vertex.
  - On the **Hounds' turn**: The Hounds' player selects and moves **exactly one Hound** to an adjacent, unoccupied vertex.

### 3. Movement Rules
- **Bidirectional Traversal**: Pieces can move along any connected edge (forward, backward, or sideways).
- **Step Size**: Exactly one edge traversal per turn.
- **Occupancy & No Collisions**:
  - No two pieces may occupy the same vertex simultaneously.
  - Pieces cannot jump over or pass through occupied vertices.
- **No Captures**:
  - Pieces cannot be captured or eliminated. The game is purely positional and tactical.
- **Chicken Coop Sanctuary**:
  - **Hounds (Dogs) cannot occupy or enter the Chicken Coop (`M0`)**. The Chicken Coop is strictly an infiltration target for the Fox.

### 4. Victory Conditions
| Faction | Objective / Victory Condition |
| :--- | :--- |
| 🦊 **Fox** | Reaches the **Chicken Coop** vertex (located at the opposite end of the graph, behind the Hounds). |
| 🐶 **Hounds** | **Completely traps** the Fox such that the Fox has **zero legal moves** on its turn. |

---

## 🗺️ Featured Level: "The River Crossing" (3x9 Bottleneck)

A tactical campaign map featuring open flanking grounds, a river bottleneck bridge on Row 6, and the Chicken Coop behind the Hounds' defensive line.

### Graph Architecture
- **Row 0** (Goal): `M0` — The Chicken Coop target vertex (1 vertex).
- **Row 1** (Hounds Start): `L1`, `M1`, `R1` — Starting posts of the 3 Hounds (3 vertices).
- **Rows 2–5** (North Fields): 3 vertices per row (`L`eft, `M`iddle, `R`ight) with centrally symmetric diamond-grid connectivity.
- **Row 6** (The Bottleneck): `M6` — A single chokepoint bridge over the river connecting North and South fields (1 vertex).
- **Rows 7–8** (South Fields): 3 vertices per row (`L`eft, `M`iddle, `R`ight).
- **Row 9** (Fox Start): `M9` — The Fox Den start vertex (1 vertex).

---

### Board Graph Visualization

<p align="center">
  <img src="assets/board_graph.svg" alt="The River Crossing Board Graph" width="380" />
</p>

---

### Graphviz DOT Specification

```dot
graph FoxAndHounds {
    layout=dot;
    rankdir=TB;
    nodesep=0.6;
    ranksep=0.5;

    node [shape=circle, style=filled, fillcolor="#f5f5f5", color="#757575", fontname="Helvetica", width=0.8, fixedsize=true];
    edge [color="#424242", penwidth=1.5];

    // Explicit Horizontal Rank Alignment
    { rank=same; M0 [label="🐔 M0", fillcolor="#ffe082", color="#ff8f00", penwidth=2.5]; }
    { rank=same; L1 [label="🐶 L1", fillcolor="#bbdefb", color="#1976d2", penwidth=2.0]; M1 [label="🐶 M1", fillcolor="#bbdefb", color="#1976d2", penwidth=2.0]; R1 [label="🐶 R1", fillcolor="#bbdefb", color="#1976d2", penwidth=2.0]; }
    { rank=same; L2; M2; R2; }
    { rank=same; L3; M3; R3; }
    { rank=same; L4; M4; R4; }
    { rank=same; L5; M5; R5; }
    { rank=same; M6 [label="M6"]; }
    { rank=same; L7; M7; R7; }
    { rank=same; L8; M8; R8; }
    { rank=same; M9 [label="🦊 M9", fillcolor="#ffcc80", color="#e65100", penwidth=2.5]; }

    // Row 0 <-> Row 1
    M0 -- {L1; M1; R1};

    // Horizontal Row Edges
    L1 -- M1 -- R1;
    L2 -- M2 -- R2;
    L3 -- M3 -- R3;
    L4 -- M4 -- R4;
    L5 -- M5 -- R5;
    L7 -- M7 -- R7;
    L8 -- M8 -- R8;

    // Symmetric North Field Edges (Rows 1 to 5)
    L1 -- {L2; M2};  M1 -- {L2; M2; R2};  R1 -- {M2; R2};
    L2 -- {L3; M3};  M2 -- {L3; M3; R3};  R2 -- {M3; R3};
    L3 -- {L4; M4};  M3 -- {L4; M4; R4};  R3 -- {M4; R4};
    L4 -- {L5; M5};  M4 -- {L5; M5; R5};  R4 -- {M5; R5};

    // Bottleneck River Bridge (Row 5 <-> Row 6 <-> Row 7)
    {L5; M5; R5} -- M6;
    M6 -- {L7; M7; R7};

    // Symmetric South Field Edges (Rows 7 to 9)
    L7 -- {L8; M8};  M7 -- {L8; M8; R8};  R7 -- {M8; R8};
    {L8; M8; R8} -- M9;
}
```

---

## 📊 Level Data Specification (JSON Schema)

Structured graph representation matching the symmetric level layout for programmatic game engine loading:

```json
{
  "name": "The River Crossing",
  "description": "3x9 board with a river bottleneck on Row 6 and centrally symmetric diamond connectivity",
  "dimensions": {
    "rows": 10,
    "max_width": 3
  },
  "initial_state": {
    "fox_start": "M9",
    "hounds_start": ["L1", "M1", "R1"],
    "coop_targets": ["M0"],
    "current_turn": "Fox"
  },
  "nodes": [
    { "id": "M0", "row": 0, "col": 1, "type": "target_coop" },
    { "id": "L1", "row": 1, "col": 0, "type": "standard" },
    { "id": "M1", "row": 1, "col": 1, "type": "standard" },
    { "id": "R1", "row": 1, "col": 2, "type": "standard" },
    { "id": "L2", "row": 2, "col": 0, "type": "standard" },
    { "id": "M2", "row": 2, "col": 1, "type": "standard" },
    { "id": "R2", "row": 2, "col": 2, "type": "standard" },
    { "id": "L3", "row": 3, "col": 0, "type": "standard" },
    { "id": "M3", "row": 3, "col": 1, "type": "standard" },
    { "id": "R3", "row": 3, "col": 2, "type": "standard" },
    { "id": "L4", "row": 4, "col": 0, "type": "standard" },
    { "id": "M4", "row": 4, "col": 1, "type": "standard" },
    { "id": "R4", "row": 4, "col": 2, "type": "standard" },
    { "id": "L5", "row": 5, "col": 0, "type": "standard" },
    { "id": "M5", "row": 5, "col": 1, "type": "standard" },
    { "id": "R5", "row": 5, "col": 2, "type": "standard" },
    { "id": "M6", "row": 6, "col": 1, "type": "bottleneck" },
    { "id": "L7", "row": 7, "col": 0, "type": "standard" },
    { "id": "M7", "row": 7, "col": 1, "type": "standard" },
    { "id": "R7", "row": 7, "col": 2, "type": "standard" },
    { "id": "L8", "row": 8, "col": 0, "type": "standard" },
    { "id": "M8", "row": 8, "col": 1, "type": "standard" },
    { "id": "R8", "row": 8, "col": 2, "type": "standard" },
    { "id": "M9", "row": 9, "col": 1, "type": "fox_start" }
  ],
  "edges": [
    ["M0", "L1"], ["M0", "M1"], ["M0", "R1"],
    ["L1", "M1"], ["M1", "R1"],
    ["L1", "L2"], ["M1", "M2"], ["R1", "R2"],
    ["L1", "M2"], ["R1", "M2"], ["M1", "L2"], ["M1", "R2"],
    ["L2", "M2"], ["M2", "R2"],
    ["L2", "L3"], ["M2", "M3"], ["R2", "R3"],
    ["L2", "M3"], ["R2", "M3"], ["M2", "L3"], ["M2", "R3"],
    ["L3", "M3"], ["M3", "R3"],
    ["L3", "L4"], ["M3", "M4"], ["R3", "R4"],
    ["L3", "M4"], ["R3", "M4"], ["M3", "L4"], ["M3", "R4"],
    ["L4", "M4"], ["M4", "R4"],
    ["L4", "L5"], ["M4", "M5"], ["R4", "R5"],
    ["L4", "M5"], ["R4", "M5"], ["M4", "L5"], ["M4", "R5"],
    ["L5", "M5"], ["M5", "R5"],
    ["L5", "M6"], ["M5", "M6"], ["R5", "M6"],
    ["M6", "L7"], ["M6", "M7"], ["M6", "R7"],
    ["L7", "M7"], ["M7", "R7"],
    ["L7", "L8"], ["M7", "M8"], ["R7", "R8"],
    ["L7", "M8"], ["R7", "M8"], ["M7", "L8"], ["M7", "R8"],
    ["L8", "M8"], ["M8", "R8"],
    ["L8", "M9"], ["M8", "M9"], ["R8", "M9"]
  ]
}
```

---

## 🛠️ Build & Run Instructions

The game is built with [Rust](https://www.rust-lang.org/) and [Macroquad](https://macroquad.rs/), supporting Native Desktop (macOS, Linux, Windows), Web (WebAssembly), and Android.

A [`Justfile`](Justfile) is provided for common development tasks.

### Prerequisites
- **Rust Toolchain**: Install via [rustup](https://rustup.rs/) (stable channel).
- **Just** (optional): Install with `cargo install just` or `brew install just`.

---

### 🖥️ 1. Native Desktop

Run the native game locally with audio and high-DPI windowing:

```bash
# Run directly
cargo run
# or with Just:
just run

# Build optimized release binary
cargo build --release
```

---

### 🌐 2. WebAssembly (Browser)

To build and run the game in the browser:

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build WASM binary and prepare web assets
just install-wasm

# Serve locally at http://localhost:8080
just serve
```

Alternatively without `just`:
```bash
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/foxandhounds.wasm web/fox-and-hounds.wasm
python3 -m http.server 8080 -d web
```

---

### 📱 3. Android (APK & App Bundle)

#### Prerequisites
- Android SDK (API 35, Build-Tools 35.0.0) & NDK (r26+)
- `cargo-quad-apk`:
  ```bash
  cargo install --git https://github.com/not-fl3/cargo-quad-apk --force
  ```
- `bundletool` (for `.aab` builds): `brew install bundletool`

#### Building
```bash
# Build release APK
just build-android
# (or: cargo quad-apk build --release)

# Build Google Play release Android App Bundle (.aab) with R8 optimization
just build-aab
# (or: ./scripts/build-aab.sh)
```

---

### 🧪 4. Testing & Code Quality

```bash
# Run unit and integration tests
cargo test
# or: just test

# Run Clippy linter with strict checks
cargo clippy -- -D warnings
# or: just clippy

# Format code
cargo fmt
# or: just fmt
```

---

## 🎯 Key Strategic Concepts

### For the Fox 🦊
1. **Bottleneck Timing**: The bridge at Row 6 (`M6`) is both a barrier and a launchpad. The Fox should feint on Row 7/8 to lure hounds out of formation before dashing through `M6`.
2. **Tempo & Flanking**: Draw two hounds toward one flank, then pivot through the symmetric diagonal connections to exploit the vacant opposite lane.
3. **Penetration Victory**: Once past the defensive line into Row 1, the Hound pack cannot recover if the Chicken Coop (`M0`) is within one move.

### For the Hounds 🐶
1. **Cohesive Wall Formation**: Hounds should advance in rank or hold key cross-lanes (`L`, `M`, `R`) to prevent the Fox from slipping between gaps.
2. **Bridge Lockout**: Controlling `M6` or establishing a blockade on Row 5 (`L5`, `M5`, `R5`) prevents the Fox from crossing the river.
3. **Corner Containment**: Drive the Fox toward boundary nodes (`L` or `R`) and collapse adjacent degrees of freedom to achieve checkmate (0 legal moves).
