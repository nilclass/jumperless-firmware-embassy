//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]
#![feature(async_fn_traits, async_closure)]

use ch446q::{Ch446q, Chip, Packet};
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIO0, PIO1, USB};
use embassy_rp::{pio, usb};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embassy_usb::class::cdc_acm;
use {defmt_rtt as _, panic_probe as _};

mod ch446q;
mod leds;
mod shell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

/// Number of LEDs on the board. This will vary in the future, depending on hardware revision.
const NUM_LEDS: usize = 111;

static LEDS: Mutex<ThreadModeRawMutex, Option<leds::Leds<'static, PIO0, 0, NUM_LEDS>>> =
    Mutex::new(None);

/// One-off task playing the startup LED animation
async fn startup_leds() {
    {
        if let Some(leds) = LEDS.lock().await.as_mut() {
            leds.startup_colors().await;
            Timer::after_millis(2).await;
            leds.set_rgb8(110, (32, 0, 0));
            leds.flush().await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio::Pio {
        mut common, sm0, ..
    } = pio::Pio::new(p.PIO0, Irqs);

    // Initialize shared LEDs
    {
        *(LEDS.lock().await) = Some(leds::Leds::new(leds::Ws2812::new(
            &mut common,
            sm0,
            p.DMA_CH0,
            p.PIN_25,
        )));
    }

    let pio::Pio {
        mut common,
        sm0,
        ..
    } = pio::Pio::new(p.PIO1, Irqs);

    let cs_pins = [
        common.make_pio_pin(p.PIN_6),
        common.make_pio_pin(p.PIN_7),
        common.make_pio_pin(p.PIN_8),
        common.make_pio_pin(p.PIN_9),
        common.make_pio_pin(p.PIN_10),
        common.make_pio_pin(p.PIN_11),
        common.make_pio_pin(p.PIN_12),
        common.make_pio_pin(p.PIN_13),
        common.make_pio_pin(p.PIN_20),
        common.make_pio_pin(p.PIN_21),
        common.make_pio_pin(p.PIN_22),
        common.make_pio_pin(p.PIN_23),
    ];

    let mut ch446q = Ch446q::new(
        &mut common,
        sm0,
        p.DMA_CH1,
        p.PIN_14,
        p.PIN_15,
        Output::new(p.PIN_24, Level::Low), // reset
        cs_pins,
        // Output::new(p.PIN_6, Level::Low),  // cs_a
    );

    ch446q.reset().await;

    // Make one example connection, between breadboard nodes [2] and [3]

    ch446q.set_chip(Chip::A);

    let path_on: [u32; 2] = [
        Packet::new(0, 1, true).into(),
        Packet::new(0, 2, true).into(),
    ];

    ch446q.write_raw_path(&path_on).await;

    let path_off: [u32; 2] = [
        Packet::new(0, 1, false).into(),
        Packet::new(0, 2, false).into(),
    ];

    // Uncomment this to test switching between Chip A and Chip B:
    /*
    loop {
        ch446q.set_chip(Chip::A);

        ch446q.write_raw_path(&path_on).await;

        Timer::after_secs(1).await;

        ch446q.write_raw_path(&path_off).await;

        Timer::after_secs(1).await;

        ch446q.set_chip(Chip::B);

        ch446q.write_raw_path(&path_on).await;

        Timer::after_secs(1).await;

        ch446q.write_raw_path(&path_off).await;

        Timer::after_secs(1).await;
}

    */
    
    // // connect x0 (-> AI) with y1 (-> [2])
    // ch446q.write(Packet::new(0, 1, true)).await;
    // // connect x0 (-> AI) with y2 (-> [3])
    // ch446q.write(Packet::new(0, 2, true)).await;

    // Initialize USB driver
    let usb_driver = usb::Driver::new(p.USB, Irqs);
    let mut config = embassy_usb::Config::new(0x1D50, 0xACAB);
    config.manufacturer = Some("Architeuthis Flux");
    config.product = Some("Jumperless");
    config.serial_number = Some("0");
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = cdc_acm::State::new();

    let mut builder = embassy_usb::Builder::new(
        usb_driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut class = cdc_acm::CdcAcmClass::new(&mut builder, &mut state, 64);

    let mut usb = builder.build();

    let usb_future = usb.run();

    let shell_future = async {
        loop {
            class.wait_connection().await;
            info!("USB Serial Connected");
            let mut shell: shell::Shell<'_, '_, 256> = shell::Shell::new(&mut class);
            let _ = shell.run().await;
            info!("USB Serial Disconnected");
        }
    };

    join(usb_future, join(shell_future, startup_leds())).await;
}
