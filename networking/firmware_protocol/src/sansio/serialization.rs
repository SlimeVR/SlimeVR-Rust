/// A simpler alternative to [`SerializeExact`], which will serialize without needing an
/// exact buffer size.
pub trait Serialize {
	type Error;
	/// Serializes into `buf`, returning the number of bytes written, or an error.
	/// Note that this must not return `Ok` if only part of `self` was actually serialized.
	fn serialize(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Serializes directly into a buffer, such as directly into the tcp/udp buffers with no
/// the exact size known.
pub trait SerializeExact {
	type Error;
	/// Serializes the packet into the provided buffer. Implementations may choose to steal
	/// `self`, or perform a copy, or they may have already serialized into bytes preemptively
	/// (such as with flatbuffers).
	///
	/// # Panics
	/// May panic if `f` returns an Ok variant with a buffer that is not the exact size as
	/// `f`'s argument.
	fn serialize_exact<'a, 'b>(
		&'a mut self,
		f: impl FnOnce(usize) -> Result<&'b mut [u8], Self::Error>,
	) -> Result<(), Self::Error>;
}
