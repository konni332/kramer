use std::time::Duration;

pub fn allocate_time(
    time_remaining_ms: u64,
    increment_ms: u64,
    moves_to_go: Option<u32>,
) -> Duration {
    let mtg = moves_to_go.unwrap_or(30) as u64;

    let base = time_remaining_ms / mtg;

    // add increment bonus (small buffer)
    let with_inc = base + (increment_ms * 3 / 4);

    // never use more than half time remaining
    let capped = with_inc.min(time_remaining_ms / 2);

    // always keep at least 50ms buffer
    let safe = capped.saturating_sub(50);

    Duration::from_millis(safe.max(10)) // never less than 10ms
}
