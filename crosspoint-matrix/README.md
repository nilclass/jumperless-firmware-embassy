# crosspoint-matrix

WIP implementation of algorithm to compute the crosspoint matrix state from a netlist.

Eventually this should be included as a library within the firmware, but for now I'm using a lot of things from `std` to speed things up, so currently the code won't compile to run on a board.

## How to use this

Currently the only entry point is the test cases. Check out the functions within `lib.rs` that are marked with `#[test]`.
Most of them call `test_netlist`, passing in a list of nets and the expected crosspoint configuration.

To check some additional scenarios, simply add a new test case or tweak an existing one.

The tests can be run with
```
cargo test
```

The tests have a bunch of output, but that's only visible when a test fails.

To see the output any way, run
```
cargo test -- --nocapture
```

## Documentation

### Terminology

- **Chip**: one of the CH446q chips. Labeled `A` to `L`.
- **Dimension**: Either `X` or `Y`.
- **Port**: Represents a specific pin of a specific crosspoint switch. Examples: `AX0` (chip `A`, dimension `X`, index `0`), `LY7` (chip `L`, dimension `Y`, index 7), `CX15` (chip `C`, dimension `X`, index 15)
- **Edge**: All the **ports** on a single dimension of a **chip**. Examples: `AX` (chip `A`, dimension `X`), `LY` (chip `L`, dimension `Y`).
- **Orthogonal edge**: Edge on the same chip with orthogonal dimension. Examples: `AX` is orthogonal to `AY`.
- **Lane**: A physical connection between two **ports**.
- **Node**: Logical name given to a leaf **port** (i.e. one not connected to a **lane**).
- **Bounce Port**: A special **port** that is not connected to anything (so can be used to bounce a signal between two **lanes** without intersecting a **node** -- this isn't used yet, since it only exists on the V5)
- **Net**: List of **Nodes** that should be interconnected.

### Algorithm

1. Take an initially empty `ChipStatus` structure. The `ChipStatus` keeps track of a net number for each **port**.
2. For every **Net**:
   1. For every **node** assign net number to the corresponding **port**.
   2. Identify the **edges** of those ports, and keep a list of their **orthogonals** (these are all the edges that are missing connections)
   3. Check how many edges to connect:
      - Just one: choose a random free lane that touches the edge, and assign the net to it's ports (this is done at the very end, when the chip interconnections are done)
      - Two: choose a random free lane that touches both edges. If there is no matching lane, 

(to be continued)
