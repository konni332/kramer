use std::time::Duration;

use vampirc_uci::UciTimeControl;

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

pub fn allocate_time_from_time_control(
    white: bool,
    time_control: Option<&UciTimeControl>,
) -> Option<Duration> {
    let tc = time_control?;

    match tc {
        UciTimeControl::TimeLeft {
            white_time,
            black_time,
            white_increment,
            black_increment,
            moves_to_go,
        } => {
            let time_ms = if white {
                white_time.map(|d| d.num_milliseconds() as u64)
            } else {
                black_time.map(|d| d.num_milliseconds() as u64)
            }?;

            let inc_ms = if white {
                white_increment.map(|d| d.num_milliseconds() as u64)
            } else {
                black_increment.map(|d| d.num_milliseconds() as u64)
            }
            .unwrap_or(0);

            Some(allocate_time(
                time_ms,
                inc_ms,
                moves_to_go.map(|m| m as u32),
            ))
        }
        UciTimeControl::MoveTime(duration) => Some(Duration::from_millis(
            duration.num_milliseconds().saturating_sub(50) as u64,
        )),
        UciTimeControl::Infinite => None,
        UciTimeControl::Ponder => None,
    }
}
