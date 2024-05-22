use embassy_rp::{
    dma::{AnyChannel, Channel},
    gpio::{Drive, Output},
    into_ref,
    pio::{
        Common, Config, Direction, FifoJoin, Instance, Pin, PioPin, ShiftConfig, ShiftDirection,
        StateMachine,
    },
    Peripheral, PeripheralRef,
};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use pio::{InstructionOperands, SetDestination};
use jumperless_common::{types::ChipId, Crosspoint};

pub struct Ch446q<'d, P: Instance, const S: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
    config: Config<'d, P>,
    cs_pins: [Pin<'d, P>; 12],
    reset: Output<'d>,
}

impl<'d, P: Instance, const S: usize> Ch446q<'d, P, S> {
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        data_pin: impl PioPin,
        clock_pin: impl PioPin,
        mut reset: Output<'d>,
        mut cs_pins: [Pin<'d, P>; 12],
    ) -> Self {
        into_ref!(dma);

        let program = pio_proc::pio_asm!(
            r#"
            .side_set 1
            .wrap_target
            bitloop:
              // Shift out to DATA, toggling CLK via side-set:
              out pins, 1        side 0x0 [2]
              nop                side 0x1 [2]
              jmp x-- bitloop    side 0x1
              out pins, 1        side 0x1
              mov x, y           side 0x1
              // Pulse CS_x line when done:
              set pins 1         side 0x1 [3]
              set pins 0         side 0x1
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

        for pin in &mut cs_pins {
            pin.set_drive_strength(Drive::_8mA);
        }

        cfg.use_program(&pio.load_program(&program.program), &[&clock_pin]);
        cfg.set_out_pins(&[&data_pin]);
        cfg.set_set_pins(&[&cs_pins[0]]);

        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            threshold: 8,
            direction: ShiftDirection::Left,
            auto_fill: true,
        };
        cfg.clock_divider = 16f32.to_fixed();

        sm.set_config(&cfg);
        sm.set_pin_dirs(Direction::Out, &[&data_pin, &clock_pin]);
        for pin in &cs_pins {
            sm.set_pin_dirs(Direction::Out, &[pin]);
        }

        unsafe {
            sm.exec_instr(
                InstructionOperands::SET {
                    destination: SetDestination::X,
                    data: 6,
                }
                .encode(),
            );
            sm.exec_instr(
                InstructionOperands::SET {
                    destination: SetDestination::Y,
                    data: 6,
                }
                .encode(),
            );
        }

        sm.set_enable(true);

        reset.set_drive_strength(Drive::_12mA);

        Self {
            config: cfg,
            dma: dma.map_into(),
            sm,
            cs_pins,
            reset,
        }
    }

    pub async fn reset(&mut self) {
        self.reset.set_high();
        Timer::after_millis(3).await;
        self.reset.set_low();
    }

    pub fn set_chip(&mut self, chip: ChipId) {
        // wait for TX queue to empty
        while !self.sm.tx().empty() {}
        // disable state machine, while modifying config
        self.sm.set_enable(false);
        let pin = &self.cs_pins[chip.index()];
        // use correct CS pin for SET instructions
        self.config.set_set_pins(&[pin]);
        // apply configuration
        self.sm.set_config(&self.config);
        // re-enable state machine
        self.sm.set_enable(true);
    }

    pub async fn write_raw_path(&mut self, path: &[u32]) {
        self.sm.tx().dma_push(self.dma.reborrow(), path).await;
    }

    pub async fn write(&mut self, packet: Packet) {
        self.sm.tx().wait_push(packet.into()).await;
    }
}

pub struct Packet(u8);

impl Packet {
    pub fn new(x: u8, y: u8, connect: bool) -> Self {
        Self((x << 1) | (y << 5) | if connect { 1 } else { 0 })
    }
}

impl From<Packet> for u32 {
    fn from(val: Packet) -> Self {
        (val.0 as u32) << 24
    }
}

impl From<Crosspoint> for Packet {
    fn from(val: Crosspoint) -> Self {
        Self::new(val.x, val.y, true)
    }
}

pub struct ChipDumpParser<'a> {
    dump: &'a [u8],
    x: u8,
    y: u8,
}

impl<'a> ChipDumpParser<'a> {
    pub fn new(dump: &'a [u8]) -> Self {
        Self {
            dump,
            x: 0,
            y: 0,
        }
    }
}

impl<'a> Iterator for ChipDumpParser<'a> {
    type Item = Packet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.x == 16 {
            return None
        }

        let byte = u32::from_str_radix(core::str::from_utf8(&self.dump[(self.x as usize)..(self.x as usize + 2)]).unwrap(), 16).unwrap();

        let packet = Packet::new(self.x, self.y, ((byte >> self.y) | 1) == 1);
        if self.y < 7 {
            self.y += 1;
        } else {
            self.y = 0;
            self.x += 1;
        }
        Some(packet)
    }
}
