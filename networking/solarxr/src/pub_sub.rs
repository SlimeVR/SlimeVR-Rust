use solarxr_protocol::pub_sub::PubSubHeader;

pub async fn handle_pub_sub(msgs: impl Iterator<Item = PubSubHeader<'_>>) {
	// TODO: actually do this
	let _msgs = msgs;
}
