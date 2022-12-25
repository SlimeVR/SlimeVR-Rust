use embassy_futures::yield_now;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

/// Signals are used for concurrently updating values, where we only care about
/// keeping the latest value around
pub type Unreliable<T> = embassy_sync::signal::Signal<NoopRawMutex, T>;
/// Channel with a capacity of 1. Used instead of Signal when we need to have
/// back pressure and guaranteed reads of the value
pub type Reliable<T> = embassy_sync::channel::Channel<NoopRawMutex, T, 1>;

/// Retries function `f` by repeatedly calling it `n` times. Returns the first `Ok`
/// value or the last `Err`.
///
/// # Arguments
/// - `n` is the number of times to retry. `0` means no retries will be attempted, so
///   `f` will only be called once.
/// - `acc` is an "accumulator" which repeatedly gets passeed to the function each time
/// it is called.
/// - `f` is the function to retry.
/// - `before_retry(retry_num)` is a function that gets called right before invoking `f`
///   again on a retry. `retry_num` goes from `0..n`. Often useful for logging or
///   controlling a delay between invocations of `f`.
#[allow(dead_code)]
pub fn retry<A, T, E>(
	n: u8,
	acc: A,
	mut f: impl FnMut(A) -> Result<T, (A, E)>,
	mut before_retry: impl FnMut(u8),
) -> Result<T, (A, E)> {
	// First attempt
	let mut last_result = f(acc);
	// Any additional attempts, up to `n` times. Each time we update `last_result`.
	for i in 0..n {
		let acc = match last_result {
			Ok(t) => {
				last_result = Ok(t);
				break;
			}
			Err((acc, _err)) => acc,
		};
		before_retry(i);
		last_result = f(acc);
	}
	last_result
}

/// Converts a nb::Result to an async function by looping and yielding to the async
/// executor.
pub async fn nb2a<T, E>(mut f: impl FnMut() -> nb::Result<T, E>) -> Result<T, E> {
	loop {
		let v = f();
		match v {
			Ok(t) => return Ok(t),
			Err(nb::Error::Other(e)) => return Err(e),
			Err(nb::Error::WouldBlock) => yield_now().await,
		}
	}
}
