use crate::nets::Nets;
use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::pio::{
    Common, Config, FifoJoin, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::{clocks, into_ref, Peripheral, PeripheralRef};
use embassy_time::{Duration, Timer};
use fixed::types::U24F8;
use fixed_macro::fixed;
use micromath::F32Ext;

const DEFAULTBRIGHTNESS: i32 = 32;

/// In-memory buffer for LED state
///
/// The LED colors are held in an in-memory buffer. The buffer can be manipulated
/// by calling [`set_rgb8`], [`set_rgb`] or [`set_hsv`], and then written to the
/// LEDs all at once by calling [`flush`].
///
pub struct Leds<'d, P: Instance, const S: usize, const N: usize> {
    ws2812: Ws2812<'d, P, S, N>,
    words: [u32; N],
}

impl<'d, P: Instance, const S: usize, const N: usize> Leds<'d, P, S, N> {
    pub fn new(ws2812: Ws2812<'d, P, S, N>) -> Self {
        Self {
            ws2812,
            words: [0; N],
        }
    }

    /// Set LED at given index to color in RGB colorspace
    ///
    /// This version receives 8 bit color components, and is the most direct / fastest way to set a color.
    pub fn set_rgb8(&mut self, i: usize, (r, g, b): (u8, u8, u8)) {
        assert!(i < N);
        self.words[i] = (g as u32) << 24 | (r as u32) << 16 | (b as u32) << 8;
    }

    /// Set LED at given index to color in RGB colorspace
    ///
    /// `r`, `g` and `b` should be in the range from `0.0` to `1.0`.
    pub fn set_rgb(&mut self, i: usize, (r, g, b): (f32, f32, f32)) {
        assert!(i < N);
        self.set_rgb8(i, ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8));
    }

    /// Set LED at given index to color in HSV colorspace
    ///
    /// `h`, `s` and `v` should be in the range from `0.0` to `1.0`.
    ///
    /// Converts to RGB internally. This is presumably the slowest way to set a color.
    pub fn set_hsv(&mut self, i: usize, (h, s, v): (f32, f32, f32)) {
        assert!(i < N);
        let (r, g, b) = hsv_to_rgb((h, s, v));
        self.set_rgb(i, (r, g, b));
    }

    /// Update LEDs with change made by the `set_*` methods.
    pub async fn flush(&mut self) {
        self.ws2812.write_raw(&self.words).await;
    }

    /// Turn off all LEDs (set all colors to 0 and flush)
    pub async fn off(&mut self) {
        self.words.fill(0);
        self.flush().await;
    }

    /// Play startup animation
    ///
    /// Turns all LEDs off when done.
    pub async fn startup_colors(&mut self) {
        let mut offset = 1;
        let mut fade;
        let mut done = false;
        for j in (4..162).step_by(2) {
            if j < DEFAULTBRIGHTNESS / 3 {
                fade = j * 3;
            } else {
                let mut fadeout = j - DEFAULTBRIGHTNESS;
                if fadeout < 0 {
                    fadeout = 0;
                }
                if fadeout > DEFAULTBRIGHTNESS {
                    fadeout = DEFAULTBRIGHTNESS;
                    done = true;
                }
                fade = DEFAULTBRIGHTNESS - fadeout;
            }

            for i in 0..N {
                let led_index = (i + offset) % N;
                let mut h = (i as f32 * j as f32 * 0.1) / 255.0;
                let s = 0.99;
                let v = if led_index == 110 {
                    h = (189 + j) as f32 / 255.0;
                    0.33
                } else {
                    fade as f32 / 255.0
                };
                self.set_hsv(led_index, (h, s, v));
            }
            self.flush().await;

            offset += 1;

            if done {
                break;
            } else {
                Timer::after_millis(14).await;
            }
        }
        Timer::after_millis(2).await;
        self.off().await;
    }

    /// Play "rainbow bounce" animation
    ///
    /// Turns all LEDs off when done.
    pub async fn rainbow_bounce(&mut self, wait: Duration) {
        for j in (0..40).chain((0..40).rev()) {
            for i in 0..N {
                self.set_hsv(i, ((i as f32 / j as f32).sin(), 0.99, 0.1));
            }
            self.flush().await;
            Timer::after(wait + Duration::from_millis(j / 20)).await;
        }

        Timer::after_millis(2).await;
        self.off().await;
    }

    /// Update LEDs to reflect the given nets
    ///
    /// Flushes changes to the board when done.
    ///
    /// In detail, this:
    /// - lights up the breadboard & nano nodes corresponding to each net
    /// - lights up the rails (respecting `nets.supply_switch_pos`)
    /// - adds headerglow to the unused nano LEDs
    /// - turns off all other LEDs
    pub async fn update_from_nets(&mut self, nets: &Nets) {
        self.words.fill(0);

        for i in 80..=109 {
            self.set_rgb8(i, (0x02, 0x00, 0x08)); // headerglow
        }

        for net in &nets.nets {
            for node in net.nodes.iter() {
                if let Some(pixel) = node.pixel() {
                    self.set_rgb8(pixel as usize, nets.color(net.id));
                }
            }
        }
        let gnd = nets.color(nets.nets[0].id);
        let v5 = nets.color(nets.nets[1].id);
        let v33 = nets.color(nets.nets[2].id);

        self.set_rgb8(83, gnd);
        self.set_rgb8(108, gnd);
        self.set_rgb8(109, v5);
        self.set_rgb8(96, v33);
        self.set_rgb8(106, v5);

        for i in [
            71, 72, 75, 76, 79, // top negative rail
            68, 67, 64, 63, 60, // bottom negative rail
        ] {
            self.set_rgb8(i, gnd);
        }

        let v8p = (0x30, 0x1A, 0x02);
        let v8n = (0x12, 0x09, 0x32);

        let (top, bottom) = match nets.supply_switch_pos {
            crate::nets::SupplySwitchPos::_3V3 => (v33, v33),
            crate::nets::SupplySwitchPos::_5V => (v5, v5),
            crate::nets::SupplySwitchPos::_8V => (v8p, v8n),
        };

        for i in [70, 73, 74, 77, 78] {
            self.set_rgb8(i, top);
        }

        for i in [69, 66, 65, 62, 61] {
            self.set_rgb8(i, bottom);
        }

        self.flush().await;
    }
}

