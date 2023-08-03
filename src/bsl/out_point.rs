use crate::{slice::read_slice, Parse, ParseResult, SResult};

/// The out point of a transaction input, identifying the previous output being spent
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutPoint<'a> {
    slice: &'a [u8],
}

impl<'a> AsRef<[u8]> for OutPoint<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Parse<'a> for OutPoint<'a> {
    /// Parse the out point from the given slice
    fn parse(slice: &'a [u8]) -> SResult<Self> {
        let outpoint = read_slice(slice, 36usize)?;
        Ok(ParseResult::new(
            outpoint.remaining(),
            OutPoint {
                slice: outpoint.parsed_owned(),
            },
        ))
    }
}
impl<'a> OutPoint<'a> {
    /// Returns the transaction txid of the previous output
    pub fn txid(&self) -> &[u8] {
        &self.slice[..32]
    }
    /// Returns the vout of the previous output
    pub fn vout(&self) -> u32 {
        let arr = self.slice[32..36]
            .as_ref()
            .try_into()
            .expect("slice length ensured by parsing");
        u32::from_le_bytes(arr)
    }
}

#[cfg(feature = "redb")]
impl<'o> redb::RedbValue for OutPoint<'o> {
    // TODO fix where position once MSRV allows
    type SelfType<'a>
    where
        Self: 'a,
    = OutPoint<'a>;

    type AsBytes<'a>
    where
        Self: 'a,
    = &'a [u8];

    fn fixed_width() -> Option<usize> {
        Some(36)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        OutPoint { slice: data }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.as_ref()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("bsl::OutPoint")
    }
}

#[cfg(feature = "redb")]
impl<'o> redb::RedbKey for OutPoint<'o> {
    fn compare(data1: &[u8], data2: &[u8]) -> core::cmp::Ordering {
        data1.cmp(data2)
    }
}

#[cfg(feature = "bitcoin")]
impl<'a> Into<bitcoin::OutPoint> for &OutPoint<'a> {
    fn into(self) -> bitcoin::OutPoint {
        use bitcoin::hashes::Hash;
        bitcoin::OutPoint {
            txid: bitcoin::Txid::from_byte_array(self.txid().try_into().unwrap()),
            vout: self.vout(),
        }
    }
}

#[cfg(feature = "bitcoin")]
impl<'a> Into<bitcoin::OutPoint> for OutPoint<'a> {
    fn into(self) -> bitcoin::OutPoint {
        (&self).into()
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::OutPoint, Error, Parse, ParseResult};

    #[test]
    fn parse_out_point() {
        let expected = OutPoint { slice: &[0u8; 36] };
        assert_eq!(OutPoint::parse(&[1u8]), Err(Error::Needed(35)));
        assert_eq!(OutPoint::parse(&[0u8; 35]), Err(Error::Needed(1)));
        assert_eq!(
            OutPoint::parse(&[0u8; 36]),
            Ok(ParseResult::new_exact(expected.clone()))
        );
        assert_eq!(
            OutPoint::parse(&[0u8; 37]),
            Ok(ParseResult::new(&[0u8][..], expected,))
        );
        let vec: Vec<_> = (0..36).collect();
        let txid: Vec<_> = (0..32).collect();
        let out_point = OutPoint::parse(&vec[..]).unwrap();
        assert_eq!(out_point.parsed().txid(), &txid[..]);
    }

    #[cfg(feature = "redb")]
    #[test]
    fn test_out_point_redb() {
        use redb::ReadableTable;

        const TABLE: redb::TableDefinition<OutPoint, OutPoint> =
            redb::TableDefinition::new("my_data");
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let db = redb::Database::create(path).unwrap();
        let out_point_slice = [1u8; 36];
        let out_point = OutPoint::parse(&out_point_slice).unwrap().parsed_owned();

        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            table.insert(&out_point, &out_point).unwrap();
        }
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();
        assert_eq!(table.get(&out_point).unwrap().unwrap().value(), out_point);
    }

    #[cfg(feature = "bitcoin")]
    #[test]
    fn test_tx_out_bitcoin() {
        let out_point_slice = [1u8; 36];
        let out_point = OutPoint::parse(&out_point_slice).unwrap().parsed_owned();

        let out_point_bitcoin: bitcoin::OutPoint =
            bitcoin::consensus::deserialize(out_point.as_ref()).unwrap();

        let out_point_bitcoin_bytes = bitcoin::consensus::serialize(&out_point_bitcoin);
        assert_eq!(&out_point_slice[..], &out_point_bitcoin_bytes[..]);
    }
}
