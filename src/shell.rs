use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_time::Duration;
use embassy_usb::{class::cdc_acm::CdcAcmClass, driver::EndpointError};

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
    input_buf: [u8; BUF_SIZE],
    cursor: usize,
}

const HELP: &[&[u8]] = &[
    b"Available instructions:\r\n",
    b"  help            Print this help text\r\n",
    b"  rainbow-bounce  Play rainbow animation\r\n",
];

impl<'a, 'b, const BUF_SIZE: usize> Shell<'a, 'b, BUF_SIZE> {
    pub fn new(class: &'a mut CdcAcmClass<'b, Driver<'b, USB>>) -> Self {
        Self {
            class,
            input_buf: [0; BUF_SIZE],
            cursor: 0,
        }
    }

    pub async fn run(&mut self) -> Result<(), Disconnected> {
        let mut buf = [0; 64];
        loop {
            let n = self.class.read_packet(&mut buf).await?;

            let mut submit = false;

            for &c in &buf[..n] {
                if c == b'\r' {
                    submit = true;
                } else if c == 0x08 {
                    // backspace
                    self.cursor -= 1;
                    self.input_buf[self.cursor] = 0;
                } else if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                    self.input_buf[self.cursor] = c;
                    self.cursor += 1;
                }
            }

            self.prompt().await?;

            if submit {
                self.class.write_packet(b"\r\n").await?;
                self.process().await?;
            }
        }
    }

    async fn prompt(&mut self) -> Result<(), Disconnected> {
        self.class.write_packet(b"\r> ").await?;
        if self.cursor > 0 {
            self.class
                .write_packet(&self.input_buf[0..self.cursor])
                .await?;
        }
        Ok(())
    }

    async fn process(&mut self) -> Result<(), Disconnected> {
        if self.cursor == 0 {
            self.prompt().await?;
            return Ok(());
        }
        if let Ok(input) = core::str::from_utf8(&self.input_buf[0..self.cursor]) {
            match input.trim() {
                "help" => {
                    for line in HELP {
                        self.class.write_packet(line).await?;
                    }
                }
                "rainbow-bounce" => {
                    if let Some(leds) = crate::LEDS.lock().await.as_mut() {
                        leds.rainbow_bounce(Duration::from_millis(40)).await;
                    }
                }
                _ => {
                    self.class
                        .write_packet(b"Error: Unknown instruction\r\n")
                        .await?
                }
            }
        }
        self.input_buf.fill(0);
        self.cursor = 0;
        self.prompt().await?;
        Ok(())
    }
}
