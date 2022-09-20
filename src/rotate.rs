// TODO finish implementation

use core::fmt;
use std::io::{self, Cursor, Read};

use crate::{bsl::BlockHeader, EmptyVisitor, Visit, Visitor};

struct RotateBuffer {
    buffer: Vec<u8>,
    start: usize,
    end: usize,
    min_available: usize,
    read_chunk_size: usize,
    read_finish: bool,
}

impl fmt::Debug for RotateBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("RotateBuffer")
            .field("buffer.len()", &self.buffer.len())
            .field("start", &self.start)
            .field("end", &self.end)
            .field("min_available", &self.min_available)
            .field("read_chunk_size", &self.read_chunk_size)
            .field("read_finish", &self.read_finish)
            .finish()
    }
}

impl RotateBuffer {
    pub fn new(buffer_size: usize, min_available: usize, read_chunk_size: usize) -> Self {
        Self {
            buffer: vec![0u8; buffer_size],
            start: 0,
            end: 0,
            min_available,
            read_chunk_size,
            read_finish: false,
        }
    }
    fn produce<R: Read>(&mut self, read: &mut R) -> Result<(), io::Error> {
        let desired_end = self.end + self.read_chunk_size;
        let final_end = self.buffer.len().min(desired_end);
        if self.end < self.buffer.len() {
            let bytes_read = read.read(&mut self.buffer[self.end..final_end])?;
            if bytes_read == 0 {
                self.read_finish = true;
            }
            self.end += bytes_read;
            print!("produced {} bytes. ", bytes_read);
        } else if self.len() < self.start {
            self.buffer.copy_within(self.start..self.end, 0);
            self.end = self.len();
            self.start = 0;
            print!("rotating back {} bytes ", self.end);
        } else {
            print!("no space to rotate, consume bytes! ");
        }
        println!("{:?}", self);
        Ok(())
    }

    fn len(&self) -> usize {
        self.end - self.start
    }

    fn consume<'a, P: Visit<'a, P>, V: Visitor>(&'a mut self, visit: &mut V) -> bool {
        if self.len() < self.min_available {
            false
        } else {
            let parsed = P::visit(&self.buffer[self.start..self.end], visit).unwrap();
            self.start += parsed.consumed;

            println!("consumed {} bytes {:?}", parsed.consumed, self);
            true
        }
    }

    pub fn read_and_visit<'a, P: Visit<'a, P>, R: Read, V: Visitor>(
        &mut self,
        read: &mut R,
        visit: &mut V,
    ) {
        loop {
            self.produce(read).unwrap();
            while self.consume::<P, _>(visit) {} // consume the more we can to minimize rotating back
            if self.read_finish {
                break;
            }
        }
    }
}

#[test]
fn rotate_buffer() {
    let read = vec![0u8; 2000];
    let mut cursor = Cursor::new(read);
    let mut rot = RotateBuffer::new(1_000, 80, 200);

    rot.read_and_visit::<BlockHeader, _, _>(&mut cursor, &mut EmptyVisitor {});
}
