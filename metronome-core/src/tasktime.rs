use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct TaskTime {
    // Time data for the task for recording duration in H:M:S form
    hours: i64,
    minutes: i64,
    seconds: i64,
    // Number of total seconds for the task
    raw: i64,
}

impl From<i64> for TaskTime {
    fn from(total_seconds: i64) -> Self {
        let hours: i64 = total_seconds / 3600;
        let mut remainder: i64 = total_seconds % 3600;
        let minutes: i64 = remainder / 60;
        remainder = remainder % 60;
        let seconds = remainder;

        Self {
            hours,
            minutes,
            seconds,
            raw: total_seconds,
        }
    }
}

impl Display for TaskTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!("{}h {}m {}s", self.hours, self.minutes, self.seconds)
        )
    }
}
