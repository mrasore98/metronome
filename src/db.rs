pub(crate) mod filters;
mod tasktime;

use chrono::{DateTime, Local, TimeDelta};
use rusqlite::{params, Connection, Rows, Statement};

use filters::Filter;
use tasktime::TaskTime;

pub const DB_NAME: &str = "tasks.db";

pub fn create_task_table(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS tasks (\
        id INTEGER PRIMARY KEY NOT NULL, \
        name TEXT NOT NULL, \
        start_time INTEGER NOT NULL, \
        end_time INTEGER, \
        total_time INTEGER, \
        category TEXT NOT NULL, \
        status TEXT NOT NULL\
        )",
        (),
    )?;
    Ok(())
}

// START FUNCTIONS
pub fn start_task(
    connection: &Connection,
    task_name: &String,
    category: Option<&String>,
) -> rusqlite::Result<()> {
    // Task start time
    let start_time_dt = Local::now();

    // Values for creating new task
    let default_category = "Misc".to_string();
    let category = category.unwrap_or_else(|| &default_category);
    let start_time = start_time_dt.timestamp();
    let status = "Active";

    connection.execute(
        "INSERT INTO tasks (name, start_time, category, status) VALUES\
        (?1, ?2, ?3, ?4)",
        params![task_name, start_time, category, status],
    )?;

    println!(
        "Task \"{}\" started at {}!",
        task_name,
        start_time_dt.format("%c")
    );

    Ok(())
}

// END FUNCTIONS
pub fn end_task(connection: &Connection, task_name: &String) -> rusqlite::Result<()> {
    let end_time_dt = Local::now();
    let end_time = end_time_dt.timestamp();

    println!(
        "Ending task \"{}\" at {}",
        task_name,
        end_time_dt.format("%c")
    );
    let mut stmt = connection.prepare("SELECT start_time FROM tasks WHERE name = ?")?;

    let start_time: i64 = stmt.query_row(params![task_name], |row| row.get(0))?;
    let total_time = end_time - start_time;
    let status = "Complete";

    connection.execute(
        "UPDATE tasks SET end_time = ?1, total_time = ?2, status = ?3 WHERE name = ?4",
        params![end_time, total_time, status, task_name],
    )?;

    println!(
        "Task \"{}\" ended after {}",
        task_name,
        TaskTime::from(total_time)
    );

    Ok(())
}

pub fn end_all_active(connection: &Connection) -> rusqlite::Result<()> {
    let end_time_dt = Local::now();
    let end_time = end_time_dt.timestamp();
    let status = "Complete";

    // Update the end time for use in calculating total time
    connection.execute(
        "UPDATE tasks SET end_time = ?1 WHERE status = 'Active'",
        params![end_time],
    )?;

    // Update total time and set status to complete
    connection.execute(
"UPDATE tasks SET total_time = (end_time - start_time), status = ?1 WHERE status = 'Active'",
    params![status])?;

    // TODO add count of active tasks
    println!("Ended all active tasks at {}.", end_time_dt.format("%c"));

    Ok(())
}

// LIST FUNCTIONS

fn list_from_stmt(mut stmt: Statement, filter_start_time: i64) -> rusqlite::Result<()> {
    let rows = stmt.query(params![filter_start_time])?;
    print_list_rows(rows)?;

    Ok(())
}
pub fn list_active(connection: &Connection, filter: Filter) -> rusqlite::Result<()> {
    let start_time = parse_filter(filter);
    let stmt = connection.prepare(
        "SELECT * from tasks WHERE status = 'Active' \
         AND start_time > ?1",
    )?;
    return list_from_stmt(stmt, start_time);
}

pub fn list_complete(connection: &Connection, filter: Filter) -> rusqlite::Result<()> {
    let start_time = parse_filter(filter);
    let stmt = connection.prepare(
        "SELECT * from tasks WHERE status = 'Complete' \
        AND start_time > ?1",
    )?;
    return list_from_stmt(stmt, start_time);
}

