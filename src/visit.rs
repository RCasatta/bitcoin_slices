use super::bsl;

/// every `visit()` function take a `visit` parameter that implements this trait.
///
/// By default every visit function is an empty blanket implementation, the user could implement
/// the one for the interested data.
///
/// Not every function is called, for example `visit_block_header` is obviously not called when
/// parsing a transasction.
/// Traits with single function would have been more precies, however, it would have required the
/// user to implement those trait with an empty block which was considered too verbose. Morever it
/// looks a single trait with many functions is more perfomant.
#[allow(unused)]
pub trait Visitor {
    /// Visit the block header, called from [`bsl::Block::visit()`] and [`bsl::BlockHeader::visit()`]
    fn visit_block_header(&mut self, header: &bsl::BlockHeader) {}
    /// Visit the number of transactions in a block, called from [`bsl::Block::visit()`]
    fn visit_block_begin(&mut self, total_transactions: usize) {}

    /// Visit a transaction, called from  [`bsl::Block::visit()`] and  [`bsl::Transaction::visit()`]
    ///
    /// Note you can't access inputs and outputs from the transaction, you need [`Visitor::visit_tx_ins()`]
    /// or [`Visitor::visit_tx_outs()`]
    fn visit_transaction(&mut self, tx: &bsl::Transaction) {}

    /// We are going to visit `total_inputs` transaction inputs
    fn visit_tx_ins(&mut self, total_inputs: usize) {}
    /// Visit transaction input at position `vin`
    fn visit_tx_in(&mut self, vin: usize, tx_in: &bsl::TxIn) {}

    /// We are going to visit `total_outputs` transaction outputs
    fn visit_tx_outs(&mut self, total_outputs: usize) {}
    /// Visit transaction output at position `vout`
    fn visit_tx_out(&mut self, vout: usize, tx_out: &bsl::TxOut) {}

    /// We are going to visit the witnes of the `vin` input
    fn visit_witness(&mut self, vin: usize) {}
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
