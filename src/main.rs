mod cli;
mod core;

use core::filters::Filter;
use rusqlite::Connection;

fn main() -> rusqlite::Result<()> {
    let conn = Connection::open(core::DB_NAME)?;
    core::create_task_table(&conn)?; // Only created if it does not yet exist

    let matches = cli::match_cli();

    match matches.subcommand() {
        Some(("start", sub_args)) => {
            let task: &String = sub_args.get_one("task").unwrap(); // required argument
            let category = sub_args.get_one("category");
            core::start_task(&conn, task, category)
        }
        Some(("end", sub_args)) => {
            if sub_args.get_flag("all") {
                core::end_all_active(&conn)
            } else if sub_args.get_flag("last") {
                core::end_last(&conn)
            } else {
                let task: &String = sub_args.get_one("task").unwrap(); // required argument
                core::end_task(&conn, task)
            }
        }
        Some(("list", sub_args)) => {
            let filter = Filter::from(sub_args.get_one("filter"));
            if sub_args.get_flag("active") {
                core::list_active(&conn, filter)
            } else if sub_args.get_flag("completed") {
                core::list_complete(&conn, filter)
            } else {
                core::list_all(&conn, filter)
            }
        }
        Some(("total", sub_args)) => {
            let filter = Filter::from(sub_args.get_one::<String>("filter"));
            core::sum_task_times(&conn, filter, sub_args.get_one("category"))
        }
        _ => {
            // TODO automatically show help menu and exit
            panic!("Invalid subcommand. See help menu for list of valid commands.")
        }
    }?;

    conn.close()
        .expect(format!("Could not close connection to {}", core::DB_NAME).as_str());

    Ok(())
}
