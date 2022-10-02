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
