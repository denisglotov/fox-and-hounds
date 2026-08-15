# Fox and Hounds

A turn-based asymmetric pursuit-evasion game played on arbitrary graph topologies.

One clever **Fox** ($\text{F}$) matches wits against a pack of **$N$ Guard Dogs** ($\text{D}_1, \text{D}_2, \dots, \text{D}_N$). The Fox aims to slip past the pack and infiltrate the **Chicken Coop**, while the Dogs must coordinate as a cohesive unit to encircle and immobilize the Fox.

---

## 📖 Game Rules & Mechanics

### 1. Board as a Graph
- The game is played on an undirected graph $G = (V, E)$, where:
  - **Vertices ($V$)**: Discrete positions where pieces stand.
  - **Edges ($E$)**: Valid pathways connecting adjacent vertices.

### 2. Turn Order & Flow
- The game is strictly **turn-based**.
- **Fox Moves First**: The Fox takes turn 1, followed by the Dogs.
- **Single Piece Movement**:
  - On the **Fox's turn**: The Fox moves to an adjacent, unoccupied vertex.
  - On the **Dogs' turn**: The Dogs' player selects and moves **exactly one Dog** to an adjacent, unoccupied vertex.

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
| 🦊 **Fox** | Reaches the **Chicken Coop** vertex (located at the opposite end of the graph, behind the Dogs). |
| 🐶 **Dogs** | **Completely traps** the Fox such that the Fox has **zero legal moves** on its turn. |

---

## 🗺️ Featured Level: "The River Crossing" (3x10 Bottleneck)

A tactical campaign map featuring open flanking grounds, a river bottleneck bridge on Row 7, and the Chicken Coop behind the Dogs' defensive line.

### Graph Architecture
- **Row 0** (Goal): `Coop` — The Chicken Coop target vertex (1 vertex).
- **Row 1** (Dog Start): `D1`, `D2`, `D3` — Starting posts of the 3 Guard Dogs (3 vertices).
- **Rows 2–6** (North Fields): 3 vertices per row (`L`eft, `M`iddle, `R`ight) with grid and diagonal path connectivity.
- **Row 7** (The Bottleneck): `Bridge` — A single chokepoint bridge over the river connecting North and South fields (1 vertex).
- **Rows 8–9** (South Fields): 3 vertices per row (`L`eft, `M`iddle, `R`ight).
- **Row 10** (Fox Start): `Fox` — The Fox Den start vertex (1 vertex).

---

### Mermaid Level Graph

```mermaid
flowchart TD
    %% Styling Classes
    classDef coop fill:#ffe082,stroke:#ff8f00,stroke-width:3px,color:#000,font-weight:bold;
    classDef dog fill:#bbdefb,stroke:#1976d2,stroke-width:2px,color:#000,font-weight:bold;
    classDef bridge fill:#ffcdd2,stroke:#d32f2f,stroke-width:3px,color:#000,font-weight:bold;
    classDef fox fill:#ffcc80,stroke:#e65100,stroke-width:3px,color:#000,font-weight:bold;
    classDef field fill:#f5f5f5,stroke:#9e9e9e,stroke-width:1px,color:#333;

    %% Row 0: Chicken Coop
    C0(["🐔 Chicken Coop (Row 0)"]):::coop

    %% Row 1: Dogs Starting Positions
    subgraph Row_1 ["Row 1 (Dogs Start)"]
        D1["🐶 D1 (L1)"]:::dog
        D2["🐶 D2 (M1)"]:::dog
        D3["🐶 D3 (R1)"]:::dog
    end

    %% Rows 2 to 6: North Fields
    subgraph Row_2 ["Row 2"]
        L2["L2"]:::field --- M2["M2"]:::field --- R2["R2"]:::field
    end

    subgraph Row_3 ["Row 3"]
        L3["L3"]:::field --- M3["M3"]:::field --- R3["R3"]:::field
    end

    subgraph Row_4 ["Row 4"]
        L4["L4"]:::field --- M4["M4"]:::field --- R4["R4"]:::field
    end

    subgraph Row_5 ["Row 5"]
        L5["L5"]:::field --- M5["M5"]:::field --- R5["R5"]:::field
    end

    subgraph Row_6 ["Row 6 (North Shore)"]
        L6["L6"]:::field --- M6["M6"]:::field --- R6["R6"]:::field
    end

    %% Row 7: River Bottleneck
    B7{{"🌉 Bottleneck Bridge (Row 7)"}}:::bridge

    %% Rows 8 & 9: South Fields
    subgraph Row_8 ["Row 8 (South Shore)"]
        L8["L8"]:::field --- M8["M8"]:::field --- R8["R8"]:::field
    end

    subgraph Row_9 ["Row 9"]
        L9["L9"]:::field --- M9["M9"]:::field --- R9["R9"]:::field
    end

    %% Row 10: Fox Start
    F10(["🦊 Fox Den (Row 10)"]):::fox

    %% Row 0 <-> Row 1 Connections
    C0 --- D1
    C0 --- D2
    C0 --- D3

    %% Row 1 Internal Horizontal
    D1 --- D2 --- D3

    %% Row 1 <-> Row 2 Connections
    D1 --- L2
    D1 --- M2
    D2 --- L2
    D2 --- M2
    D2 --- R2
    D3 --- M2
    D3 --- R2

    %% Row 2 <-> Row 3 Connections
    L2 --- L3
    L2 --- M3
    M2 --- L3
    M2 --- M3
    M2 --- R3
    R2 --- M3
    R2 --- R3

    %% Row 3 <-> Row 4 Connections
    L3 --- L4
    L3 --- M4
    M3 --- L4
    M3 --- M4
    M3 --- R4
    R3 --- M4
    R3 --- R4

    %% Row 4 <-> Row 5 Connections
    L4 --- L5
    L4 --- M5
    M4 --- L5
    M4 --- M5
    M4 --- R5
    R4 --- M5
    R4 --- R5

    %% Row 5 <-> Row 6 Connections
    L5 --- L6
    L5 --- M6
    M5 --- L6
    M5 --- M6
    M5 --- R6
    R5 --- M6
    R5 --- R6

    %% Row 6 <-> Row 7 (Bottleneck Bridge)
    L6 --- B7
    M6 --- B7
    R6 --- B7

    %% Row 7 (Bottleneck Bridge) <-> Row 8
    B7 --- L8
    B7 --- M8
    B7 --- R8

    %% Row 8 <-> Row 9 Connections
    L8 --- L9
    L8 --- M9
    M8 --- L9
    M8 --- M9
    M8 --- R9
    R8 --- M9
    R8 --- R9

    %% Row 9 <-> Row 10 (Fox Start)
    L9 --- F10
    M9 --- F10
    R9 --- F10
```

