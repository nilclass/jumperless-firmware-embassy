use embassy_time::Timer;
use embassy_rp::{
    dma::{AnyChannel, Channel}, gpio::Output, into_ref, pio::{Common, Config, Direction, FifoJoin, Instance, Irq, PioPin, ShiftConfig, ShiftDirection, StateMachine}, Peripheral, PeripheralRef
};
use fixed::traits::ToFixed;
use pio::{InstructionOperands, SetDestination};

pub struct Ch446q<'d, P: Instance, const S: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
    cs_a: Output<'d>,
    reset: Output<'d>,
    irq0: Irq<'d, P, 0>,
}

impl<'d, P: Instance, const S: usize> Ch446q<'d, P, S> {
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        data_pin: impl PioPin,
        clock_pin: impl PioPin,
        mut reset: Output<'d>,
        mut cs_a: Output<'d>,
        irq0: Irq<'d, P, 0>,
    ) -> Self {
        into_ref!(dma);

        let program = pio_proc::pio_asm!(
            r#"
            .side_set 1
            .wrap_target
            bitloop:
              out pins, 1        side 0x0 [2]
              nop                side 0x1 [2]
              jmp x-- bitloop    side 0x1
              out pins, 1        side 0x1
              mov x, y           side 0x1
              irq 0              side 0x1
              wait 0 irq 0 rel   side 0x1
              jmp !osre bitloop  side 0x0
            public entry_point:
              pull ifempty       side 0x0 [1]
              nop                side 0x0 [1]
            .wrap
            "#
        );

        let mut cfg = Config::default();

        let data_pin = pio.make_pio_pin(data_pin);
        let clock_pin = pio.make_pio_pin(clock_pin);

        cfg.use_program(
            &pio.load_program(&program.program),
            &[&clock_pin],
        );
        cfg.set_out_pins(&[&data_pin]);
        cfg.set_set_pins(&[&data_pin]);

        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            threshold: 8,
            direction: ShiftDirection::Left,
            auto_fill: true,
        };
        cfg.clock_divider = 16f32.to_fixed();

        sm.set_config(&cfg);
        sm.set_pin_dirs(
            Direction::Out,
            &[&data_pin, &clock_pin],
        );

        unsafe {
            sm.exec_instr(
                InstructionOperands::SET {
                    destination: SetDestination::X,
                    data: 6,
                }.encode(),
            );
            sm.exec_instr(
                InstructionOperands::SET {
                    destination: SetDestination::Y,
                    data: 6,
                }.encode(),
            );
        }

        sm.set_enable(true);

        cs_a.set_drive_strength(embassy_rp::gpio::Drive::_8mA);
        reset.set_drive_strength(embassy_rp::gpio::Drive::_12mA);

        Self {
            dma: dma.map_into(),
            sm,
            cs_a,
            irq0,
            reset,
        }
    }

    pub async fn reset(&mut self) {
        self.reset.set_high();
        Timer::after_millis(3).await;
        self.reset.set_low();
    }

    pub async fn write(&mut self, packet: Packet) {
        self.sm.tx().wait_push(packet.into()).await;
        self.irq0.wait().await;
        self.cs_a.set_high();
        Timer::after_micros(6).await;
        self.cs_a.set_low();
        Timer::after_micros(6).await;
    }
}

pub struct Packet(u8);

impl Packet {
    pub fn new(x: u8, y: u8, connect: bool) -> Self {
        Self((x << 1) | (y << 5) | if connect { 1 } else { 0 })
    }
}

impl Into<u32> for Packet {
    fn into(self) -> u32 {
        (self.0 as u32) << 24
    }
}
