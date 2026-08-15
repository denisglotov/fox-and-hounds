# Fox and Hounds (Лиса и Гончие)

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

### 4. Victory Conditions
| Faction | Objective / Victory Condition |
| :--- | :--- |
| 🦊 **Fox** | Reaches the **Chicken Coop** vertex (located at the opposite end of the graph, behind the Hounds). |
| 🐶 **Hounds** | **Completely traps** the Fox such that the Fox has **zero legal moves** on its turn. |

---

## 🗺️ Featured Level: "The River Crossing" (3x10 Bottleneck)

A tactical campaign map featuring open flanking grounds, a river bottleneck bridge on Row 7, and the Chicken Coop behind the Hounds' defensive line.

### Graph Architecture
- **Row 0** (Goal): `C0` — The Chicken Coop target vertex (1 vertex).
- **Row 1** (Hounds Start): `H1`, `H2`, `H3` — Starting posts of the 3 Hounds (3 vertices).
- **Rows 2–6** (North Fields): 3 vertices per row (`L`eft, `M`iddle, `R`ight) with centrally symmetric diamond-grid connectivity.
- **Row 7** (The Bottleneck): `B7` — A single chokepoint bridge over the river connecting North and South fields (1 vertex).
- **Rows 8–9** (South Fields): 3 vertices per row (`L`eft, `M`iddle, `R`ight).
- **Row 10** (Fox Start): `F10` — The Fox Den start vertex (1 vertex).

---

### Mermaid Level Graph

> **Note on Layout Alignment**: Mermaid uses the Dagre ranking engine. To enforce that $L_i$, $M_i$, and $R_i$ stay strictly on the exact same horizontal level (same rank), each row is grouped with `direction LR` and styled with invisible containers (`fill:none,stroke:none`).

```mermaid
flowchart TD
    %% Node Styling
    classDef coop fill:#ffe082,stroke:#ff8f00,stroke-width:3px,color:#000,font-weight:bold;
    classDef hound fill:#bbdefb,stroke:#1976d2,stroke-width:2px,color:#000,font-weight:bold;
    classDef bridge fill:#ffcdd2,stroke:#d32f2f,stroke-width:3px,color:#000,font-weight:bold;
    classDef fox fill:#ffcc80,stroke:#e65100,stroke-width:3px,color:#000,font-weight:bold;
    classDef field fill:#f5f5f5,stroke:#757575,stroke-width:1px,color:#212121;
    classDef invisible fill:none,stroke:none;

    %% Row 0: Chicken Coop
    C0(["🐔 C0 (Chicken Coop)"]):::coop

    %% Row 1: Hounds Start (Strict Horizontal Alignment)
    subgraph Row1 [" "]
        direction LR
        H1["🐶 H1 (L1)"]:::hound --- H2["🐶 H2 (M1)"]:::hound --- H3["🐶 H3 (R1)"]:::hound
    end

    %% Rows 2 to 6: North Fields (Strict Horizontal Alignment)
    subgraph Row2 [" "]
        direction LR
        L2["L2"]:::field --- M2["M2"]:::field --- R2["R2"]:::field
    end

    subgraph Row3 [" "]
        direction LR
        L3["L3"]:::field --- M3["M3"]:::field --- R3["R3"]:::field
    end

    subgraph Row4 [" "]
        direction LR
        L4["L4"]:::field --- M4["M4"]:::field --- R4["R4"]:::field
    end

    subgraph Row5 [" "]
        direction LR
        L5["L5"]:::field --- M5["M5"]:::field --- R5["R5"]:::field
    end

    subgraph Row6 [" "]
        direction LR
        L6["L6"]:::field --- M6["M6"]:::field --- R6["R6"]:::field
    end

    %% Row 7: River Bottleneck Bridge
    B7{{"🌉 B7 (River Bridge)"}}:::bridge

    %% Rows 8 & 9: South Fields (Strict Horizontal Alignment)
    subgraph Row8 [" "]
        direction LR
        L8["L8"]:::field --- M8["M8"]:::field --- R8["R8"]:::field
    end

    subgraph Row9 [" "]
        direction LR
        L9["L9"]:::field --- M9["M9"]:::field --- R9["R9"]:::field
    end

    %% Row 10: Fox Start
    F10(["🦊 F10 (Fox Den)"]):::fox

    %% Hide Subgraph Containers
    class Row1,Row2,Row3,Row4,Row5,Row6,Row8,Row9 invisible;

    %% Row 0 <-> Row 1
    C0 --- H1
    C0 --- H2
    C0 --- H3

    %% Row 1 <-> Row 2 Connections
    H1 --- L2
    H2 --- M2
    H3 --- R2
    H1 --- M2
    H3 --- M2
    H2 --- L2
    H2 --- R2

    %% Row 2 <-> Row 3 Connections
    L2 --- L3
    M2 --- M3
    R2 --- R3
    L2 --- M3
    R2 --- M3
    M2 --- L3
    M2 --- R3

    %% Row 3 <-> Row 4 Connections
    L3 --- L4
    M3 --- M4
    R3 --- R4
    L3 --- M4
    R3 --- M4
    M3 --- L4
    M3 --- R4

    %% Row 4 <-> Row 5 Connections
    L4 --- L5
    M4 --- M5
    R4 --- R5
    L4 --- M5
    R4 --- M5
    M4 --- L5
    M4 --- R5

    %% Row 5 <-> Row 6 Connections
    L5 --- L6
    M5 --- M6
    R5 --- R6
    L5 --- M6
    R5 --- M6
    M5 --- L6
    M5 --- R6

    %% Row 6 <-> Row 7 Bridge
    L6 --- B7
    M6 --- B7
    R6 --- B7

    %% Row 7 Bridge <-> Row 8
    B7 --- L8
    B7 --- M8
    B7 --- R8

    %% Row 8 <-> Row 9 Connections
    L8 --- L9
    M8 --- M9
    R8 --- R9
    L8 --- M9
    R8 --- M9
    M8 --- L9
    M8 --- R9

    %% Row 9 <-> Row 10 Fox
    L9 --- F10
    M9 --- F10
    R9 --- F10
```

