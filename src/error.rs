/// All possible error variants in the crate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// This bytes are needed to complete the parsing of the current element.
    ///
    /// Note it could be lower than needed to parse the whole object. For example if you pass a 20
    /// bytes slice to `OutPoint::parse(&slice[..20])` you will get `Error::Needed(12)` because the first
    /// element parsed is the hash of 32 bytes, but to complete the object you would need another
    /// 4 bytes of the `vout`.
    ///
    /// Note also it's an u32 instead of an usize to save space on 64 bits system and
    /// significantly improve performance.
    Needed(u32),

    /// Returned if the segwit parsed tx contains a segwit flag which is not 1
    UnknownSegwitFlag(u8),

    /// Segwit markers are found, but the transaction has no witnesses
    SegwitFlagWithoutWitnesses,

    /// The decoded varint is not in it's minimal form, eg. `0xFD0100` it's decoded as `1` but it's
    /// minimal encoding is `0x01`
    NonMinimalVarInt,

    /// The implemented visitor decided to break by returning `true` from [`crate::visit::Visitor::visit_transaction`]
    /// for example because it found what it was searching for
    VisitBreak,
}

#[cfg(test)]
mod test {

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(
            std::mem::size_of::<super::Error>(),
            8,
            "Size of Error type is important for performance, check benches"
        );
    }
}
