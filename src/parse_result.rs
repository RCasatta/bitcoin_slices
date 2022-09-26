/// Every `parse` or `visit` functions on success return this struct.
/// It contains the object parsed `T` the remaining bytes (empty slice if all bytes in the slice are
/// consumed), and the bytes consumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult<'a, T: AsRef<[u8]>> {
    remaining: &'a [u8],
    parsed: T,
}

impl<'a, T: AsRef<[u8]>> ParseResult<'a, T> {
    /// Creates a new ParseResult
    pub fn new(remaining: &'a [u8], parsed: T) -> Self {
        ParseResult { remaining, parsed }
    }
    /// map the `ParseResult` to another type `Y` as specified in the given function.
    pub fn map<Y, O: FnOnce(Self) -> Y>(self, op: O) -> Y {
        op(self)
    }
    /// returns the remaining slice, which is empty if all the bytes in the slice have been used.
    pub fn remaining(&self) -> &'a [u8] {
        self.remaining
    }
    /// returns a reference of the object parsed
    pub fn parsed(&'a self) -> &'a T {
        &self.parsed
    }
    /// returns the object parsed owned
    pub fn parsed_owned(self) -> T {
        self.parsed
    }
    /// returns the byte used to parse `T`
    pub fn consumed(&self) -> usize {
        self.parsed.as_ref().len()
    }
}
