use std::time;

pub fn duration_span<F, R>(f: F) -> Result<(time::Duration, R)>
where
    F: FnOnce() -> Result<R>,
{
    let before = time::precise_time_ns();
    let r: R = f()?;
    let duration = time::Duration::nanoseconds((time::precise_time_ns() - before) as i64);
    Ok((duration, r))
}
