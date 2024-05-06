use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::{Channel, Sender}};
use embassy_time::{Timer, Duration};
use crate::{bus, LEDS, NETS};

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
pub async fn main() {
    // Play startup animation
    startup_leds().await;

    // Set up normal state (colors indicate nets)
    update_from_nets().await;

    loop {
        match CHANNEL.receive().await {
            Message::PlayRainbowBounce => {
                if let Some(leds) = LEDS.lock().await.as_mut() {
                    leds.rainbow_bounce(Duration::from_millis(40)).await;
                }
                // restore normal state
                update_from_nets().await;
            }
            Message::UpdateFromNets => {
                update_from_nets().await;
            }
        }
    }
}

async fn startup_leds() {
    if let Some(leds) = LEDS.lock().await.as_mut() {
        leds.startup_colors().await;
        Timer::after_millis(2).await;
        leds.set_rgb8(110, (32, 0, 0));
        leds.flush().await;
    }
}

async fn update_from_nets() {
    Timer::after_millis(2).await;
    if let Some(leds) = LEDS.lock().await.as_mut() {
        if let Some(nets) = NETS.lock().await.as_mut() {
            leds.update_from_nets(&nets).await;
        }
    }
}