---

### Graphviz DOT Specification

For standard Graphviz tools, the layout uses explicit `{ rank=same; ... }` constraints to guarantee pixel-perfect horizontal rows:

```dot
graph FoxAndHounds {
    layout=dot;
    rankdir=TB;
    nodesep=0.6;
    ranksep=0.5;

    node [shape=circle, style=filled, fillcolor="#f5f5f5", color="#757575", fontname="Helvetica", width=0.8, fixedsize=true];
    edge [color="#424242", penwidth=1.5];

    // Explicit Horizontal Rank Alignment
    { rank=same; C0 [label="🐔 C0", fillcolor="#ffe082", color="#ff8f00", penwidth=2.5]; }
    { rank=same; H1 [label="🐶 H1", fillcolor="#bbdefb", color="#1976d2", penwidth=2.0]; H2 [label="🐶 H2", fillcolor="#bbdefb", color="#1976d2", penwidth=2.0]; H3 [label="🐶 H3", fillcolor="#bbdefb", color="#1976d2", penwidth=2.0]; }
    { rank=same; L2; M2; R2; }
    { rank=same; L3; M3; R3; }
    { rank=same; L4; M4; R4; }
    { rank=same; L5; M5; R5; }
    { rank=same; L6; M6; R6; }
    { rank=same; B7 [label="🌉 B7", fillcolor="#ffcdd2", color="#d32f2f", penwidth=2.5, shape=hexagon]; }
    { rank=same; L8; M8; R8; }
    { rank=same; L9; M9; R9; }
    { rank=same; F10 [label="🦊 F10", fillcolor="#ffcc80", color="#e65100", penwidth=2.5]; }

    // Row 0 <-> Row 1
    C0 -- {H1; H2; H3};

    // Horizontal Row Edges
    H1 -- H2 -- H3;
    L2 -- M2 -- R2;
    L3 -- M3 -- R3;
    L4 -- M4 -- R4;
    L5 -- M5 -- R5;
    L6 -- M6 -- R6;
    L8 -- M8 -- R8;
    L9 -- M9 -- R9;

    // Symmetric North Field Edges (Rows 1 to 6)
    H1 -- {L2; M2};  H2 -- {L2; M2; R2};  H3 -- {M2; R2};
    L2 -- {L3; M3};  M2 -- {L3; M3; R3};  R2 -- {M3; R3};
    L3 -- {L4; M4};  M3 -- {L4; M4; R4};  R3 -- {M4; R4};
    L4 -- {L5; M5};  M4 -- {L5; M5; R5};  R4 -- {M5; R5};
    L5 -- {L6; M6};  M5 -- {L6; M6; R6};  R5 -- {M6; R6};

    // Bottleneck River Bridge (Row 6 <-> Row 7 <-> Row 8)
    {L6; M6; R6} -- B7;
    B7 -- {L8; M8; R8};

    // Symmetric South Field Edges (Rows 8 to 10)
    L8 -- {L9; M9};  M8 -- {L9; M9; R9};  R8 -- {M9; R9};
    {L9; M9; R9} -- F10;
}
```

