use embassy_sync::blocking_mutex::raw::NoopRawMutex;

pub type Signal<T> = embassy_sync::signal::Signal<NoopRawMutex, T>;

pub struct Signals {
	/// The latest `Message` that should be sent
	pub latest: Signal<Message>,
	/// The `Message` that was already sent
	pub sent: Signal<Message>,
}
impl Signals {
	pub fn new() -> Self {
		Self {
			latest: Signal::new(),
			sent: Signal::new(),
		}
	}
}

pub struct Message;
impl Message {
	pub fn as_bytes(&self) -> &[u8] {
		b"todo"
	}
}
