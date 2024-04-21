pub(crate) mod filters;
mod tasktime;

use chrono::{DateTime, Local, TimeDelta};
use fallible_streaming_iterator::FallibleStreamingIterator; // Needed to count returned SQLite Rows
use rusqlite::{params, Connection, Rows, Statement};

use self::MetronomeResults::{CreateTable, EndTask, List, StartTask, SumTaskTimes};
use filters::Filter;
use tasktime::TaskTime;

pub const DB_NAME: &str = "tasks.db";

#[derive(Debug, PartialEq)]
pub enum MetronomeResults {
    CreateTable,
    StartTask(i64),              // Returns start timestamp
    EndTask(i64, i64, TaskTime), // Returns end timestamp, total time in seconds, TaskTime from total time
    EndAllActive(usize),         // Returns number of activities ended
    List(usize),                 // Returns number of rows in the list
    SumTaskTimes,
}

pub fn create_task_table(connection: &Connection) -> rusqlite::Result<MetronomeResults> {
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
    Ok(CreateTable)
}

// START FUNCTIONS
pub fn start_task(
    connection: &Connection,
    task_name: &String,
    category: Option<&String>,
) -> rusqlite::Result<MetronomeResults> {
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

    Ok(StartTask(start_time))
}

// END FUNCTIONS
pub fn end_task(connection: &Connection, task_name: &String) -> rusqlite::Result<MetronomeResults> {
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

    let task_time = TaskTime::from(total_time);

    println!("Task \"{}\" ended after {}", task_name, task_time);

    Ok(EndTask(end_time, total_time, task_time))
}

pub fn end_last(connection: &Connection) -> rusqlite::Result<MetronomeResults> {
    let last_task: String = connection.query_row(
        "SELECT name FROM tasks WHERE start_time = (SELECT MAX(start_time) FROM tasks WHERE status = 'Active')",
        (),
        |row| row.get(0),
    )?;
    end_task(&connection, &last_task)
}

pub fn end_all_active(connection: &Connection) -> rusqlite::Result<MetronomeResults> {
    let end_time_dt = Local::now();
    let end_time = end_time_dt.timestamp();
    let status = "Complete";

    // Update the end time for use in calculating total time
    connection.execute(
        "UPDATE tasks SET end_time = ?1 WHERE status = 'Active'",
        params![end_time],
    )?;

    //Get the number of active tasks
    let mut stmt = connection.prepare("SELECT * FROM tasks WHERE status = 'Active'")?;
    let num_ended_tasks = stmt.query(())?.count()?;
    stmt.finalize()?;

    // Update total time and set status to complete
    connection.execute(
"UPDATE tasks SET total_time = (end_time - start_time), status = ?1 WHERE status = 'Active'",
    params![status])?;

    println!(
        "Ended {} active tasks at {}.",
        num_ended_tasks,
        end_time_dt.format("%c")
    );

    Ok(MetronomeResults::EndAllActive(num_ended_tasks))
}

// LIST FUNCTIONS

fn list_from_stmt(
    mut stmt: Statement,
    filter_start_time: i64,
) -> rusqlite::Result<MetronomeResults> {
    let rows = stmt.query(params![filter_start_time])?;
    let num_returned = print_list_rows(rows)?;

    Ok(List(num_returned))
}
pub fn list_active(connection: &Connection, filter: Filter) -> rusqlite::Result<MetronomeResults> {
    let start_time = parse_filter(filter);
    let stmt = connection.prepare(
        "SELECT * from tasks WHERE status = 'Active' \
         AND start_time > ?1",
    )?;
    return list_from_stmt(stmt, start_time);
}

pub fn list_complete(
    connection: &Connection,
    filter: Filter,
) -> rusqlite::Result<MetronomeResults> {
    let start_time = parse_filter(filter);
    let stmt = connection.prepare(
        "SELECT * from tasks WHERE status = 'Complete' \
        AND start_time > ?1",
    )?;
    return list_from_stmt(stmt, start_time);
}

pub fn list_all(connection: &Connection, filter: Filter) -> rusqlite::Result<MetronomeResults> {
    let start_time = parse_filter(filter);
    let stmt = connection.prepare("SELECT * from tasks WHERE start_time > ?1")?;

    return list_from_stmt(stmt, start_time);
}

