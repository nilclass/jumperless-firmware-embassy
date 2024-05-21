use crate::{bus, NETS};
use embassy_rp::peripherals::PIO0;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use embassy_time::{Duration, Timer};

/// Number of LEDs on the board. This will vary in the future, depending on hardware revision.
const NUM_LEDS: usize = 111;

type Leds = crate::leds::Leds<'static, PIO0, 0, NUM_LEDS>;

static CHANNEL: bus::Channel<Message> = Channel::new();

/// A [`bus::BusMessage`] targeting the `leds` task.
pub enum Message {
    /// Play the "rainbow-bounce" animation, then return to normal state
    PlayRainbowBounce,

    /// Update LEDs to reflect changes to the nets or other state
    UpdateFromNets,
}

impl bus::BusMessage for Message {
    fn sender<'a>() -> Sender<'a, ThreadModeRawMutex, Self, { bus::CHANNEL_SIZE }> {
        CHANNEL.sender()
    }
}

#[embassy_executor::task]
pub async fn main(mut leds: Leds) {
    // Play startup animation
    startup_leds(&mut leds).await;

    // Set up normal state (colors indicate nets)
    update_from_nets(&mut leds).await;

    loop {
        match CHANNEL.receive().await {
            Message::PlayRainbowBounce => {
                leds.rainbow_bounce(Duration::from_millis(40)).await;
                // restore normal state
                update_from_nets(&mut leds).await;
            }
            Message::UpdateFromNets => {
                defmt::debug!("Updating from nets");
                update_from_nets(&mut leds).await;
            }
        }
    }
}

async fn startup_leds(leds: &mut Leds) {
    leds.startup_colors().await;
    Timer::after_millis(2).await;
    leds.set_rgb8(110, (32, 0, 0));
    leds.flush().await;
}

async fn update_from_nets(leds: &mut Leds) {
    Timer::after_millis(2).await;
    if let Some(nets) = NETS.lock().await.as_ref() {
        leds.update_from_nets(nets).await;
    }
}
