use slimevr_protocol::flatbuffers;
use slimevr_protocol::flatbuffers::InvalidFlatbuffer;
use slimevr_protocol::server::OutboundPacket;

use ouroboros::self_referencing;

#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    #[error("Flatbuffer failed verification")]
    FbVerification(#[from] InvalidFlatbuffer),
    // #[error("Io error: {0}")]
    // Io(#[from] std::io::Error),
}

/// Root flatbuffer type, after verification
#[self_referencing]
pub struct Data {
    data: Vec<u8>,
    #[borrows(data)]
    #[covariant]
    table: OutboundPacket<'this>,
}
impl Data {
    // TODO: add a new function

    pub fn deserialize(data: Vec<u8>) -> Result<Self, DecodeError> {
        Self::try_new(data, |v| flatbuffers::root::<OutboundPacket>(v)).map_err(|e| e.into())
    }

    pub fn serialize(self) -> Vec<u8> {
        self.into_heads().data
    }
}

pub type DataResult = Result<Data, DecodeError>;

// TODO
pub struct FeedUpdate;
