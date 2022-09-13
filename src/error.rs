/// All possible error variants in the crate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// This bytes are needed to complete the parsing of the last element. Note it could be lower
    /// than needed to parse the whole object. For example if you pass a 20 bytes slice to
    /// [`crate::bsl::OutPoint::parse()`] you will get `Error::Needed(12)` because the first element parsed is
    /// the hash of 32 bytes, but to complete the object you would need another 4 bytes of the `vout`
    Needed(usize),

    /// Returned if the segwit parsed tx contains a segwit flag which is not 1
    UnknownSegwitFlag(u8),

    /// Segwit markers are found, but the transaction has no witnesses
    SegwitFlagWithoutWitnesses,
}