---

## 📊 Level Data Specification (JSON Schema)

To support programmatic loading and verification (e.g. in Rust game engines), levels can be represented as structured graphs:

```json
{
  "name": "The River Crossing",
  "description": "3x10 board with a river bottleneck on Row 7",
  "dimensions": {
    "rows": 11,
    "max_width": 3
  },
  "initial_state": {
    "fox_start": "F10",
    "dogs_start": ["D1", "D2", "D3"],
    "coop_targets": ["C0"],
    "current_turn": "Fox"
  },
  "nodes": [
    { "id": "C0", "row": 0, "col": 1, "type": "target_coop" },
    { "id": "D1", "row": 1, "col": 0, "type": "standard" },
    { "id": "D2", "row": 1, "col": 1, "type": "standard" },
    { "id": "D3", "row": 1, "col": 2, "type": "standard" },
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
    ["C0", "D1"], ["C0", "D2"], ["C0", "D3"],
    ["D1", "D2"], ["D2", "D3"],
    ["D1", "L2"], ["D1", "M2"], ["D2", "L2"], ["D2", "M2"], ["D2", "R2"], ["D3", "M2"], ["D3", "R2"],
    ["L2", "M2"], ["M2", "R2"],
    ["L2", "L3"], ["L2", "M3"], ["M2", "L3"], ["M2", "M3"], ["M2", "R3"], ["R2", "M3"], ["R2", "R3"],
    ["L3", "M3"], ["M3", "R3"],
    ["L3", "L4"], ["L3", "M4"], ["M3", "L4"], ["M3", "M4"], ["M3", "R4"], ["R3", "M4"], ["R3", "R4"],
    ["L4", "M4"], ["M4", "R4"],
    ["L4", "L5"], ["L4", "M5"], ["M4", "L5"], ["M4", "M5"], ["M4", "R5"], ["R4", "M5"], ["R4", "R5"],
    ["L5", "M5"], ["M5", "R5"],
    ["L5", "L6"], ["L5", "M6"], ["M5", "L6"], ["M5", "M6"], ["M5", "R6"], ["R5", "M6"], ["R5", "R6"],
    ["L6", "M6"], ["M6", "R6"],
    ["L6", "B7"], ["M6", "B7"], ["R6", "B7"],
    ["B7", "L8"], ["B7", "M8"], ["B7", "R8"],
    ["L8", "M8"], ["M8", "R8"],
    ["L8", "L9"], ["L8", "M9"], ["M8", "L9"], ["M8", "M9"], ["M8", "R9"], ["R8", "M9"], ["R8", "R9"],
    ["L9", "M9"], ["M9", "R9"],
    ["L9", "F10"], ["M9", "F10"], ["R9", "F10"]
  ]
}
```

---

## 🎯 Key Strategic Concepts

### For the Fox 🦊
1. **Bottleneck Timing**: The bridge at Row 7 is both a barrier and a launchpad. The Fox should feint on Row 8/9 to lure dogs out of formation before dashing through `B7`.
2. **Tempo & Flanking**: Draw two dogs toward one flank, then pivot through diagonal connections to exploit the vacant opposite lane.
3. **Penetration Victory**: Once past the defensive line into Row 1, the Dog pack cannot recover if the Chicken Coop (`C0`) is within one move.

### For the Dogs 🐶
1. **Cohesive Wall Formation**: Dogs should advance in rank or hold key cross-lanes (`L`, `M`, `R`) to prevent the Fox from slipping between gaps.
2. **Bridge Lockout**: Controlling `B7` or establishing a blockade on Row 6 (`L6`, `M6`, `R6`) prevents the Fox from crossing the river.
3. **Corner Containment**: Drive the Fox toward boundary nodes (`L` or `R`) and collapse adjacent degrees of freedom to achieve checkmate (0 legal moves).
