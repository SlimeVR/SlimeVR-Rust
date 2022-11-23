use core::future::Future;
use solarxr_protocol::data_feed::DataFeedMessageHeader;
use solarxr_protocol::data_feed::DataFeedUpdate;

/// Trait alias for callback function
pub trait DataFeedCallback: FnMut(DataFeedUpdate) -> Self::Fut {
	type Fut: Future<Output = ()>;
}
impl<Func, Fut> DataFeedCallback for Func
where
	Func: FnMut(DataFeedUpdate) -> Fut,
	Fut: Future<Output = ()>,
{
	type Fut = Fut;
}

pub async fn handle_data_feed(
	mut cb: impl DataFeedCallback,
	msgs: impl Iterator<Item = DataFeedMessageHeader<'_>>,
) {
	for m in msgs {
		let Some(update) = m.message_as_data_feed_update() else {
	        continue
	    };
		cb(update);
	}
}
