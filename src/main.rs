mod cli;
mod db;

use db::filters::Filter;
use rusqlite::Connection;

fn main() -> rusqlite::Result<()> {
    let conn = Connection::open(db::DB_NAME)?;
    db::create_task_table(&conn)?; // Only created if it does not yet exist

    let matches = cli::match_cli();

    match matches.subcommand() {
        Some(("start", sub_args)) => {
            let task: &String = sub_args.get_one("task").unwrap(); // required argument
            let category = sub_args.get_one("category");
            db::start_task(&conn, task, category)
        }
        Some(("end", sub_args)) => {
            match sub_args.get_flag("all") {
                true => db::end_all_active(&conn),
                false => {
                    let task: &String = sub_args.get_one("task").unwrap(); // required argument
                    db::end_task(&conn, task)
                }
            }
        }
        Some(("list", sub_args)) => {
            let filter = Filter::from(sub_args.get_one("filter"));
            if sub_args.get_flag("active") {
                db::list_active(&conn, filter)
            } else if sub_args.get_flag("completed") {
                db::list_complete(&conn, filter)
            } else {
                db::list_all(&conn, filter)
            }
        }
        Some(("total", sub_args)) => {
            let filter = Filter::from(sub_args.get_one::<String>("filter"));
            db::sum_task_times(&conn, filter, sub_args.get_one("category"))
        }
        _ => {
            // TODO automatically show help menu and exit
            panic!("Invalid subcommand. See help menu for list of valid commands.")
        }
    }?;

    Ok(())
}
