use std::fmt::Display;

pub struct TaskTime {
    hours: i64,
    minutes: i64,
    seconds: i64,
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
            seconds
        }
    }
}

impl Display for TaskTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{}h {}m {}s", self.hours, self.minutes, self.seconds))
    }

}