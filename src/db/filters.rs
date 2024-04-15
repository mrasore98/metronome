pub enum Filter {
    Day,
    Week,
    Month,
    Quarter,
    SemiAnnual,
    Year,
    All,
}

impl From<Option<&String>> for Filter {
    fn from(value: Option<&String>) -> Self {
        let filter: Self = match value {
            Some(filter) => {
                match filter.to_uppercase().as_str() {
                    "D" => {
                        println!("** Filtering events to those started in the last day **");
                        Filter::Day
                    }
                    "W" => {
                        println!("** Filtering events to those started in the last week **");
                        Filter::Week
                    }
                    "M" => {
                        println!("** Filtering events to those started in the last 30 days **");
                        Filter::Month
                    }
                    "Q" => {
                        println!("** Filtering events to those started in the last quarter (13 weeks) **");
                        Filter::Quarter
                    }
                    "S" => {
                        println!("** Filtering events to those started in the last 6 months (26 weeks) **");
                        Filter::SemiAnnual
                    }
                    "Y" => {
                        println!(
                            "** Filtering events to those started in the last year (365 days) **"
                        );
                        Filter::Year
                    }
                    _ => {
                        println!("Filter not recognized. No filter will be applied.");
                        Filter::All
                    }
                }
            }
            None => Filter::All,
        };

        filter
    }
}
