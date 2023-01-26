use super::len::{parse_len, Len};
use crate::Visit;
use crate::{slice::read_slice, ParseResult, SResult, Visitor};

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

impl<'a> Visit<'a> for Witness<'a> {
    fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Witness<'a>> {
        let Len { mut consumed, n } = parse_len(slice)?;
        let mut remaining = &slice[consumed..];
        let witness_total_element = n as usize;

        visit.visit_witness_total_element(witness_total_element);
        for i in 0..witness_total_element {
            let len = parse_len(remaining)?;
            let sl = read_slice(&remaining[len.consumed()..], len.n() as usize)?;
            remaining = sl.remaining();
            consumed += len.slice_len();
            visit.visit_witness_element(i, sl.parsed());
        }

        let witness = Witness {
            slice: &slice[..consumed],
        };
        Ok(ParseResult::new(&slice[consumed..], witness))
    }
}
impl<'a> Witness<'a> {
    /// If this witness contain no elements
    pub fn is_empty(&self) -> bool {
        self.slice[0] == 0
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::Witness, Parse, ParseResult, Visit, Visitor};
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
