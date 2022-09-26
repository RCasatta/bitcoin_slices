use crate::bsl::Len;
use crate::{slice::read_slice, EmptyVisitor, ParseResult, SResult, Visitor};

/// A single witness associated with a single transaction input.
/// Logically is a vector of bytes vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness<'a> {
    slice: &'a [u8],
}

impl<'a> AsRef<[u8]> for Witness<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

impl<'a> Witness<'a> {
    /// Parse the witness in the slice
    pub fn parse(slice: &'a [u8]) -> SResult<Self> {
        Self::visit(slice, &mut EmptyVisitor {})
    }
    /// Visit the witness in the slice
    pub fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Witness<'a>> {
        let len = Len::parse(slice)?;
        let mut consumed = len.consumed();
        let mut remaining = len.remaining();
        let witness_total_element = len.parsed().n() as usize;

        visit.visit_witness_total_element(witness_total_element);
        for i in 0..witness_total_element {
            let len = Len::parse(remaining)?;
            let sl = read_slice(len.remaining(), len.parsed().n() as usize)?;
            remaining = sl.remaining();
            consumed += len.parsed().slice_len();
            visit.visit_witness_element(i, sl.parsed());
        }

        let witness = Witness {
            slice: &slice[..consumed],
        };
        Ok(ParseResult::new(&slice[consumed..], witness))
    }
    /// If this witness contain no elements
    pub fn is_empty(&self) -> bool {
        self.slice[0] == 0
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::Witness, ParseResult, Visitor};
    use hex_lit::hex;

    #[test]
    fn parse_witness() {
        let witness = hex!("0201000100");

        let expected = Witness {
            slice: &witness[..],
        };

        assert_eq!(
            Witness::parse(&witness[..]),
            Ok(ParseResult::new_exact(expected))
        );
    }

    #[test]
    fn visit_witness() {
        let witness = hex!("0201000101");
        struct WitnessVisititor(usize);
        impl Visitor for WitnessVisititor {
            fn visit_witness_total_element(&mut self, witness_total: usize) {
                assert_eq!(witness_total, 2);
            }
            fn visit_witness_element(&mut self, witness_i: usize, witness_element: &[u8]) {
                assert_eq!(witness_i, self.0);
                assert_eq!(witness_element, &[self.0 as u8]);
                self.0 += 1;
            }
        }
        Witness::visit(&witness[..], &mut WitnessVisititor(0)).unwrap();
    }
}
