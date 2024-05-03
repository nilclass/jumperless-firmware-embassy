//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::pio::{
    InterruptHandler, Pio,
};
use embassy_time::Timer;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use {defmt_rtt as _, panic_probe as _};
use defmt::*;

mod leds;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// Number of LEDs on the board. This will vary in the future, depending on hardware revision.
const NUM_LEDS: usize = 111;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut ws2812: leds::Ws2812<'_, _, 0, NUM_LEDS> = leds::Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_25);

    ws2812.startup_colors().await;

    Timer::after_secs(10).await;

    loop {
        ws2812.rainbow_bounce().await;
        info!("Bounce done, setting off");
        Timer::after_millis(1).await;
        ws2812.off().await;
        info!("Waiting...");
        Timer::after_secs(3).await;
        info!("Here we go again!");
    }
}
