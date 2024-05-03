use smart_leds::RGB8;
use embassy_rp::pio::{
    Common, Config, FifoJoin, Instance, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_time::Timer;
use embassy_rp::{clocks, into_ref, Peripheral, PeripheralRef};
use fixed_macro::fixed;
use fixed::types::U24F8;
use micromath::F32Ext;
use embassy_rp::dma::{AnyChannel, Channel};


/// Overall brightness of LEDs.
const BRIGHTNESS: f32 = 0.3;

const DEFAULTBRIGHTNESS: i32 = 32;

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

        // Setup sm0

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

    pub async fn write(&mut self, colors: &[RGB8; N]) {
        // Precompute the word bytes from the colors
        let mut words = [0u32; N];
        for i in 0..N {
            let word = (u32::from(colors[i].g) << 24) | (u32::from(colors[i].r) << 16) | (u32::from(colors[i].b) << 8);
            words[i] = word;
        }

        // DMA transfer
        self.sm.tx().dma_push(self.dma.reborrow(), &words).await;

        Timer::after_micros(55).await;
    }

    pub async fn rainbow_bounce(&mut self) {
        let mut data = [RGB8::default(); N];
        for j in (0..40).into_iter().chain((0..40).into_iter().rev()) {
            for (i, color) in data.iter_mut().enumerate() {
                *color = rainbow_bounce_color(i as f32, j as f32);
            }
            self.write(&data).await;
            Timer::after_millis(40 * j / 20).await;
        }
    }

    pub async fn off(&mut self) {
        self.write(&[RGB8::new(0, 0, 0); N]).await;
    }

    pub async fn on(&mut self) {
        self.write(&[RGB8::new(80, 80, 80); N]).await;
    }

    pub async fn startup_colors(&mut self) {
        let mut offset = 1;
        let mut fade;
        let mut done = false;

        let mut data = [RGB8::default(); N];
        
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

            for (i, color) in data.iter_mut().enumerate() {
                let mut h = (i as f32 * j as f32 * 0.1) / 255.0;
                let s = 0.99;
                let v = if (i + offset) % N == 110 {
                    h = (189 + j) as f32 / 255.0;
                    0.33
                } else {
                    fade as f32 / 255.0
                };
                let (r, g, b) = hsv_to_rgb((h, s, v));
                *color = RGB8::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            }
            self.write(&data).await;

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
}

fn rainbow_bounce_color(i: f32, j: f32) -> RGB8 {
    let (h, s, v) = ((i / j).sin(), 0.99, 0.1);
    let (r, g, b) = hsv_to_rgb((h, s, v));
    RGB8::new((BRIGHTNESS * r * 255.0) as u8, (BRIGHTNESS * g * 255.0) as u8, (BRIGHTNESS * b * 255.0) as u8)
}

pub fn hsv_to_rgb(c: (f32, f32, f32)) -> (f32, f32, f32) {
    let p = (
        ((c.0 + 1.0).fract() * 6.0 - 3.0).abs(),
        ((c.0 + 2.0 / 3.0).fract() * 6.0 - 3.0).abs(),
        ((c.0 + 1.0 / 3.0).fract() * 6.0 - 3.0).abs(),
    );
    return (
        c.2 * mix(1.0, (p.0 - 1.0).clamp(0.0, 1.0), c.1),
        c.2 * mix(1.0, (p.1 - 1.0).clamp(0.0, 1.0), c.1),
        c.2 * mix(1.0, (p.2 - 1.0).clamp(0.0, 1.0), c.1),
    );
}

pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1. - t) + b * t
}
