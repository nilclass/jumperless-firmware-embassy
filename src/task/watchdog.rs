use embassy_rp::watchdog::Watchdog;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Sender},
};
use embassy_time::{Duration, Timer};

use crate::bus;

static CHANNEL: bus::Channel<Message> = Channel::new();

/// A [`bus::BusMessage`] targeting the `watchdog` task.
pub enum Message {
    /// Trigger a device reset
    Reset,
}

impl bus::BusMessage for Message {
    fn sender<'a>() -> Sender<'a, ThreadModeRawMutex, Self, { bus::CHANNEL_SIZE }> {
        CHANNEL.sender()
    }
}

#[embassy_executor::task]
pub async fn main(mut watchdog: Watchdog) {
    watchdog.start(Duration::from_millis(1500));
    watchdog.pause_on_debug(true);
    loop {
        Timer::after_millis(750).await;
        watchdog.feed();

        if let Ok(message) = CHANNEL.try_receive() {
            match message {
                Message::Reset => watchdog.trigger_reset(),
            }
        }
    }
}
