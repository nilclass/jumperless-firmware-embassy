use crate::{bus, nets::Nets, NETS, ch446q::Ch446q, task::leds};
use embassy_rp::peripherals::PIO1;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use embassy_time::Timer;
use jumperless_common::{
    nets_to_connections,
    board::{init_board, Board, Node},
    ChipStatus,
};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

static CHANNEL: bus::Channel<Message> = Channel::new();

pub enum Message {
    Reset,
    AddBridge(Node, Node),
}

impl bus::BusMessage for Message {
    fn sender<'a>() -> Sender<'a, ThreadModeRawMutex, Self, { bus::CHANNEL_SIZE }> {
        CHANNEL.sender()
    }
}

#[embassy_executor::task]
pub async fn main(mut chips: Ch446q<'static, PIO1, 0>) {
    let board = init_board();
    let mut chip_status = ChipStatus::default();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
    loop {
        match CHANNEL.receive().await {
            Message::Reset => {
                if let Some(nets) = NETS.lock().await.as_mut() {
                    *nets = Nets::default();
                    update_chips(nets, &mut chip_status, &mut chips, &board).await;
                }
            }
            Message::AddBridge(a, b) => {
                if let Some(nets) = NETS.lock().await.as_mut() {
                    add_bridge(nets, a, b, &mut rng);
                    update_chips(nets, &mut chip_status, &mut chips, &board).await;
                }
            }
        }
    }
}

fn add_bridge(nets: &mut Nets, a: Node, b: Node, rng: &mut SmallRng) {
    let net_a = nets.with_node(a);
    let net_b = nets.with_node(b);
    match (net_a, net_b) {
        (Some(net_a), Some(net_b)) => {
            if net_a.is_special() && net_b.is_special() {
                defmt::error!("Cannot merge special nets");
            } else if net_b.is_special() {
                nets.merge(net_b, net_a);
            } else {
                nets.merge(net_a, net_b);
            }
        }
        (Some(net_a), None) => {
            _ = nets.add_node(net_a, b);
        },
        (None, Some(net_b)) => {
            _ = nets.add_node(net_b, a);
        },
        (None, None) => {
            let net_id = nets.new_net(random_color(rng));
            _ = nets.add_node(net_id, a);
            _ = nets.add_node(net_id, b);
        }
    }
}

async fn update_chips(nets: &Nets, chip_status: &mut ChipStatus, chips: &mut Ch446q<'static, PIO1, 0>, board: &Board) {
    defmt::info!("Nets changed, recomputing connections");
    chip_status.clear();
    match nets_to_connections(nets.nets.iter(), chip_status, &board) {
        Ok(_) => {
            defmt::info!("Connections computed");
            let mut current_chip = None;
            chips.reset().await;
            for crosspoint in chip_status.crosspoints() {
                if current_chip.is_none() || current_chip.unwrap() != crosspoint.chip {
                    current_chip = Some(crosspoint.chip);
                    chips.set_chip(crosspoint.chip);
                }
                // defmt::debug!("Set {}/{}/{}", crosspoint.chip.index(), crosspoint.x, crosspoint.y);
                chips.write(crosspoint.into()).await;
                Timer::after_micros(100).await;
            }
        },
        Err(_err) => {
            defmt::error!("Failed to compute connections");
        }
    }

    bus::inject(leds::Message::UpdateFromNets).await;
}

/// Pick a random color, for a net
///
/// Port of the `randomColor` function from jumperlab, which was originally written by Kevin Santo Cappuccio in 2024.
/// Except for this function description, all comments are copied from the JavaScript version.
fn random_color(rng: &mut SmallRng) -> (u8, u8, u8) {
    let (mut r, mut g, mut b): (u8, u8, u8) = (rng.gen(), rng.gen(), rng.gen());

    // this picks a random channel to (not quite) zero out, so we get more saturated colors
    match rng.gen_range(0..3) {
        0 => r /= 4,
        1 => g /= 4,
        2 => b /= 4,
        _ => unreachable!(),
    }

    let max = r.max(g).max(b);

    if max < 0xBB { // if it's a dark color, make it brighter
        if r > (max - 0x55) { // this value kinda determines the likelihoood of getting secondary colors, so it's tuned to be roughly 50/50 primary and secondaries (rgb are primaries in this case)
            _ = r.saturating_mul(3);
        }
        if g > (max - 0x55) {
            _ = g.saturating_mul(3);
        }
        if b > (max - 0x55) {
            _ = b.saturating_mul(3);
        }
    }

    let max = r.max(g).max(b);

    if max < 0xAA { // even with multiplying by 3, it's still dark, so we need to make it brighter
        if r == max {
            _ = r.saturating_mul(4);
        }
        if g == max {
            _ = g.saturating_mul(4);
        }
        if b == max {
            _ = g.saturating_mul(4);
        }
    }

    (r, g, b)
}
