use std::pin::Pin;

use solarxr_protocol::flatbuffers;
use solarxr_protocol::flatbuffers::InvalidFlatbuffer;
use solarxr_protocol::MessageBundle;

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
    table: MessageBundle<'this>,
}
impl Data {
    pub unsafe fn from_vec_unchecked(data: Vec<u8>) -> Self {
        Self::new(data, |v| flatbuffers::root_unchecked::<MessageBundle>(v))
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, DecodeError> {
        Self::try_new(data, |v| flatbuffers::root::<MessageBundle>(v)).map_err(|e| e.into())
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.into_heads().data
    }

    pub fn as_slice(&self) -> &[u8] {
        self.with_data(|v| v.as_slice())
    }
}

pub type DataResult = Result<Data, DecodeError>;

pub struct FeedUpdate(pub Data);
