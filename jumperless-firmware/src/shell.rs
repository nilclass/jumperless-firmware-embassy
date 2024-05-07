use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_time::Duration;
use heapless::Vec;
use embassy_usb::{class::cdc_acm::CdcAcmClass, driver::EndpointError};

use crate::nets::SupplySwitchPos;
use crate::{bus, task};

enum Instruction {
    Help,
    Reset,
    RainbowBounce,
    SwitchPos(SupplySwitchPos),
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
                    let pos = shift_arg(&mut tokens)?;
                    no_more_args(&mut tokens)?;
                    if let Some(pos) = SupplySwitchPos::parse(pos) {
                        Ok(Some(Instruction::SwitchPos(pos)))
                    } else {
                        Err(b"Error: invalid argument\r\n")
                    }
                }
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
    input_buf: Vec<u8, BUF_SIZE>,
    // offset of the cursor from the *end* of the line
    cursor_offset: usize,
}

const HELP: &[&[u8]] = &[
    b"Available instructions:\r\n",
    b"  help                    Print this help text\r\n",
    b"  reset                   Reset (reboot) the device\r\n",
    b"  rainbow-bounce          Play rainbow animation\r\n",
    b"  switch-pos <5V|3V3|8V>  Set switch position\r\n",
];

impl<'a, 'b, const BUF_SIZE: usize> Shell<'a, 'b, BUF_SIZE> {
    pub fn new(class: &'a mut CdcAcmClass<'b, Driver<'b, USB>>) -> Self {
        Self {
            class,
            input_buf: Vec::new(),
            cursor_offset: 0,
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
                            if self.cursor_offset > 0 {
                                self.cursor_offset -= 1;
                            }
                        }
                        b'D' => { // LEFT
                            if self.cursor_offset < self.input_buf.len() {
                                self.cursor_offset += 1;
                            }
                        }
                        b'F' => { // END
                            self.cursor_offset = 0;
                        }
                        b'H' => { // HOME
                            self.cursor_offset = if self.input_buf.len() == 0 { 0 } else { self.input_buf.len() - 1 };
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
                    if c == b'\r' {
                        submit = true;
                    } else if c == 27 { // ESC
                        escape = true;
                    } else if c == 127 {
                        // backspace
                        if self.input_buf.len() - self.cursor_offset > 0 {
                            self.input_buf.remove(self.input_buf.len() - 1 - self.cursor_offset);
                        }
                    } else if c.is_ascii_graphic() || c == b' ' {
                        if let Err(_) = self.input_buf.insert(self.input_buf.len() - self.cursor_offset, c) {
                            self.input_buf.clear();
                            self.class.write_packet(b"\r\n -- overflow; buffer cleared --\r\n").await?;
                        }
                        if self.input_buf.len() == 0 {
                            self.cursor_offset = 0;
                        } else if self.cursor_offset > self.input_buf.len() - 1 {
                            self.cursor_offset -= 1;
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
        self.class.write_packet(b"\r> ").await?;
        if self.input_buf.len() > 0 {
            self.class
                .write_packet(&self.input_buf)
                .await?;
        }
        // overwrite one character, in case it was removed.
        self.class.write_packet(&[b' ', 8]).await?;
        if self.cursor_offset > 0 {
            let cursor_move = [27, b'[', b'0' + self.cursor_offset as u8, b'D'];
            defmt::debug!("Cursor offset {}, move {:?}", self.cursor_offset, cursor_move);
            self.class.write_packet(&cursor_move).await?;
        }
        Ok(())
    }

    /// Process the input buffer, and execute any instruction found
    async fn process(&mut self) -> Result<(), Disconnected> {
        if self.input_buf.is_empty() {
            self.prompt().await?;
            return Ok(());
        }
        if let Ok(input) = core::str::from_utf8(&self.input_buf) {
            match Instruction::parse(input) {
                Ok(Some(instruction)) => self.execute(instruction).await?,
                Ok(None) => {}
                Err(message) => self.class.write_packet(message).await?,
            }
        }
        self.input_buf.clear();
        self.cursor_offset = 0;
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
            Instruction::SwitchPos(pos) => {
                if let Some(nets) = crate::NETS.lock().await.as_mut() {
                    nets.supply_switch_pos = pos;
                    bus::inject(task::leds::Message::UpdateFromNets).await;
                }
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
