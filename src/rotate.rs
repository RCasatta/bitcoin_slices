// TODO finish implementation

use core::fmt;
use std::io::{self, Cursor, Read};

use crate::{bsl::BlockHeader, EmptyVisitor, Visitor};

struct RotateBuffer {
    data: Vec<u8>,
    start: usize,
    end: usize,
    min_available: usize,
    read_chunk_size: usize,
    read_finish: bool,
}

impl fmt::Debug for RotateBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("RotateBuffer")
            .field("data.len()", &self.data.len())
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
            data: vec![0u8; buffer_size],
            start: 0,
            end: 0,
            min_available,
            read_chunk_size,
            read_finish: false,
        }
    }
    fn produce<R: Read>(&mut self, read: &mut R) -> Result<(), io::Error> {
        let desired_end = self.end + self.read_chunk_size;
        let final_end = self.data.len().min(desired_end);
        if self.end < self.data.len() {
            let bytes_read = read.read(&mut self.data[self.end..final_end])?;
            if bytes_read == 0 {
                self.read_finish = true;
            }
            self.end += bytes_read;
            print!("produced {} bytes. ", bytes_read);
        } else if self.len() < self.start {
            self.data.copy_within(self.start..self.end, 0);
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

    fn consume_header<V: Visitor>(&mut self, visit: &mut V) -> bool {
        // TODO genericize with Visit trait
        if self.len() < self.min_available {
            false
        } else {
            let header = BlockHeader::visit(&self.data[self.start..self.end], visit).unwrap();
            self.start += header.consumed;

            println!("consumed {} bytes {:?}", header.consumed, self);
            true
        }
    }

    pub fn read_and_visit<R: Read, V: Visitor>(&mut self, read: &mut R, visit: &mut V) {
        loop {
            self.produce(read).unwrap();
            while self.consume_header(visit) {} // consume the more we can to minimize rotating back
            if self.read_finish {
                break;
            }
        }
    }
}

trait Consume {
    fn consume(slice: &[u8]) -> usize;
}

#[test]
fn rotate_buffer() {
    let read = vec![0u8; 2000];
    let mut cursor = Cursor::new(read);
    let mut rot = RotateBuffer::new(1_000, 80, 200);

    rot.read_and_visit(&mut cursor, &mut EmptyVisitor {});
}
