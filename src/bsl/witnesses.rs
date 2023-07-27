use core::ops::ControlFlow;

use crate::bsl::Witness;
use crate::{ParseResult, SResult, Visit};

/// Struct containining all the Witness in the tx (which is the same number as the inputs)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Witnesses<'a> {
    slice: &'a [u8],
    all_empty: bool,
}
impl<'a> AsRef<[u8]> for Witnesses<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Witnesses<'a> {
    /// Parse the witnesses in the slice
    pub fn parse(slice: &'a [u8], total_inputs: usize) -> SResult<Self> {
        Self::visit(slice, total_inputs, &mut crate::visit::EmptyVisitor {})
    }
    /// Visit the witnesses in the slice
    pub fn visit<'b, V: crate::Visitor>(
        slice: &'a [u8],
        total_inputs: usize,
        visit: &'b mut V,
    ) -> SResult<'a, Self> {
        let mut remaining = slice;
        let mut consumed = 0;
        let mut all_empty = true;
        for i in 0..total_inputs {
            if let ControlFlow::Break(_) = visit.visit_witness(i) {
                return Err(crate::Error::VisitBreak);
            }

            let witness = Witness::visit(remaining, visit)?;
            visit.visit_witness_end();

            remaining = witness.remaining();
            consumed += witness.consumed();
            if !witness.parsed().is_empty() {
                all_empty = false;
            }
        }
        let witnesses = Witnesses {
            slice: &slice[..consumed],
            all_empty,
        };
        Ok(ParseResult::new(&slice[consumed..], witnesses))
    }

    /// Returns if all the witness are empty (eg. contains only `0x00`)
    pub fn all_empty(&self) -> bool {
        self.all_empty
    }
}

#[cfg(test)]
mod test {
    use core::ops::ControlFlow;

    use crate::{bsl::Witnesses, Visitor};
    use hex_lit::hex;

    #[test]
    fn parse_witnesses() {
        let witnesses_bytes = hex!("0101000201000100"); // first witness is [[0]], second witness is [[0][0]]
        let witnesses = Witnesses::parse(&witnesses_bytes[..], 2).unwrap();
        assert_eq!(witnesses.remaining(), &[][..]);
        assert_eq!(witnesses.parsed().as_ref(), &witnesses_bytes[..]);
        assert_eq!(witnesses.consumed(), 8);
    }

    #[test]
    fn visit_witnesses() {
        let witnesses_bytes = hex!("0101000201010102"); // first witness is [[0]], second witness is [[1][2]]

        struct V {
            witness_vin: usize,
            witness_el_i: usize,
        }
        impl Visitor for V {
            fn visit_witness(&mut self, vin: usize) -> ControlFlow<()> {
                assert_eq!(vin, self.witness_vin);
                ControlFlow::Continue(())
            }
            fn visit_witness_total_element(&mut self, witness_total: usize) {
                match self.witness_vin {
                    0 => assert_eq!(witness_total, 1),
                    1 => assert_eq!(witness_total, 2),
                    _ => assert!(false),
                }
            }
            fn visit_witness_element(&mut self, _witness_i: usize, witness_element: &[u8]) {
                match (self.witness_vin, self.witness_el_i) {
                    (0, 0) => assert_eq!(witness_element, &[0u8]),
                    (1, 0) => assert_eq!(witness_element, &[1u8]),
                    (1, 1) => assert_eq!(witness_element, &[2u8]),
                    _ => assert!(false),
                }
                self.witness_el_i += 1;
            }
            fn visit_witness_end(&mut self) {
                self.witness_vin += 1;
                self.witness_el_i = 0;
            }
        }
        Witnesses::visit(
            &witnesses_bytes[..],
            2,
            &mut V {
                witness_vin: 0,
                witness_el_i: 0,
            },
        )
        .unwrap();
    }
}
