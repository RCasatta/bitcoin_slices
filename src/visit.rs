use crate::SResult;

use super::bsl;

/// Parse and return an object like [`crate::bsl::Len`] without visiting it.
///
/// We don't provide is_empty like suggested by clippy because it would have different meaning:
/// eg `TxOuts(&[0u8])` is considered empty because there are no tx outputs but is not an empty slice.
#[allow(clippy::len_without_is_empty)]
pub trait Parse<'a>: Sized + AsRef<[u8]> {
    /// Parse the object from the slice
    fn parse(slice: &'a [u8]) -> SResult<'a, Self>;

    /// Return the serialized len of this object
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

/// Visit a blockchain object such as a [`crate::bsl::Block`] or a [`crate::bsl::Transaction`].
///
/// while consuming the slice it calls methods on the provided visitor.
#[allow(clippy::len_without_is_empty)]
pub trait Visit<'a>: Sized + AsRef<[u8]> {
    /// Visit the object from the slice while calling methods on the given visitor
    fn visit<'b, V: Visitor>(slice: &'a [u8], visit: &'b mut V) -> SResult<'a, Self>;

    /// Self visit calling methods on the given visitor.
    ///
    /// It's generally better to avoid a double pass and visit directly the first passing through
    /// the slice. However, there are case where the slice has already been validated, for example
    /// inserted in a db and you need to visit again.
    fn self_visit<'b, V: Visitor>(&'a self, visit: &'b mut V) -> SResult<'a, Self> {
        Self::visit(self.as_ref(), visit)
    }

    /// Return the serialized len of this object
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<'a, T: Visit<'a>> Parse<'a> for T {
    fn parse(slice: &'a [u8]) -> SResult<'a, Self> {
        Self::visit(slice, &mut EmptyVisitor {})
    }
}

/// every `visit()` function take a `visit` parameter that implements this trait.
///
/// By default every visit function is an empty blanket implementation, the user could implement
/// the one for the interested data.
///
/// Not every function is called, for example `visit_block_header` is obviously not called when
/// parsing a transasction.
/// Traits with single function would have been more precise, however, it would have required the
/// user to implement those trait with an empty block which was considered too verbose. Morever it
/// looks a single trait with many functions is more perfomant.
#[allow(unused)]
pub trait Visitor {
    /// Visit the block header, called from [`bsl::Block::visit()`] and [`bsl::BlockHeader::visit()`]
    fn visit_block_header(&mut self, header: &bsl::BlockHeader) -> core::ops::ControlFlow<()> {
        core::ops::ControlFlow::Continue(())
    }
    /// Visit the number of transactions in a block, called from [`bsl::Block::visit()`]
    fn visit_block_begin(&mut self, total_transactions: usize) {}

    /// Visit a transaction, called from  [`bsl::Block::visit()`] and  [`bsl::Transaction::visit()`]
    ///
    /// Note you can't access inputs and outputs from the transaction, you need [`Visitor::visit_tx_ins()`]
    /// or [`Visitor::visit_tx_outs()`]
    fn visit_transaction(&mut self, tx: &bsl::Transaction) -> core::ops::ControlFlow<()> {
        core::ops::ControlFlow::Continue(())
    }

    /// We are going to visit `total_inputs` transaction inputs
    fn visit_tx_ins(&mut self, total_inputs: usize) {}
    /// Visit transaction input at position `vin`
    fn visit_tx_in(&mut self, vin: usize, tx_in: &bsl::TxIn) -> core::ops::ControlFlow<()> {
        core::ops::ControlFlow::Continue(())
    }
    /// We are going to visit `total_outputs` transaction outputs
    fn visit_tx_outs(&mut self, total_outputs: usize) {}
    /// Visit transaction output at position `vout`
    fn visit_tx_out(&mut self, vout: usize, tx_out: &bsl::TxOut) -> core::ops::ControlFlow<()> {
        core::ops::ControlFlow::Continue(())
    }

    /// We are going to visit the witnes of the `vin` input
    fn visit_witness(&mut self, vin: usize) -> core::ops::ControlFlow<()> {
        core::ops::ControlFlow::Continue(())
    }
    /// The following witness has `witness_total` element
    fn visit_witness_total_element(&mut self, witness_total: usize) {}
    /// Visiting the `witness_i`ith element of this witness: `witness_element`
    fn visit_witness_element(&mut self, witness_i: usize, witness_element: &[u8]) {}
    /// Finishing visiting this witness
    fn visit_witness_end(&mut self) {}
}

/// A visitor with all empty function.
///
/// When `visit()` is present in structs, the `parse()` method is constructed by calling `visit`
/// with this empty visitor.
pub struct EmptyVisitor {}
impl Visitor for EmptyVisitor {}