---

## 📊 Level Data Specification (JSON Schema)

Structured graph representation matching the symmetric level layout for programmatic game engine loading:

```json
{
  "name": "The River Crossing",
  "description": "3x10 board with a river bottleneck on Row 7 and centrally symmetric diamond connectivity",
  "dimensions": {
    "rows": 11,
    "max_width": 3
  },
  "initial_state": {
    "fox_start": "F10",
    "hounds_start": ["H1", "H2", "H3"],
    "coop_targets": ["C0"],
    "current_turn": "Fox"
  },
  "nodes": [
    { "id": "C0", "row": 0, "col": 1, "type": "target_coop" },
    { "id": "H1", "row": 1, "col": 0, "type": "standard" },
    { "id": "H2", "row": 1, "col": 1, "type": "standard" },
    { "id": "H3", "row": 1, "col": 2, "type": "standard" },
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
    { "id": "L6", "row": 6, "col": 0, "type": "standard" },
    { "id": "M6", "row": 6, "col": 1, "type": "standard" },
    { "id": "R6", "row": 6, "col": 2, "type": "standard" },
    { "id": "B7", "row": 7, "col": 1, "type": "bottleneck" },
    { "id": "L8", "row": 8, "col": 0, "type": "standard" },
    { "id": "M8", "row": 8, "col": 1, "type": "standard" },
    { "id": "R8", "row": 8, "col": 2, "type": "standard" },
    { "id": "L9", "row": 9, "col": 0, "type": "standard" },
    { "id": "M9", "row": 9, "col": 1, "type": "standard" },
    { "id": "R9", "row": 9, "col": 2, "type": "standard" },
    { "id": "F10", "row": 10, "col": 1, "type": "fox_start" }
  ],
  "edges": [
    ["C0", "H1"], ["C0", "H2"], ["C0", "H3"],
    ["H1", "H2"], ["H2", "H3"],
    ["H1", "L2"], ["H2", "M2"], ["H3", "R2"],
    ["H1", "M2"], ["H3", "M2"], ["H2", "L2"], ["H2", "R2"],
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
    ["L5", "L6"], ["M5", "M6"], ["R5", "R6"],
    ["L5", "M6"], ["R5", "M6"], ["M5", "L6"], ["M5", "R6"],
    ["L6", "M6"], ["M6", "R6"],
    ["L6", "B7"], ["M6", "B7"], ["R6", "B7"],
    ["B7", "L8"], ["B7", "M8"], ["B7", "R8"],
    ["L8", "M8"], ["M8", "R8"],
    ["L8", "L9"], ["M8", "M9"], ["R8", "R9"],
    ["L8", "M9"], ["R8", "M9"], ["M8", "L9"], ["M8", "R9"],
    ["L9", "M9"], ["M9", "R9"],
    ["L9", "F10"], ["M9", "F10"], ["R9", "F10"]
  ]
}
```

---

## 🎯 Key Strategic Concepts

### For the Fox 🦊
1. **Bottleneck Timing**: The bridge at Row 7 is both a barrier and a launchpad. The Fox should feint on Row 8/9 to lure hounds out of formation before dashing through `B7`.
2. **Tempo & Flanking**: Draw two hounds toward one flank, then pivot through the symmetric diagonal connections to exploit the vacant opposite lane.
3. **Penetration Victory**: Once past the defensive line into Row 1, the Hound pack cannot recover if the Chicken Coop (`C0`) is within one move.

### For the Hounds 🐶
1. **Cohesive Wall Formation**: Hounds should advance in rank or hold key cross-lanes (`L`, `M`, `R`) to prevent the Fox from slipping between gaps.
2. **Bridge Lockout**: Controlling `B7` or establishing a blockade on Row 6 (`L6`, `M6`, `R6`) prevents the Fox from crossing the river.
3. **Corner Containment**: Drive the Fox toward boundary nodes (`L` or `R`) and collapse adjacent degrees of freedom to achieve checkmate (0 legal moves).
