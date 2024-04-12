mod tasktime;

use std::fmt::format;
use std::string::ToString;
use chrono::{DateTime, Local, NaiveDateTime};
use rusqlite::{Connection, params, Rows};
use rusqlite::types::Value::Null;

use tasktime::TaskTime;

pub const DB_NAME: &str = "tasks.db";
pub const TABLE_NAME: &str = "tasks";


pub fn create_task_table(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute(
        format!("CREATE TABLE IF NOT EXISTS {} (\
        id INTEGER PRIMARY KEY NOT NULL, \
        name TEXT NOT NULL, \
        start_time INTEGER NOT NULL, \
        end_time INTEGER, \
        total_time INTEGER, \
        category TEXT NOT NULL, \
        status TEXT NOT NULL\
        )", TABLE_NAME).as_str()
        ,
        ()
    )?;
    Ok(())
}

// START FUNCTIONS
pub fn start_task(connection: &Connection,
                  task_name: &String,
                  category: Option<&String>) -> rusqlite::Result<()> {

    // Task start time
    let start_time_dt = Local::now();

    // Values for creating new task
    let default_category = "Misc".to_string();
    let category = category.unwrap_or_else(|| &default_category);
    let start_time = start_time_dt.timestamp();
    let status = "Active";

    connection.execute(
        format!("INSERT INTO {} (name, start_time, category, status) VALUES\
        (?1, ?2, ?3, ?4)", TABLE_NAME).as_str(),
        params![task_name, start_time, category, status]
    )?;

    println!("Task \"{}\" started at {}!", task_name, start_time_dt.format("%c"));

    Ok(())
}


// END FUNCTIONS
pub fn end_task(connection: &Connection, task_name: &String) -> rusqlite::Result<()> {
    let end_time_dt = Local::now();
    let end_time = end_time_dt.timestamp();

    println!("Ending task \"{}\" at {}", task_name,
             end_time_dt.format("%c"));
    let mut stmt = connection.prepare(
        format!("SELECT start_time FROM {} WHERE name = ?", TABLE_NAME).as_str())?;

    let start_time: i64 = stmt.query_row(params![task_name], |row| row.get(0))?;
    let total_time = end_time - start_time;
    let status = "Complete";

    connection.execute(
        format!("UPDATE {} SET end_time = ?1, total_time = ?2, status = ?3 WHERE name = ?4",
                TABLE_NAME).as_str(),
        params![end_time, total_time, status, task_name])?;

    println!("Task \"{}\" ended after {}", task_name, TaskTime::from(total_time));

    Ok(())
}

pub fn end_all_active(connection: &Connection) -> rusqlite::Result<()> {
    let end_time_dt = Local::now();
    let end_time = end_time_dt.timestamp();
    let status = "Complete";

    // Update the end time for use in calculating total time
    connection.execute(
        format!("UPDATE {} SET end_time = ?1 WHERE status = 'Active'",
                TABLE_NAME).as_str(),
        params![end_time])?;

    // Update total time and set status to complete
    connection.execute(
        format!("UPDATE {} SET total_time = (end_time - start_time), status = ?1 WHERE status = 'Active'",
                TABLE_NAME).as_str(),
    params![status])?;

    // TODO add count of active tasks
    println!("Ended all active tasks at {}.", end_time_dt.format("%c"));

    Ok(())
}

// LIST FUNCTIONS

fn list_from_stmt(mut stmt: rusqlite::Statement) -> rusqlite::Result<()> {
    let rows = stmt.query(())?;
    print_list_rows(rows)?;

    Ok(())
}
pub fn list_active(connection: &Connection, filter: Option<&String>) -> rusqlite::Result<()> {
    let stmt = connection.prepare(
        format!("SELECT * from {} WHERE status = 'Active'", TABLE_NAME).as_str())?;
    return list_from_stmt(stmt);
}

pub fn list_complete(connection: &Connection, filter: Option<&String>) -> rusqlite::Result<()> {
    let stmt = connection.prepare(
        format!("SELECT * from {} WHERE status = 'Complete'", TABLE_NAME).as_str())?;
    return list_from_stmt(stmt);
}

pub fn list_all(connection: &Connection, filter: Option<&String>) -> rusqlite::Result<()> {
    let stmt = connection.prepare(
        format!("SELECT * from {}",TABLE_NAME).as_str())?;
    return list_from_stmt(stmt);
}

// TOTAL FUNCTIONS
pub fn sum_task_times(connection: &Connection, filter:Option<&String>, category: Option<&String>)
    -> rusqlite::Result<()> {
    // TODO implement filters and categories
    let mut sql_stmt = format!("SELECT category, SUM(total_time) FROM {} GROUP BY category ORDER BY SUM(total_time) DESC", TABLE_NAME);
    // let category_is_some = category.is_some();
    // if category_is_some {
    //     sql_stmt += " WHERE category = ?1";
    // }
    // if filter.is_some() {
    //     sql_stmt += if category_is_some {" AND"} else {""};
    //     sql_stmt += format!(" WHERE start_time > ?{}", if category_is_some {2} else {1}).as_str();
    // }
    let mut stmt = connection.prepare(sql_stmt.as_str())?;
    // let rows = stmt.query(
    //     params![category.unwrap_or_else(|| ()), filter.unwrap_or_else(||)]
    // )?;
    let mut rows = stmt.query(())?;

    // Print results
    print_total_time_rows(rows)?;

    Ok(())
}

// HELPER FUNCTIONS

/// Parses a filter string to use for limiting the number of rows returned.
fn parse_filter(filter: Option<&String>) -> Option<DateTime<Local>> {
    todo!()
}

fn print_list_rows(mut rows: Rows) -> rusqlite::Result<()>{

    // Status does not seem necessary since active tasks will have NULL end times and total times
    let headers =
        ("ID", "TASK", "START TIME", "END TIME", "TOTAL TIME", "CATEGORY");


    println!("| {:^4} | {:^40} | {:^30} | {:^30} | {:^15} | {:^20} |",
             headers.0, headers.1, headers.2, headers.3, headers.4, headers.5);
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
                    .format("%c").to_string()
            }
            else {
                "NULL".to_string()
            }
        };

        let total_time = match total_time_s {
            Some(time_s) => TaskTime::from(time_s).to_string(),
            None => "NULL".to_string()
        };

        println!("| {:^4} | {:^40} | {:^30} | {:^30} | {:^15} | {:^20} |",
                 id, task, start_time, end_time, total_time, category);

    }

    Ok(())
}

fn print_total_time_rows(mut rows: Rows) -> rusqlite::Result<()> {
    let headers = ("CATEGORY", "TOTAL TIME");
    println!("| {:^20} | {:^15} |", headers.0, headers.1);
    println!("{}", "=".repeat(42));

    while let Some(row) = rows.next()?  {
        let category: String = row.get(0)?;
        let total_time_s: i64 = row.get(1)?;

        // Format from total seconds to h m d format
        let total_time_fmt = TaskTime::from(total_time_s).to_string();

        println!("| {:^20} | {:^15} |", category, total_time_fmt);
    }

    Ok(())
}



