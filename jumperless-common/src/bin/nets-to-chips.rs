use std::{env::args, process::exit};

use jumperless_common::{
    ChipStatus, CrosspointConfig,
    board::{init_board, Node},
    types::Net,
    nets_to_connections,
    print_crosspoints,
};

fn main() {
    let nets: Vec<Net<Node>> = args()
        .skip(1)
        .enumerate()
        .map(|(i, arg)| Net {
            id: (i as u8 + 1).into(),
            nodes: arg
                .split(",")
                .map(|part| part.parse().expect("invalid node"))
                .collect(),
        })
        .collect();

    if nets.len() == 0 {
        eprintln!("Usage: nets-to-chips <net1> [<net2> ...]");
        exit(-1);
    }

    let board = init_board();

    let mut chip_status = ChipStatus::default();

    nets_to_connections(nets.iter(), &mut chip_status, &board).unwrap();

    print_crosspoints(chip_status.crosspoints());

    let crosspoint_config: CrosspointConfig = chip_status.crosspoints().collect();

    println!(
        "{}",
        core::str::from_utf8(&crosspoint_config.to_hex_bytes()).unwrap()
    );
}
