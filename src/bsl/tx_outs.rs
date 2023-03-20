use super::len::{parse_len, Len};
use crate::bsl::TxOut;
use crate::{Parse, ParseResult, SResult, Visit, Visitor};

/// The transaction outputs of a transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOuts<'a> {
    slice: &'a [u8],
    n: usize,
}

impl<'a> Visit<'a> for TxOuts<'a> {
    fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Self> {
        let Len { mut consumed, n } = parse_len(slice)?;
        let mut remaining = &slice[consumed..];
        let total_outputs = n as usize;
        visit.visit_tx_outs(total_outputs);

        for i in 0..total_outputs {
            let tx_out = TxOut::parse(remaining)?;
            remaining = tx_out.remaining();
            consumed += tx_out.consumed();
            visit.visit_tx_out(i, tx_out.parsed());
        }
        Ok(ParseResult::new(
            &slice[consumed..],
            TxOuts {
                slice: &slice[..consumed],
                n: total_outputs,
            },
        ))
    }
}
impl<'a> TxOuts<'a> {
    /// If there are no outputs.
    pub fn is_empty(&self) -> bool {
        self.slice[0] == 0
    }
    /// The number of outputs.
    pub fn n(&self) -> usize {
        self.n
    }

    /// Returns an iterator over [`bitcoin::TxOut`]
    ///
    /// If possible is better to use [`Visitor::visit_tx_out`] to avoid double pass, however, it may
    /// be conveniet to iterate in case you already have validated the slice, for example some data
    /// in a db.
    pub fn iter(&self) -> impl Iterator<Item = TxOut> {
        let len = parse_len(self.slice).expect("len granted by parsing");
        TxOutIterator {
            elements: len.n() as usize,
            offset: len.consumed(),
            tx_outs: self,
        }
    }
}

struct TxOutIterator<'a> {
    elements: usize,
    offset: usize,
    tx_outs: &'a TxOuts<'a>,
}

impl<'a> Iterator for TxOutIterator<'a> {
    type Item = TxOut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.tx_outs.as_ref().len() {
            None
        } else {
            let tx_out =
                TxOut::parse(&self.tx_outs.slice[self.offset..]).expect("granted from parsing");
            self.offset += tx_out.consumed();
            Some(tx_out.parsed_owned())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.elements, Some(self.elements))
    }
}

impl<'a> ExactSizeIterator for TxOutIterator<'a> {}

impl<'a> AsRef<[u8]> for TxOuts<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(feature = "redb")]
impl<'o> redb::RedbValue for TxOuts<'o> {
    type SelfType<'a> = TxOuts<'a>
    where
        Self: 'a;

    type AsBytes<'a> =  &'a [u8]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let n = parse_len(&data)
            .expect("inserted data is not a valid TxOuts")
            .n() as usize;
        TxOuts { slice: data, n }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.as_ref()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("TxOuts")
    }
}

#[cfg(test)]
mod test {
    use hex_lit::hex;

    use crate::{
        bsl::{TxOut, TxOuts},
        Error, Parse, ParseResult, Visit, Visitor,
    };

    #[test]
    fn parse_tx_outs() {
        let tx_outs = tx_outs_bytes();
        let tx_outs_expected = TxOuts {
            slice: &tx_outs[..],
            n: 2,
        };
        assert_eq!(
            TxOuts::parse(&tx_outs[..]),
            Ok(ParseResult::new_exact(tx_outs_expected))
        );

        assert_eq!(
            TxOuts::parse(&tx_outs[..tx_outs.len() - 1]),
            Err(Error::Needed(1))
        );
    }

    fn tx_outs_bytes() -> Vec<u8> {
        let tx_out_bytes = hex!("ffffffffffffffff0100");
        let mut tx_outs = vec![];
        tx_outs.push(2u8);
        tx_outs.extend(&tx_out_bytes);
        tx_outs.extend(&tx_out_bytes);
        tx_outs
    }

    #[test]
    fn visit_tx_outs() {
        let tx_outs = tx_outs_bytes();

        struct VisitTxOuts(usize);
        impl Visitor for VisitTxOuts {
            fn visit_tx_out(&mut self, vout: usize, tx_out: &TxOut) {
                assert_eq!(vout, self.0);
                self.0 += 1;
                assert_eq!(tx_out.value(), u64::MAX);
            }

            fn visit_tx_outs(&mut self, n: usize) {
                assert_eq!(n, 2);
            }
        }
        TxOuts::visit(&tx_outs[..], &mut VisitTxOuts(0)).unwrap();

        struct IsMine(Vec<u8>, bool);
        impl Visitor for IsMine {
            fn visit_tx_out(&mut self, _vout: usize, tx_out: &TxOut) {
                assert_eq!(tx_out.value(), u64::MAX);
                if tx_out.script_pubkey() == self.0 {
                    self.1 = true;
                }
            }
        }
        let mut visitor = IsMine(vec![0u8], false);
        TxOuts::visit(&tx_outs, &mut visitor).unwrap();
        assert!(visitor.1);

        let mut visitor = IsMine(vec![1u8], false);
        TxOuts::visit(&tx_outs, &mut visitor).unwrap();
        assert!(!visitor.1);
    }

    #[test]
    fn iter_tx_outs() {
        let mut tx_outs_bytes = tx_outs_bytes();
        *tx_outs_bytes.last_mut().unwrap() = 1;
        tx_outs_bytes.push(1);
        let tx_outs = TxOuts::parse(&tx_outs_bytes[..]).unwrap().parsed_owned();
        let mut iter = tx_outs.iter();
        let tx_out = iter.next().unwrap();
        assert_eq!(tx_out.value(), 0xffffffffffffffff);
        assert_eq!(tx_out.script_pubkey(), &[0]);

        let tx_out = iter.next().unwrap();
        assert_eq!(tx_out.value(), 0xffffffffffffffff);
        assert_eq!(tx_out.script_pubkey(), &[1]);

        assert!(iter.next().is_none());
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<TxOuts>(), 24);
    }

    #[cfg(feature = "redb")]
    #[test]
    fn test_tx_outs_redb() {
        use redb::ReadableTable;

        const TABLE: redb::TableDefinition<&str, TxOuts> = redb::TableDefinition::new("my_data");
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let db = redb::Database::create(path).unwrap();
        let tx_outs_bytes = tx_outs_bytes();
        let tx_outs = TxOuts::parse(&tx_outs_bytes).unwrap().parsed_owned();

        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            table.insert("", &tx_outs).unwrap();
        }
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();
        assert_eq!(table.get("").unwrap().unwrap().value(), tx_outs);
    }
}
