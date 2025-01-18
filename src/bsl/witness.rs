use super::scan_len;
use crate::{Error, Visit};
use crate::{ParseResult, SResult, Visitor};

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
        let mut consumed = 0usize;
        let n = scan_len(slice, &mut consumed)?;
        let witness_total_element = n as usize;

        visit.visit_witness_total_element(witness_total_element);
        for i in 0..witness_total_element {
            let len = scan_len(&slice[consumed..], &mut consumed)? as usize;
            let witness_element = &slice
                .get(consumed..consumed + len)
                .ok_or(Error::MoreBytesNeeded)?;
            consumed += len;
            visit.visit_witness_element(i, witness_element);
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

        let malformed_witness = hex!("02010001");
        assert_eq!(
            Witness::parse(&malformed_witness[..]),
            Err(crate::Error::MoreBytesNeeded)
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
