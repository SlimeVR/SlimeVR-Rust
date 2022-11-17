use solarxr_protocol::pub_sub::Message;

pub const TOPIC_ORG: &str = "slimevr.dev";
pub const TOPIC_APP: &str = "overlay";
pub const TOPIC_DISPLAY_SETTINGS: &str = "display_settings";

pub fn is_overlay_topic(msg: Message<'_>) -> bool {
	if let Some(topic_id) = msg.topic_as_topic_id() {
		matches!(topic_id.topic(), Some(TOPIC_DISPLAY_SETTINGS))
			&& matches!(topic_id.organization(), Some(TOPIC_ORG))
			&& matches!(topic_id.app_name(), Some(TOPIC_APP))
	} else if let Some(_topic_handle) = msg.topic_as_topic_handle() {
		// TODO: Actually check topic handle
		true
	} else {
		false
	}
}