pub fn list_all(connection: &Connection, filter: Filter) -> rusqlite::Result<()> {
    let start_time = parse_filter(filter);
    let stmt = connection.prepare("SELECT * from tasks WHERE start_time > ?1")?;

    return list_from_stmt(stmt, start_time);
}

// TOTAL FUNCTIONS
pub fn sum_task_times(
    connection: &Connection,
    filter: Filter,
    category: Option<&String>,
) -> rusqlite::Result<()> {
    let mut stmt: Statement;
    let rows: Rows;
    let start_time = parse_filter(filter);
    match category {
        Some(category) => {
            stmt = connection.prepare(
                "SELECT category, SUM(total_time) FROM tasks WHERE start_time > ?1 AND category = ?2 \
                GROUP BY category ORDER BY SUM(total_time) DESC"
            )?;
            rows = stmt.query(params![start_time, category])?;
        }
        None => {
            stmt = connection.prepare(
                "SELECT category, SUM(total_time) FROM tasks WHERE start_time > ?1 \
                GROUP BY category ORDER BY SUM(total_time) DESC",
            )?;
            rows = stmt.query(params![start_time])?;
        }
    }

    // Print results
    print_total_time_rows(rows)?;

    Ok(())
}

// HELPER FUNCTIONS

fn parse_filter(filter: Filter) -> i64 {
    let timedelta = match filter {
        Filter::Day => TimeDelta::days(1),
        Filter::Week => TimeDelta::weeks(1),
        Filter::Month => TimeDelta::days(30),
        Filter::Quarter => TimeDelta::weeks(13),
        Filter::SemiAnnual => TimeDelta::weeks(26),
        Filter::Year => TimeDelta::days(365),
        _ => TimeDelta::zero(), // need a time delta to match type but will not be used
    };

    if !timedelta.is_zero() {
        (Local::now() - timedelta).timestamp()
    } else {
        0
    }
}

fn print_list_rows(mut rows: Rows) -> rusqlite::Result<()> {
    // Status does not seem necessary since active tasks will have NULL end times and total times
    let headers = (
        "ID",
        "TASK",
        "START TIME",
        "END TIME",
        "TOTAL TIME",
        "CATEGORY",
    );

    println!(
        "| {:^4} | {:^40} | {:^30} | {:^30} | {:^15} | {:^20} |",
        headers.0, headers.1, headers.2, headers.3, headers.4, headers.5
    );
    println!("{}", "=".repeat(158));
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let task: String = row.get(1)?;
        let start_time_nix: i64 = row.get(2)?;
        let end_time_nix: Option<i64> = row.get(3)?;
        let total_time_s: Option<i64> = row.get(4)?;
        let category: String = row.get(5)?;

        let start_time = DateTime::from_timestamp(start_time_nix, 0)
            .unwrap()
            .naive_local()
            .format("%c")
            .to_string();

        let end_time = {
            if end_time_nix.is_some() {
                DateTime::from_timestamp(end_time_nix.unwrap(), 0)
                    .unwrap()
                    .naive_local()
                    .format("%c")
                    .to_string()
            } else {
                "NULL".to_string()
            }
        };

        let total_time = match total_time_s {
            Some(time_s) => TaskTime::from(time_s).to_string(),
            None => "NULL".to_string(),
        };

        println!(
            "| {:^4} | {:^40} | {:^30} | {:^30} | {:^15} | {:^20} |",
            id, task, start_time, end_time, total_time, category
        );
    }

    Ok(())
}

fn print_total_time_rows(mut rows: Rows) -> rusqlite::Result<()> {
    let headers = ("CATEGORY", "TOTAL TIME");
    println!("| {:^20} | {:^15} |", headers.0, headers.1);
    println!("{}", "=".repeat(42));

    while let Some(row) = rows.next()? {
        let category: String = row.get(0)?;
        let total_time_s: i64 = row.get(1)?;

        // Format from total seconds to h m d format
        let total_time_fmt = TaskTime::from(total_time_s).to_string();

        println!("| {:^20} | {:^15} |", category, total_time_fmt);
    }

    Ok(())
}
