mod cli;

use metronome_core::{filters::Filter, set_database_location};
use rusqlite::Connection;

fn main() -> rusqlite::Result<()> {
    let db_loc = set_database_location();
    let conn = Connection::open(db_loc)?;
    metronome_core::create_task_table(&conn)?; // Only created if it does not yet exist

    let matches = cli::match_cli();

    match matches.subcommand() {
        Some(("start", sub_args)) => {
            let task: &String = sub_args.get_one("task").unwrap(); // required argument
            let category = sub_args.get_one("category");
            metronome_core::start_task(&conn, task, category)
        }
        Some(("end", sub_args)) => {
            if sub_args.get_flag("all") {
                metronome_core::end_all_active(&conn)
            } else if sub_args.get_flag("last") {
                metronome_core::end_last(&conn)
            } else {
                let task: &String = sub_args.get_one("task").unwrap(); // required argument
                metronome_core::end_task(&conn, task)
            }
        }
        Some(("list", sub_args)) => {
            let filter = Filter::from(sub_args.get_one("filter"));
            if sub_args.get_flag("active") {
                metronome_core::list_active(&conn, filter)
            } else if sub_args.get_flag("completed") {
                metronome_core::list_complete(&conn, filter)
            } else {
                metronome_core::list_all(&conn, filter)
            }
        }
        Some(("total", sub_args)) => {
            let filter = Filter::from(sub_args.get_one::<String>("filter"));
            metronome_core::sum_task_times(&conn, filter, sub_args.get_one("category"))
        }
        _ => {
            // TODO automatically show help menu and exit
            panic!("Invalid subcommand. See help menu for list of valid commands.")
        }
    }?;

    conn.close()
        .expect(format!("Could not close connection to {}", db_loc).as_str());

    Ok(())
}
