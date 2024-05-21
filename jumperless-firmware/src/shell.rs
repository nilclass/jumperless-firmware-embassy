use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_time::Duration;
// use heapless::Vec;
use embassy_usb::{class::cdc_acm::CdcAcmClass, driver::EndpointError};
use line_buffer::LineBuffer;
use jumperless_common::layout::Node;

use crate::nets::SupplySwitchPos;
use crate::task::net_manager;
use crate::{bus, task};

enum Instruction {
    Help,
    Reset,
    RainbowBounce,
    SetSwitchPos(SupplySwitchPos),
    PrintSwitchPos,
    Clear,
    AddBridge(Node, Node),
}

impl Instruction {
    fn parse(input: &str) -> Result<Option<Instruction>, &'static [u8]> {
        let mut tokens = input.trim().split_ascii_whitespace();
        if let Some(token) = tokens.next() {
            match token {
                "help" => {
                    no_more_args(&mut tokens)?;
                    Ok(Some(Instruction::Help))
                }
                "reset" => {
                    no_more_args(&mut tokens)?;
                    Ok(Some(Instruction::Reset))
                }
                "rainbow-bounce" => {
                    no_more_args(&mut tokens)?;
                    Ok(Some(Instruction::RainbowBounce))
                }
                "switch-pos" => {
                    if let Some(pos) = tokens.next() {
                        no_more_args(&mut tokens)?;
                        if let Some(pos) = SupplySwitchPos::parse(pos) {
                            Ok(Some(Instruction::SetSwitchPos(pos)))
                        } else {
                            Err(b"Error: invalid argument\r\n")
                        }
                    } else {
                        Ok(Some(Instruction::PrintSwitchPos))
                    }
                }
                "clear" => {
                    no_more_args(&mut tokens)?;
                    Ok(Some(Instruction::Clear))
                }
                "add-bridge" => {
                    let a = shift_arg(&mut tokens)?;
                    let b = shift_arg(&mut tokens)?;
                    no_more_args(&mut tokens)?;
                    if let Ok(a) = a.parse::<Node>() {
                        if let Ok(b) = b.parse::<Node>() {
                            Ok(Some(Instruction::AddBridge(a, b)))
                        } else {
                            Err(b"Error: invalid second node\r\n")
                        }
                    } else {
                        Err(b"Error: invalid  irstnode\r\n")
                    }
                }
                // "chipdump" => {
                //     no_more_args(&mut tokens)?;
                // }
                _ => Err(b"Error: no such instruction\r\n"),
            }
        } else {
            Ok(None)
        }
    }
}

pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => defmt::panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

pub struct Overflow;

impl From<Overflow> for &'static [u8] {
    fn from(_: Overflow) -> Self {
        "Input buffer overflow".as_bytes()
    }
}

pub struct Shell<'a, 'b, const BUF_SIZE: usize> {
    class: &'a mut CdcAcmClass<'b, Driver<'b, USB>>,
    buffer: LineBuffer<BUF_SIZE>,
}

const HELP: &[&[u8]] = &[
    b"Available instructions:\r\n",
    b"  help                      Print this help text\r\n",
    b"  reset                     Reset (reboot) the device\r\n",
    b"  rainbow-bounce            Play rainbow animation\r\n",
    b"  switch-pos [<5V|3V3|8V>]  Get/set switch position\r\n",
    b"  clear                     Clear all connections\r\n",
    b"  add-bridge <node> <node>  Connect two nodes\r\n",
];

impl<'a, 'b, const BUF_SIZE: usize> Shell<'a, 'b, BUF_SIZE> {
    pub fn new(class: &'a mut CdcAcmClass<'b, Driver<'b, USB>>) -> Self {
        Self {
            class,
            buffer: LineBuffer::new(),
        }
    }

