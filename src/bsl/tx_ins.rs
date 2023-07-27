use core::ops::ControlFlow;

use super::len::{parse_len, Len};
use crate::bsl::TxIn;
use crate::{Parse, ParseResult, SResult, Visit, Visitor};

/// The transaction inputs of a transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxIns<'a> {
    slice: &'a [u8],
    n: usize,
}

impl<'a> Visit<'a> for TxIns<'a> {
    fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Self> {
        let Len { mut consumed, n } = parse_len(slice)?;
        let mut remaining = &slice[consumed..];
        let total_inputs = n as usize;
        visit.visit_tx_ins(total_inputs);

        for i in 0..total_inputs {
            let tx_in = TxIn::parse(remaining)?;
            remaining = tx_in.remaining();
            consumed += tx_in.consumed();
            if let ControlFlow::Break(_) = visit.visit_tx_in(i, tx_in.parsed()) {
                return Err(crate::Error::VisitBreak);
            }
        }

        Ok(ParseResult::new(
            &slice[consumed..],
            TxIns {
                slice: &slice[..consumed],
                n: total_inputs,
            },
        ))
    }
}
impl<'a> TxIns<'a> {
    /// Returns if there are no transaction inputs
    pub fn is_empty(&self) -> bool {
        self.slice[0] == 0
    }
    /// Return the number of transaction inputs
    pub fn n(&self) -> usize {
        self.n
    }
}

impl<'a> AsRef<[u8]> for TxIns<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> TxIns<'a> {}

#[cfg(test)]
mod test {
    use core::ops::ControlFlow;

    use hex_lit::hex;

    use crate::{
        bsl::{TxIn, TxIns},
        Error, Parse, ParseResult, Visit, Visitor,
    };

    #[test]
    fn parse_tx_ins() {
        let tx_in_bytes = hex!("a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff");
        let mut tx_ins = vec![];
        tx_ins.push(2u8);
        tx_ins.extend(&tx_in_bytes);
        tx_ins.extend(&tx_in_bytes);
        let tx_ins_expected = TxIns {
            slice: &tx_ins[..],
            n: 2,
        };
        assert_eq!(
            TxIns::parse(&tx_ins[..]),
            Ok(ParseResult::new_exact(tx_ins_expected))
        );

        assert_eq!(
            TxIns::parse(&[0u8][..]),
            Ok(ParseResult::new_exact(TxIns {
                slice: &[0u8][..],
                n: 0
            }))
        );

        assert_eq!(
            TxIns::parse(&tx_ins[..tx_ins.len() - 1]),
            Err(Error::Needed(1))
        );
    }

    #[test]
    fn visit_tx_ins() {
        let tx_in_bytes = hex!("a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff");
        let mut tx_ins = vec![];
        tx_ins.push(2u8);
        tx_ins.extend(&tx_in_bytes);
        tx_ins.extend(&tx_in_bytes);

        struct VisitTxIns(usize);
        impl Visitor for VisitTxIns {
            fn visit_tx_in(&mut self, vin: usize, tx_in: &TxIn) -> ControlFlow<()> {
                assert_eq!(vin, self.0);
                self.0 += 1;
                assert_eq!(tx_in.sequence(), 4294967295u32);
                ControlFlow::Continue(())
            }
            fn visit_tx_ins(&mut self, n: usize) {
                assert_eq!(n, 2);
            }
        }
        TxIns::visit(&tx_ins[..], &mut VisitTxIns(0)).unwrap();

        struct IsMine(Vec<u8>, bool);
        impl Visitor for IsMine {
            fn visit_tx_in(&mut self, _vin: usize, tx_in: &TxIn) -> ControlFlow<()> {
                assert_eq!(tx_in.sequence(), 4294967295u32);
                if tx_in.script_sig() == self.0 {
                    self.1 = true;
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
        }
        let mut is_mine = IsMine(hex!("493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52").to_vec(), false);
        let _ = TxIns::visit(&tx_ins[..], &mut is_mine);
        assert!(is_mine.1);

        let mut is_mine = IsMine(vec![1u8], false);
        let _ = TxIns::visit(&tx_ins[..], &mut is_mine);
        assert!(!is_mine.1);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<TxIns>(), 24);
    }
}
