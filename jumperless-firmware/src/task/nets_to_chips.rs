use crate::{bus, NETS, ch446q::Ch446q};
use embassy_rp::peripherals::PIO1;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use jumperless_common::{
    layout::Layout,
    ChipStatus,
};

static CHANNEL: bus::Channel<Message> = Channel::new();

pub enum Message {
    Update,
}

impl bus::BusMessage for Message {
    fn sender<'a>() -> Sender<'a, ThreadModeRawMutex, Self, { bus::CHANNEL_SIZE }> {
        CHANNEL.sender()
    }
}

#[embassy_executor::task]
pub async fn main(mut chips: Ch446q<'static, PIO1, 0>) {
    let layout = Layout::v4();
    let mut chip_status = ChipStatus::default();
    loop {
        match CHANNEL.receive().await {
            Message::Update => {
                defmt::info!("Received update request");
                chip_status.clear();
                if let Some(nets) = NETS.lock().await.as_ref() {
                    match layout.nets_to_connections(&nets.nets, &mut chip_status) {
                        Ok(_) => {
                            defmt::info!("Chip status computed");
                            let mut current_chip = None;
                            chips.reset().await;
                            for crosspoint in chip_status.crosspoints() {
                                if current_chip.is_none() || current_chip.unwrap() != crosspoint.chip {
                                    current_chip = Some(crosspoint.chip);
                                    chips.set_chip(crosspoint.chip);
                                }
                                chips.write(crosspoint.into()).await;
                            }
                        },
                        Err(err) => {
                            defmt::error!("Failed to route nets to chips");
                        }
                    }
                }
            }
        }
    }
}
