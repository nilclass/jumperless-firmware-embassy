#![no_std]

use heapless::Vec;

pub struct LineBuffer<const SIZE: usize> {
    buffer: Vec<u8, SIZE>,
    cursor_offset: usize,
}

impl<const SIZE: usize> LineBuffer<SIZE> {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor_offset: 0,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.cursor_offset = 0;
    }

    /// See current buffer contents
    pub fn content(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns cursor position (from the left)
    pub fn cursor(&self) -> usize {
        self.buffer.len() - self.cursor_offset
    }

    /// Insert given character at current position
    ///
    /// Fails if the buffer is full.
    pub fn insert(&mut self, c: u8) -> Result<(), ()> {
        self.buffer.insert(self.cursor(), c).map_err(|_| ())
    }

    /// Deletes character left of the cursor
    ///
    /// Ignored, if the cursor is at the very left
    pub fn backspace(&mut self) {
        let c = self.cursor();
        if c > 0 {
            self.buffer.remove(c - 1);
            self.fix_cursor();
        }
    }

    /// Move the cursor to the left
    pub fn move_left(&mut self) {
        self.cursor_offset = (self.cursor_offset + 1).min(self.buffer.len());
    }

    /// Move the cursor to the right
    pub fn move_right(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
        }
    }

    /// Move cursor to the beginning of the line
    pub fn move_home(&mut self) {
        self.cursor_offset = self.buffer.len();
    }

    /// Move cursor to the end of the line
    pub fn move_end(&mut self) {
        self.cursor_offset = 0;
    }

    fn fix_cursor(&mut self) {
        if self.cursor_offset > self.buffer.len() {
            self.cursor_offset = self.buffer.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_buffer() {
        let buf: LineBuffer<64> = LineBuffer::new();
        assert_eq!(buf.cursor(), 0);
        assert_eq!(buf.content().len(), 0);
    }

    #[test]
    fn test_simple_input() {
        let mut buf: LineBuffer<64> = LineBuffer::new();

        buf.insert(b'h').unwrap();
        buf.insert(b'e').unwrap();
        buf.insert(b'l').unwrap();
        buf.insert(b'l').unwrap();
        buf.insert(b'o').unwrap();
        assert_eq!(buf.cursor(), 5);
        assert_eq!(buf.content(), b"hello");
    }

    #[test]
    fn test_correction_with_arrow_movement() {
        let mut buf: LineBuffer<64> = LineBuffer::new();

        buf.insert(b'h').unwrap();
        buf.insert(b'u').unwrap();
        buf.insert(b'l').unwrap();
        buf.insert(b'L').unwrap();
        buf.insert(b'o').unwrap();

        // fix 'L' -> 'l'
        buf.move_left();
        buf.backspace();
        buf.insert(b'l').unwrap();

        // fix 'u' -> 'e'
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.backspace();
        buf.insert(b'e').unwrap();

        assert_eq!(buf.cursor(), 2);
        assert_eq!(buf.content(), b"hello");
    }
}
