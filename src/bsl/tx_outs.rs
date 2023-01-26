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
            visit.visit_tx_out(i as usize, tx_out.parsed());
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
}

impl<'a> AsRef<[u8]> for TxOuts<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
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
        let tx_out_bytes = hex!("ffffffffffffffff0100");
        let mut tx_outs = vec![];
        tx_outs.push(2u8);
        tx_outs.extend(&tx_out_bytes);
        tx_outs.extend(&tx_out_bytes);
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

    #[test]
    fn visit_tx_outs() {
        let tx_out_bytes = hex!("ffffffffffffffff0100");
        let mut tx_outs = vec![];
        tx_outs.push(2u8);
        tx_outs.extend(&tx_out_bytes);
        tx_outs.extend(&tx_out_bytes);

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

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<TxOuts>(), 24);
    }
}