// TOTAL FUNCTIONS
pub fn sum_task_times(
    connection: &Connection,
    filter: Filter,
    category: Option<&String>,
) -> rusqlite::Result<MetronomeResults> {
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

    Ok(SumTaskTimes)
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

fn print_list_rows(mut rows: Rows) -> rusqlite::Result<usize> {
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

    let mut row_count: usize = 0;
    while let Some(row) = rows.next()? {
        row_count += 1;
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

    Ok(row_count)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use strum::IntoEnumIterator; // For iterating through Filter enums in testing

    fn setup() -> rusqlite::Result<Connection> {
        let conn = Connection::open("unit_tests.db")?;
        create_task_table(&conn)?;
        Ok(conn)
    }

    fn teardown(connection: Connection) {
        connection
            .execute("DROP TABLE tasks", ())
            .expect("Table could not be dropped for teardown");
        connection
            .close()
            .expect("Connection to unit_tests.db was not successfully closed.");
        std::fs::remove_file("unit_tests.db").expect("Could not remove unit_tests.db");
    }

    #[test]
    fn test_start_task_no_category() -> rusqlite::Result<()> {
        // Setup
        let conn = setup()?;
        let task_name = "test_start_task_no_category".to_string();
        let expected_category = "Misc".to_string();

        // Add task to table
        let start_time = Local::now().timestamp();
        let returned_start_fn = match start_task(&conn, &task_name, None)? {
            StartTask(start_time) => start_time,
            _ => panic!("Unexpected enum returned from start_task function."),
        };

        assert!((start_time - returned_start_fn).abs() <= 1);

        let mut stmt = conn.prepare("SELECT * from tasks WHERE name = ?1")?;
        let (returned_name, returned_start, category) =
            stmt.query_row(params![task_name], |row| {
                Ok((
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(5)?,
                ))
            })?;
        stmt.finalize()?; // Needed to release the borrowed connection

        println!(
            "Set task name: {}, Returned task name: {}",
            task_name, returned_name
        );
        assert_eq!(task_name, returned_name);

        println!(
            "Expected start time: {}, Returned start time: {}",
            start_time, returned_start
        );
        assert!((start_time - returned_start).abs() <= 1);

        println!(
            "Expected category: {}, Returned category: {}",
            expected_category, category
        );
        assert_eq!(expected_category, category);

        teardown(conn);

        Ok(())
    }

    #[test]
    fn test_start_task_with_category() -> rusqlite::Result<()> {
        // Setup
        let conn = setup()?;
        let task_name = "test_start_task_with_category".to_string();
        let expected_category = "unit_tests".to_string();

        // Add task to table
        let start_time = Local::now().timestamp();
        let returned_start_fn = match start_task(&conn, &task_name, Some(&expected_category))? {
            StartTask(start_time) => start_time,
            _ => panic!("Unexpected enum returned from start_task function."),
        };

        assert!((start_time - returned_start_fn).abs() <= 1);

        let mut stmt = conn.prepare("SELECT * from tasks WHERE name = ?1")?;
        let (returned_name, returned_start, category) =
            stmt.query_row(params![task_name], |row| {
                Ok((
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(5)?,
                ))
            })?;
        stmt.finalize()?; // Needed to release the borrowed connection

        println!(
            "Set task name: {}, Returned task name: {}",
            task_name, returned_name
        );
        assert_eq!(task_name, returned_name);

        println!(
            "Expected start time: {}, Returned start time: {}",
            start_time, returned_start
        );
        assert!((start_time - returned_start).abs() <= 1);

        println!(
            "Expected category: {}, Returned category: {}",
            expected_category, category
        );
        assert_eq!(expected_category, category);

        teardown(conn);

        Ok(())
    }

    #[test]
    fn test_end_task() -> rusqlite::Result<()> {
        let conn = setup()?;

        // Start tasks to end
        let tasks_to_start = vec!["Task_A", "Task_B", "Task_C"];
        for task in &tasks_to_start {
            let task = task.to_string();
            start_task(&conn, &task, None)?; // category unimportant to this test
        }

        // Query the number of active tasks
        let sql = "SELECT * FROM tasks WHERE status = 'Active'";
        let mut stmt = conn.prepare(sql)?;
        let rows_before = stmt.query(())?.count()?;

        println!("Number of initial active tasks: {}", rows_before);
        assert_eq!(rows_before, tasks_to_start.len());

        // Call end task, ensure the number of active tasks are decreasing
        for (i, task) in tasks_to_start.iter().enumerate() {
            let task = task.to_string();
            let i = i + 1;
            let expected_rows = rows_before - i;

            println!("Calling end_task: Iteration {}", i);
            end_task(&conn, &task)?;

            let mut stmt = conn.prepare(sql)?;
            let rows_after = stmt.query(())?.count()?;

            println!(
                "Expected number of active tasks: {}, Returned number of active tasks: {}",
                expected_rows, rows_after
            );
            assert_eq!(expected_rows, rows_after);
        }

        stmt.finalize()?;
        teardown(conn);

        Ok(())
    }

    #[test]
    fn test_end_all_active() -> rusqlite::Result<()> {
        let conn = setup()?;

        // Start tasks to end
        let tasks_to_start = vec!["Task_A", "Task_B", "Task_C"];
        for task in &tasks_to_start {
            let task = task.to_string();
            start_task(&conn, &task, None)?; // category unimportant to this test
        }

        // Query the number of active tasks
        let sql = "SELECT * FROM tasks WHERE status = 'Active'";
        let mut stmt = conn.prepare(sql)?;
        let rows_before = stmt.query(())?.count()?;
        stmt.finalize()?; // Free the connection

        println!("Number of active tasks: {}", rows_before);
        assert_eq!(rows_before, tasks_to_start.len());

        // End all active tasks
        let num_ended = match end_all_active(&conn)? {
            MetronomeResults::EndAllActive(ended) => ended,
            _ => panic!("Unexpected enum returned from end_all_active call."),
        };

        assert_eq!(rows_before, num_ended);

        // Ensure there are no active tasks remaining
        let mut stmt = conn.prepare(sql)?;
        let rows_after = stmt.query(())?.count()?;
        stmt.finalize()?;

        println!(
            "Expected number of active tasks: {}, Returned number of active tasks: {}",
            0, rows_after
        );
        assert_eq!(0, rows_after);

        teardown(conn);

        Ok(())
    }

    #[test]
    fn test_list_active() -> rusqlite::Result<()> {
        let conn = setup()?;

        // Create active tasks
        let tasks_to_start = vec!["Task_A", "Task_B", "Task_C", "Task_D", "Task_E"];
        for task in &tasks_to_start {
            let task = task.to_string();
            // Ensure category is correct in output
            let category = format!("Category_{}", task);
            start_task(&conn, &task, Some(&category))?;
        }

        end_task(&conn, &String::from("Task_B"))?;
        end_task(&conn, &String::from("Task_D"))?;

        let expected_active = tasks_to_start.len() - 2;
        let num_active = match list_active(&conn, Filter::All)? {
            List(active) => active,
            _ => panic!("Unexpected enum returned from list_active call."),
        };

        println!(
            "Expected active tasks: {} Returned active tasks: {}",
            expected_active, num_active
        );
        assert_eq!(expected_active, num_active);

        teardown(conn);

        Ok(())
    }

    #[test]
    fn test_list_complete() -> rusqlite::Result<()> {
        let conn = setup()?;

        // Create active tasks
        let tasks_to_start = vec!["Task_A", "Task_B", "Task_C", "Task_D", "Task_E"];
        for task in &tasks_to_start {
            let task = task.to_string();
            // Ensure category is correct in output
            let category = format!("Category_{}", task);
            start_task(&conn, &task, Some(&category))?;
        }

        // Complete 2 tasks
        end_task(&conn, &String::from("Task_B"))?;
        end_task(&conn, &String::from("Task_D"))?;

        let expected_complete = 2usize;
        let num_complete = match list_complete(&conn, Filter::All)? {
            List(comlete) => comlete,
            _ => panic!("Unexpected enum returned from list_all call."),
        };

        println!(
            "Expected complete tasks {}, Returned complete tasks {}",
            expected_complete, num_complete
        );
        assert_eq!(expected_complete, num_complete);

        teardown(conn);

        Ok(())
    }

    #[test]
    fn test_list_all() -> rusqlite::Result<()> {
        let conn = setup()?;

        // Create active tasks
        let tasks_to_start = vec!["Task_A", "Task_B", "Task_C", "Task_D", "Task_E"];
        for task in &tasks_to_start {
            let task = task.to_string();
            // TODO Ensure category is correct in output
            let category = format!("Category_{}", task);
            start_task(&conn, &task, Some(&category))?;
        }

        // Complete 2 tasks
        end_task(&conn, &String::from("Task_B"))?;
        end_task(&conn, &String::from("Task_D"))?;

        let num_tasks = match list_all(&conn, Filter::All)? {
            List(all_tasks) => all_tasks,
            _ => panic!("Unexpected enum returned from list_all call."),
        };

        println!(
            "Expected total tasks: {}, Returned total tasks: {}",
            tasks_to_start.len(),
            num_tasks
        );
        assert_eq!(tasks_to_start.len(), num_tasks);

        teardown(conn);

        Ok(())
    }

    #[test]
    fn test_sum_task_times() -> rusqlite::Result<()> {
        // Not sure exactly what to test here.
        // Think confirmation of the enum returned should be fine as a first pass
        let conn = setup()?;

        let mut stmt = conn.prepare(
            "INSERT INTO tasks \
        (name, start_time, end_time, total_time, category, status) VALUES \
        (?1, ?2, ?3, ?4, ?5, 'Complete')",
        )?;
        fn add_completed_tasks(
            stmt: &mut Statement,
            task_name: &str,
            duration: i64,
            category: &str,
        ) -> rusqlite::Result<()> {
            let start_time = Local::now().timestamp();
            let _ = stmt.execute(params![
                task_name,
                start_time,
                start_time + duration,
                duration,
                category
            ])?;
            Ok(())
        }

        add_completed_tasks(&mut stmt, "Task A", 300, "Category A")?;
        add_completed_tasks(&mut stmt, "Task B", 65, "Category B")?;
        add_completed_tasks(&mut stmt, "Task C", 1800, "Category A")?;
        add_completed_tasks(&mut stmt, "Task D", 4500, "Category B")?;
        add_completed_tasks(&mut stmt, "Misc Task", 600, "Misc")?;

        stmt.finalize()?;

        let total_enum = sum_task_times(&conn, Filter::All, None)?;

        assert_eq!(SumTaskTimes, total_enum);

        teardown(conn);

        Ok(())
    }

    fn filter_test_helper(connection: &Connection) -> rusqlite::Result<()> {
        let mut stmt = connection.prepare(
            "INSERT INTO tasks \
        (name, start_time, category, status) VALUES \
        (?1, ?2, ?3, 'Active')",
        )?;
        fn add_active_tasks(
            stmt: &mut Statement,
            task_name: &str,
            start_time: i64,
            category: &str,
        ) -> rusqlite::Result<()> {
            let _ = stmt.execute(params![task_name, start_time, category])?;
            Ok(())
        }

        let now = Local::now();

        add_active_tasks(&mut stmt, "First Task", 10, "Misc")?;
        add_active_tasks(
            &mut stmt,
            "Over a Year",
            (now - TimeDelta::days(400)).timestamp(),
            "Category A",
        )?;
        add_active_tasks(
            &mut stmt,
            "Within a Year",
            (now - TimeDelta::days(300)).timestamp(),
            "Category B",
        )?;
        add_active_tasks(
            &mut stmt,
            "SemiAnnual",
            (now - TimeDelta::weeks(23)).timestamp(),
            "Misc",
        )?;
        add_active_tasks(
            &mut stmt,
            "Quarter",
            (now - TimeDelta::weeks(10)).timestamp(),
            "Category A",
        )?;
        add_active_tasks(
            &mut stmt,
            "Month",
            (now - TimeDelta::days(25)).timestamp(),
            "Category B",
        )?;
        add_active_tasks(
            &mut stmt,
            "Week",
            (now - TimeDelta::days(5)).timestamp(),
            "Misc",
        )?;
        add_active_tasks(
            &mut stmt,
            "Day",
            (now - TimeDelta::seconds(3600 * 22)).timestamp(),
            "Category A",
        )?;
        stmt.finalize()?;

        Ok(())
    }
    #[test]
    fn test_filtered_results() -> rusqlite::Result<()> {
        let conn = setup()?;

        filter_test_helper(&conn)?;

        for filter in Filter::iter() {
            let List(num_returned) = list_all(&conn, filter.clone())? else {
                unreachable!()
            };

            let expected_num: usize = match filter {
                Filter::Day => 1,
                Filter::Week => 2,
                Filter::Month => 3,
                Filter::Quarter => 4,
                Filter::SemiAnnual => 5,
                Filter::Year => 6,
                Filter::All => 8,
            };

            assert_eq!(expected_num, num_returned);
        }

        teardown(conn);

        Ok(())
    }
}
