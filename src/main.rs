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

// /// Input a value 0 to 255 to get a color value
// /// The colours are a transition r - g - b - back to r.
// fn wheel(mut wheel_pos: u8) -> RGB8 {
//     wheel_pos = 255 - wheel_pos;
//     if wheel_pos < 85 {
//         return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
//     }
//     if wheel_pos < 170 {
//         wheel_pos -= 85;
//         return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
//     }
//     wheel_pos -= 170;
//     let (r, g, b) = (wheel_pos * 3, 255 - wheel_pos * 3, 0);
//     //(r / 8, g / 8, b / 8).into()
//     (r / 10, g / 10, b / 10).into()
// }

// const NUM_LEDS: usize = 111;
// const BRIGHTNESS: f32 = 0.3;

// fn rainbow_bounce_color(i: f32, j: f32) -> RGB8 {
//     let (h, s, v) = ((i / j).sin().into(), 0.99, 0.1);
//     let (r, g, b) = hsv_to_rgb((h, s, v));
//     RGB8::new((BRIGHTNESS * r * 255.0) as u8, (BRIGHTNESS * g * 255.0) as u8, (BRIGHTNESS * b * 255.0) as u8)
// }

// pub fn hsv_to_rgb(c: (f32, f32, f32)) -> (f32, f32, f32) {
//     let p = (
//         ((c.0 + 1.0).fract() * 6.0 - 3.0).abs(),
//         ((c.0 + 2.0 / 3.0).fract() * 6.0 - 3.0).abs(),
//         ((c.0 + 1.0 / 3.0).fract() * 6.0 - 3.0).abs(),
//     );
//     return (
//         c.2 * mix(1.0, (p.0 - 1.0).clamp(0.0, 1.0), c.1),
//         c.2 * mix(1.0, (p.1 - 1.0).clamp(0.0, 1.0), c.1),
//         c.2 * mix(1.0, (p.2 - 1.0).clamp(0.0, 1.0), c.1),
//     );
// }

// pub fn mix(a: f32, b: f32, t: f32) -> f32 {
//     a * (1. - t) + b * t
// }

/// Number of LEDs on the board. This will vary in the future, depending on hardware revision.
const NUM_LEDS: usize = 111;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    // let mut led = Output::new(p.PIN_25, Level::Low);
    
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    // let mut data = [RGB8::default(); NUM_LEDS];

    let mut ws2812: leds::Ws2812<'_, _, 0, NUM_LEDS> = leds::Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_25);

    // let mut ticker = Ticker::every(Duration::from_millis(40));

    // info!("Off for a bit...");
    // ws2812.off().await;
    // Timer::after_secs(2).await;
    // info!("On for a bit...");
    // ws2812.on().await;
    // Timer::after_secs(2).await;
    // info!("Off for a bit...");
    // ws2812.off().await;
    // Timer::after_secs(2).await;

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
        // for j in 0..40 {
        //     for (i, mut color) in data.iter_mut().enumerate() {
        //         *color = rainbow_bounce_color(i as f32, j as f32);
        //     }
        //     ws2812.write(&data).await;
        //     ticker.next().await;
        // }
        // for j in 0..40 {
        //     let j = 40 - j;
        //     for (i, mut color) in data.iter_mut().enumerate() {
        //         *color = rainbow_bounce_color(i as f32, j as f32);
        //     }
        //     ws2812.write(&data).await;
        //     ticker.next().await;
        // }

        // info!("led on!");
        // led.set_high();
        // Timer::after_secs(1).await;

        // info!("led off!");
        // led.set_low();
        // Timer::after_secs(1).await;
    }
}
