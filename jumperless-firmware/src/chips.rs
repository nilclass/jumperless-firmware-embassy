use crate::{ch446q, nets};

struct Chip {
    x: [Port; 16],
    y: [Port; 8],
}

enum Dimension {
    X,
    Y,
}

enum Port {
    Lane(ch446q::Chip, Dimension, u8),
    Node(nets::Node),
}

// fn chips() -> [Chip; 12] {
//     use Port::*;
//     use ch446q::Chip::*;
//     use Dimension::*;
//     [
//         Chip {
//             x: [
//                 Lane(I, Y, 0),
//                 Lane(J, Y, 0),

//             ]
//         },
//     ]
// }
