use crate::data::Data;

use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub struct FbEncoder {}
impl Encoder<Data> for FbEncoder {
    type Error = eyre::Report;

    fn encode(&mut self, item: Data, dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
