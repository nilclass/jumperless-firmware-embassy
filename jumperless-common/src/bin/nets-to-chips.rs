use std::{env::args, process::exit};

use jumperless_common::{
    ChipStatus,
    CrosspointConfig,
    layout::{Layout, Net},
};

fn main() {
    let nets: Vec<_> = args().skip(1).enumerate().map(|(i, arg)| {
        Net {
            id: (i as u8 + 1).into(),
            nodes: arg.split(",").map(|part| part.parse().expect("invalid node")).collect(),
        }
    }).collect();

    if nets.len() == 0 {
        eprintln!("Usage: nets-to-chips <net1> [<net2> ...]");
        exit(-1);
    }

    let layout = Layout::v4();

    let mut chip_status = ChipStatus::default();

    layout.nets_to_connections(&nets, &mut chip_status).unwrap();

    let crosspoint_config: CrosspointConfig = chip_status.crosspoints().collect();

    println!("{}", core::str::from_utf8(&crosspoint_config.to_hex_bytes()).unwrap());
}
