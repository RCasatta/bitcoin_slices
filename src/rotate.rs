// TODO finish implementation

use std::io::Read;

struct RotateBuffer {
    data: Vec<u8>,
    start: usize,
    end: usize,
}

impl RotateBuffer {
    pub fn new() -> Self {
        Self {
            data: vec![0u8; 200_000],
            start: 0,
            end: 0,
        }
    }
    fn status(&self) {
        println!("start {}, end: {}", self.start, self.end);
    }
    pub fn produce<R: Read>(&mut self, read: &mut R) {
        let desired_end = self.end + 65_536;
        let final_end = self.data.len().min(desired_end);
        let length = self.end - self.start;
        if self.end < self.data.len() {
            let bytes_read = read.read(&mut self.data[self.end..final_end]).unwrap();
            self.end += bytes_read;
            print!("produced {} bytes. ", bytes_read);
        } else if length < self.start {
            let (first, second) = self.data.split_at_mut(self.start);
            let length = self.end - self.start;
            first[..length].copy_from_slice(&second[..length]);
            self.start = 0;
            self.end = length;
            print!("rotating back ");
        } else {
            print!("no space to rotate, consume bytes! ");
        }
        self.status();
    }

    pub fn consume(&mut self) {
        // takes closure
        let desired_start = self.start + 65_536;
        if desired_start < self.end {
            self.start = desired_start;
            print!("consumed 65_536 bytes ");
        } else {
            print!("cannot consume otherwise start overlaps end");
        }
        self.status();
    }
}

#[test]
fn rotate_buffer() {
    let vec = vec![0u8; 200_000];

    let mut rot = RotateBuffer::new();

    rot.produce(&mut &vec[..]);
    rot.produce(&mut &vec[..]);

    rot.consume();
    rot.produce(&mut &vec[..]);
    rot.produce(&mut &vec[..]);
    rot.produce(&mut &vec[..]);

    rot.consume();
    rot.consume();
    rot.produce(&mut &vec[..]);
}
