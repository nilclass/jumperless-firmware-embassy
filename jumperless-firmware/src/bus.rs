use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Sender};

/// Size of the message channel for each task.
pub const CHANNEL_SIZE: usize = 8;

pub type Channel<T> = embassy_sync::channel::Channel<ThreadModeRawMutex, T, CHANNEL_SIZE>;

/// Implemented by Message types which are defined by a task, to become usable with `inject`.
pub trait BusMessage: Sized {
    fn sender<'a>() -> Sender<'a, ThreadModeRawMutex, Self, CHANNEL_SIZE>;
}

/// Inject a message into the bus, to send it to appropriate task.
///
/// The message will be processed asynchronously (each task has an "inbox", [`CHANNEL_SIZE`] entries long).
///
/// Example:
///     bus::inject(task::leds::PlayRainbowBounce).await;
///
pub async fn inject<'a, T: BusMessage>(message: T) {
    T::sender().send(message).await;
}
