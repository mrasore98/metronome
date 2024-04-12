mod db;
mod cli;

use rusqlite::{Connection};


fn main() -> rusqlite::Result<()>{
    let conn = Connection::open(db::DB_NAME)?;
    db::create_task_table(&conn)?;  // Only created if it does not yet exist

    let matches = cli::match_cli();

    match matches.subcommand() {
        Some(("start", sub_args)) => {
            let task: &String = sub_args.get_one("task").unwrap();  // required argument
            let category = sub_args.get_one("category");
            db::start_task(&conn, task, category)
        },
        Some(("end", sub_args)) => {
            match sub_args.get_flag("all") {
                true => db::end_all_active(&conn),
                false => {
                    let task: &String = sub_args.get_one("task").unwrap();  // required argument
                    db::end_task(&conn, task)
                }
            }
        },
        Some(("list", sub_args)) => {
            // TODO figure out how to best implement the filters
            if sub_args.get_flag("active") {
                db::list_active(&conn, None)
            }
            else if sub_args.get_flag("completed"){
                db::list_complete(&conn, None)
            }
            else {
                db::list_all(&conn, None)
            }
        },
        Some(("total", sub_args)) => {
            db::sum_task_times(&conn, None, sub_args.get_one("category"))
        },
        _ => {
            // TODO automatically show help menu and exit
            panic!("Invalid subcommand. See help menu for list of valid commands.")
        },
    }?;

    Ok(())
}
