use tokio::sync::{broadcast, mpsc};

/// Allows signalling a shutdown, with a particular reason `ShutdownReason<R>`
/// for the shutdown. Also allows for listening to an acknowledgement `A` of
/// the shutdown from the various listeners.
///
/// If dropped, a shutdown of type [`ShutdownReason::BroadcasterClosed`] will
/// occur.
///
/// # Generics
/// - `R`: The reason for initiating the shutdown. Can be helpful when there
///   are multiple causes of a shutdown, and the listeners will clean up
///   differently depending on the shutdown reason.
/// - `A`: An acknowledgement of the shutdown, possibly containing any useful
///   information that can be sent back to the `ShutdownBroadcaster`.
pub struct Broadcaster<R: Clone = (), A = ()> {
	broadcaster: broadcast::Sender<R>,
	shutdown_watcher: mpsc::UnboundedReceiver<A>,
	// we hang on to this to be able to create new listeners.
	mpsc_copy: mpsc::UnboundedSender<A>,
}
impl<R: Clone, A> Broadcaster<R, A> {
	pub fn new() -> Self {
		let (b_sender, _) = broadcast::channel(1);
		let (mpsc_sender, mpsc_receiver) = mpsc::unbounded_channel();

		Self {
			broadcaster: b_sender,
			shutdown_watcher: mpsc_receiver,
			mpsc_copy: mpsc_sender,
		}
	}

	/// Creates a new `Listener` to the `Broadcaster`.
	pub fn new_listener(&self) -> Listener<R, A> {
		let b_receiver = self.broadcaster.subscribe();
		let mpsc_sender = self.mpsc_copy.clone();

		Listener {
			b_receiver,
			mpsc_sender,
			shutdown_reason: None,
		}
	}

	pub fn num_listeners(&self) -> usize {
		self.broadcaster.receiver_count()
	}

	/// Signals shutdown, with an optional reason. If no reason is provided, the
	/// listeners will get [`ShutdownReason::BroadcasterClosed`].
	///
	/// # Returns
	/// Returns a channel to be used for receiving the shutdown acknowledgements.
	pub fn signal_shutdown(self, reason: Option<R>) -> mpsc::UnboundedReceiver<A> {
		if let Some(r) = reason {
			// We don't care if all recipients are closed
			self.broadcaster.send(r).ok();
		}
		self.shutdown_watcher
	}
}
impl<R: Clone, A> Default for Broadcaster<R, A> {
	fn default() -> Self {
		Self::new()
	}
}

/// # Generics
/// See [`Broadcaster`] for documentation on the generic args.
pub struct Listener<R: Clone = (), A = ()> {
	b_receiver: broadcast::Receiver<R>,
	mpsc_sender: mpsc::UnboundedSender<A>,
	shutdown_reason: Option<ShutdownReason<R>>,
}
impl<R: Clone, I> Listener<R, I> {
	/// Doesn't return until a shutdown occurs.
	pub async fn recv(&mut self) -> &ShutdownReason<R> {
		if let Some(ref r) = self.shutdown_reason {
			return r;
		}
		let reason = match self.b_receiver.recv().await {
			Ok(r) => ShutdownReason::Reason(r),
			Err(broadcast::error::RecvError::Closed) => {
				ShutdownReason::BroadcasterClosed
			}
			Err(_) => unreachable!(
				"we shouldn't be able to lag, only 1 shutdown is ever sent."
			),
		};
		self.shutdown_reason = Some(reason);
		self.shutdown_reason.as_ref().unwrap()
	}

	/// Returns `None` if no shutdown has occurred, otherwise returns the shutdown
	/// reason.
	pub fn try_recv(&mut self) -> Option<&ShutdownReason<R>> {
		let reason = match self.b_receiver.try_recv() {
			Ok(r) => ShutdownReason::Reason(r),
			Err(broadcast::error::TryRecvError::Closed) => {
				ShutdownReason::BroadcasterClosed
			}
			Err(broadcast::error::TryRecvError::Empty) => return None,
			Err(broadcast::error::TryRecvError::Lagged(_)) => {
				unreachable!(
					"we shouldn't be able to lag, only 1 shutdown is ever sent."
				)
			}
		};
		self.shutdown_reason = Some(reason);
		self.shutdown_reason.as_ref()
	}

	/// Get the underlying shutdown reason, if present. Does not send any shut
	pub fn into_reason(self) -> Option<ShutdownReason<R>> {
		self.shutdown_reason
	}

	/// Block until an acknowledgement of the shutdown is sent. It is possible
	/// for the `Broadcaster` to have already closed.
	///
	/// # Returns
	/// Returns the original shutdown reason, if any.
	pub fn acknowledge(self, info: I) -> Option<ShutdownReason<R>> {
		self.mpsc_sender.send(info).ok();
		self.shutdown_reason
	}
}

#[derive(Clone)]
pub enum ShutdownReason<R: Clone> {
	BroadcasterClosed,
	Reason(R),
}