    /// Run the shell, until the connection is terminated
    ///
    /// Reads input, filling the input buffer, then parses
    /// and executes instructions when ENTER ('\r') is pressed.
    pub async fn run(&mut self) -> Result<(), Disconnected> {
        let mut buf = [0; 64];
        // did we see an escape byte?
        let mut escape = false;
        // was a control sequence introduced?
        let mut csi = false;
        loop {
            let n = self.class.read_packet(&mut buf).await?;

            let mut submit = false;

            for &c in &buf[..n] {
                if csi {
                    match c {
                        b'C' => { // RIGHT
                            self.buffer.move_right();
                        }
                        b'D' => { // LEFT
                            self.buffer.move_left();
                        }
                        b'F' => { // END
                            self.buffer.move_end();
                        }
                        b'H' => { // HOME
                            self.buffer.move_home();
                        }
                        _ => {
                            defmt::debug!("Unhandled CSI: {}", c);
                        }
                    }
                    csi = false;
                } else if escape {
                    if c == b'[' {
                        csi = true;
                    } else {
                        defmt::debug!("Unhandled escape: {}", c);
                    }
                    escape = false;
                } else {
                    if c == b'\r' { // ENTER
                        submit = true;
                    } else if c == 27 { // ESC
                        escape = true;
                    } else if c == 3 { // Ctrl+C
                        self.buffer.reset();
                        self.class.write_packet(b"\r\n^C\r\n").await?;
                    } else if c == 127 { // BACKSPACE
                        self.buffer.backspace();
                    } else if c.is_ascii_graphic() || c == b' ' {
                        if let Err(_) = self.buffer.insert(c) {
                            self.buffer.reset();
                            self.class.write_packet(b"\r\n -- overflow; buffer cleared --\r\n").await?;
                        }
                    } else if c.is_ascii_control() {
                        defmt::debug!("Unhandled control character: {}", c);
                    };
                }
            }

            self.prompt().await?;

            if submit {
                self.class.write_packet(b"\r\n").await?;
                self.process().await?;
            }
        }
    }

    /// (Re-) print the prompt, including the input buffer
    async fn prompt(&mut self) -> Result<(), Disconnected> {
        // print prompt at beginning of line
        self.class.write_packet(b"\r> ").await?;

        // print current input buffer
        let line = self.buffer.content();
        if line.len() > 0 {
            self.class
                .write_packet(line)
                .await?;
        }
        // overwrite one more character after the input, to deal with backspace
        self.class.write_packet(&[b' ', 8]).await?;

        // move cursor to correct position
        let cursor = self.buffer.cursor() + 2;
        self.class.write_packet(&[b'\r']).await?;
        for i in 0..cursor {
            self.class.write_packet(&[27, b'[', b'C']).await?;
        }
        Ok(())
    }

    /// Process the input buffer, and execute any instruction found
    async fn process(&mut self) -> Result<(), Disconnected> {
        let buffer = self.buffer.content();
        if buffer.is_empty() {
            self.prompt().await?;
            return Ok(());
        }
        if let Ok(input) = core::str::from_utf8(buffer) {
            match Instruction::parse(input) {
                Ok(Some(instruction)) => self.execute(instruction).await?,
                Ok(None) => {}
                Err(message) => self.class.write_packet(message).await?,
            }
        }
        self.buffer.reset();
        self.prompt().await?;
        Ok(())
    }

    /// Execute an instruction
    async fn execute(&mut self, instruction: Instruction) -> Result<(), Disconnected> {
        match instruction {
            Instruction::Help => {
                for line in HELP {
                    self.class.write_packet(line).await?;
                }
                Ok(())
            }
            Instruction::Reset => {
                bus::inject(task::watchdog::Message::Reset).await;
                Ok(())
            }
            Instruction::RainbowBounce => {
                bus::inject(task::leds::Message::PlayRainbowBounce).await;
                Ok(())
            }
            Instruction::SetSwitchPos(pos) => {
                if let Some(nets) = crate::NETS.lock().await.as_mut() {
                    nets.supply_switch_pos = pos;
                    bus::inject(task::leds::Message::UpdateFromNets).await;
                }
                Ok(())
            }
            Instruction::PrintSwitchPos => {
                if let Some(nets) = crate::NETS.lock().await.as_ref() {
                    self.class.write_packet(nets.supply_switch_pos.label().as_bytes()).await?;
                    self.class.write_packet(b"\r\n").await?;
                }
                Ok(())
            }
            Instruction::Clear => {
                bus::inject(net_manager::Message::Reset).await;
                Ok(())
            }
            Instruction::AddBridge(a, b) => {
                bus::inject(net_manager::Message::AddBridge(a, b)).await;
                Ok(())
            }
        }
    }
}

fn shift_arg<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Result<&'a str, &'static [u8]> {
    match tokens.next() {
        Some(arg) => Ok(arg),
        None => Err(b"Error: missing argument\r\n"),
    }
}

fn no_more_args<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Result<(), &'static [u8]> {
    match tokens.next() {
        Some(_) => Err(b"Error: unexpected extra arguments\r\n"),
        None => Ok(()),
    }
}
