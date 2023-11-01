use crate::bsl::Script;
use crate::number::U64;
use crate::{Parse, ParseResult, SResult};

/// Contains a single transaction output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOut<'a> {
    slice: &'a [u8],
    value: u64,
    script_pubkey: Script<'a>,
}
impl<'a> Parse<'a> for TxOut<'a> {
    fn parse(slice: &'a [u8]) -> SResult<Self> {
        let value = U64::parse(slice)?;
        let script = Script::parse(value.remaining())?;
        let consumed = value.consumed() + script.consumed();
        let remaining = script.remaining();
        let tx_out = TxOut {
            slice: &slice[..consumed],
            value: value.parsed_owned().into(),
            script_pubkey: script.parsed_owned(),
        };
        Ok(ParseResult::new(remaining, tx_out))
    }
}
impl<'a> TxOut<'a> {
    /// Return the amount of this output (satoshi)
    pub fn value(&self) -> u64 {
        self.value
    }
    /// Return the script pubkey of this output
    pub fn script_pubkey(&self) -> &[u8] {
        self.script_pubkey.script()
    }
}

impl<'a> AsRef<[u8]> for TxOut<'a> {
    fn as_ref(&self) -> &[u8] {
        self.slice
    }
}

#[cfg(feature = "redb")]
impl<'o> redb::RedbValue for TxOut<'o> {
    // TODO fix where position once MSRV allows
    type SelfType<'a>
    where
        Self: 'a,
    = TxOut<'a>;

    type AsBytes<'a>
    where
        Self: 'a,
    = &'a [u8];

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        TxOut::parse(data)
            .expect("inserted data is not a TxOut")
            .parsed_owned()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.as_ref()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("bsl::TxOut")
    }
}

#[cfg(feature = "bitcoin")]
impl<'a> Into<bitcoin::TxOut> for &TxOut<'a> {
    fn into(self) -> bitcoin::TxOut {
        bitcoin::TxOut {
            value: bitcoin::Amount::from_sat(self.value()),
            script_pubkey: self.script_pubkey().to_vec().into(),
        }
    }
}

#[cfg(feature = "bitcoin")]
impl<'a> Into<bitcoin::TxOut> for TxOut<'a> {
    fn into(self) -> bitcoin::TxOut {
        (&self).into()
    }
}

#[cfg(test)]
mod test {
    use crate::{bsl::Script, bsl::TxOut, Parse, ParseResult};
    use hex_lit::hex;

    #[test]
    fn parse_tx_out() {
        let tx_out_bytes = hex!("ffffffffffffffff0100");

        let tx_out_expected = TxOut {
            slice: &tx_out_bytes[..],
            value: u64::MAX,
            script_pubkey: Script::parse(&hex!("0100")[..]).unwrap().parsed_owned(),
        };
        assert_eq!(
            TxOut::parse(&tx_out_bytes[..]),
            Ok(ParseResult::new_exact(tx_out_expected))
        );
    }

    #[cfg(feature = "redb")]
    #[test]
    fn test_tx_out_redb() {
        use redb::ReadableTable;

        const TABLE: redb::TableDefinition<&str, TxOut> = redb::TableDefinition::new("my_data");
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let db = redb::Database::create(path).unwrap();
        let tx_out_bytes = hex!("ffffffffffffffff0100");
        let tx_out = TxOut::parse(&tx_out_bytes).unwrap().parsed_owned();

        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(TABLE).unwrap();
            table.insert("", &tx_out).unwrap();
        }
        write_txn.commit().unwrap();

        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(TABLE).unwrap();
        assert_eq!(table.get("").unwrap().unwrap().value(), tx_out);
    }

    #[cfg(feature = "bitcoin")]
    #[test]
    fn test_tx_out_bitcoin() {
        let tx_out_bytes = hex!("ffffffffffffffff0100");
        let tx_out = TxOut::parse(&tx_out_bytes).unwrap().parsed_owned();

        let tx_out_bitcoin: bitcoin::TxOut =
            bitcoin::consensus::deserialize(tx_out.as_ref()).unwrap();
        let tx_out_bitcoin_bytes = bitcoin::consensus::serialize(&tx_out_bitcoin);
        assert_eq!(&tx_out_bytes[..], &tx_out_bitcoin_bytes[..]);

        let tx_out_back: bitcoin::TxOut = tx_out.into();

        assert_eq!(tx_out_back, tx_out_bitcoin);
    }
}