/// Controls WS2812 LEDs, using RP2040's PIO.
///
/// A lot of this code is copied from the pio_ws2812 example, found in the embassy repository.
pub struct Ws2812<'d, P: Instance, const S: usize, const N: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize, const N: usize> Ws2812<'d, P, S, N> {
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        pin: impl PioPin,
    ) -> Self {
        into_ref!(dma);

        // prepare the PIO program
        let side_set = pio::SideSet::new(false, 1, false);
        let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

        const T1: u8 = 2; // start bit
        const T2: u8 = 5; // data bit
        const T3: u8 = 3; // stop bit
        const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

        let mut wrap_target = a.label();
        let mut wrap_source = a.label();
        let mut do_zero = a.label();
        a.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);
        a.bind(&mut wrap_target);
        // Do stop bit
        a.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
        // Do start bit
        a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
        // Do data bit = 1
        a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
        a.bind(&mut do_zero);
        // Do data bit = 0
        a.nop_with_delay_and_side_set(T2 - 1, 0);
        a.bind(&mut wrap_source);

        let prg = a.assemble_with_wrap(wrap_source, wrap_target);
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);

        cfg.use_program(&pio.load_program(&prg), &[&out_pin]);

        // Clock config, measured in kHz to avoid overflows
        // TODO CLOCK_FREQ should come from embassy_rp
        let clock_freq = U24F8::from_num(clocks::clk_sys_freq() / 1000);
        let ws2812_freq = fixed!(800: U24F8);
        let bit_freq = ws2812_freq * CYCLES_PER_BIT;
        cfg.clock_divider = clock_freq / bit_freq;

        // FIFO config
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 24,
            direction: ShiftDirection::Left,
        };

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.map_into(),
            sm,
        }
    }

    pub async fn write_raw(&mut self, words: &[u32; N]) {
        self.sm.tx().dma_push(self.dma.reborrow(), words).await;
        Timer::after_micros(55).await;
    }
}

pub fn hsv_to_rgb(c: (f32, f32, f32)) -> (f32, f32, f32) {
    let p = (
        ((c.0 + 1.0).fract() * 6.0 - 3.0).abs(),
        ((c.0 + 2.0 / 3.0).fract() * 6.0 - 3.0).abs(),
        ((c.0 + 1.0 / 3.0).fract() * 6.0 - 3.0).abs(),
    );
    (
        c.2 * mix(1.0, (p.0 - 1.0).clamp(0.0, 1.0), c.1),
        c.2 * mix(1.0, (p.1 - 1.0).clamp(0.0, 1.0), c.1),
        c.2 * mix(1.0, (p.2 - 1.0).clamp(0.0, 1.0), c.1),
    )
}

pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
}
