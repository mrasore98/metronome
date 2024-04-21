use std::str::FromStr;
use std::string::ToString;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, EnumIter, EnumString, Display, Copy, Clone)]
pub enum Filter {
    #[strum(
        serialize = "d",
        serialize = "day",
        to_string = "Filtering events to those started in the last day"
    )]
    Day,

    #[strum(
        serialize = "w",
        serialize = "week",
        to_string = "Filtering events to those started in the last week"
    )]
    Week,

    #[strum(
        serialize = "m",
        serialize = "month",
        to_string = "Filtering events to those started in the last 30 days"
    )]
    Month,

    #[strum(
        serialize = "q",
        serialize = "quarter",
        to_string = "Filtering events to those started in the last quarter (13 weeks)"
    )]
    Quarter,

    #[strum(
        serialize = "s",
        serialize = "semi",
        serialize = "semiannual",
        to_string = "Filtering events to those started in the last 6 months (26 weeks)"
    )]
    SemiAnnual,

    #[strum(
        serialize = "y",
        serialize = "year",
        to_string = "Filtering events to those started in the last year (365 days)"
    )]
    Year,

    #[strum(to_string = "No filter will be applied")]
    All,
}

impl From<Option<&String>> for Filter {
    fn from(value: Option<&String>) -> Self {
        match value {
            Some(filter) => {
                let filter = Filter::from_str(filter.as_str()).unwrap_or(Filter::All);
                println!("** {} **", filter.to_string());
                filter
            }
            None => Filter::All,
        }
    }
}
