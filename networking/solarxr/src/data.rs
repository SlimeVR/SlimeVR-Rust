use solarxr_protocol::flatbuffers;
use solarxr_protocol::flatbuffers::InvalidFlatbuffer;
use solarxr_protocol::MessageBundle;

use ouroboros::self_referencing;
use std::fmt::Debug;

#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
	#[error("Flatbuffer failed verification: {0}")]
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
	/// # Safety
	/// `data` must have already been verified to be a valid [`MessageBundle`] flatbuffer
	pub unsafe fn from_vec_unchecked(data: Vec<u8>) -> Self {
		Self::new(data, |v| flatbuffers::root_unchecked::<MessageBundle>(v))
	}

	pub fn from_vec(data: Vec<u8>) -> Result<Self, (Vec<u8>, DecodeError)> {
		Self::try_new_or_recover(data, |v| flatbuffers::root::<MessageBundle>(v))
			.map_err(|(e, data)| (data.data, e.into()))
	}

	pub fn into_vec(self) -> Vec<u8> {
		self.into_heads().data
	}

	pub fn as_slice(&self) -> &[u8] {
		self.with_data(|v| v.as_slice())
	}

	pub fn table(&self) -> MessageBundle {
		*self.borrow_table()
	}
}

impl Debug for Data {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Data")
			.field("table", self.borrow_table())
			.finish()
	}
}

#[derive(Debug)]
pub struct FeedUpdate(pub Data);
