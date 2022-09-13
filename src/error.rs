#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Needed(usize),
    UnknwonNeeded,
    UnknownSegwitFlag(u8),
    SegwitFlagWithoutWitnesses,
}

pub fn to_unknown(e: Error) -> Error {
    if let Error::Needed(_) = e {
        Error::UnknwonNeeded
    } else {
        e
    }
}

pub fn to_unknown_if(e: Error, b: bool) -> Error {
    if b {
        to_unknown(e)
    } else {
        e
    }
}
