use alloc::format;
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "_: deku::ctx::Endian, tag: u32", id = "tag", endian = "big")]
pub enum CBPacket {
	#[deku(id = "0")]
	Discovery,
	#[deku(id = "1")]
	Heartbeat,
	#[deku(id = "10")]
	Ping { id: u32 },
}