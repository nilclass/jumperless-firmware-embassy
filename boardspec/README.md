# Board specs

Each subdirectory contains board-specific information about port mappings. This info is used by the `board-spec-code-generator` (currently in the `jumperless-types` package) to generate board-specific code.

- `nodes.txt`: Each line maps a node to a port. A node can be mapped to multiple ports, each on a separate line.
- `lanes.txt`: Each line is a pair of ports, representing a lane.
- `bounceports.txt`: Each line is a port that can be used as a bounce port (i.e. is not connected to anything).
