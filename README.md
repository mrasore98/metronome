# Metronome 

Metronome is a time-tracking application for the command line. [Metronomes](https://en.wikipedia.org/wiki/Metronome) keep time for musicians, and now you can keep track of your time by using Metronome to track your completed tasks!

## Features

- Organize tasks into categories
- List tasks using pre-set time filters or by task status
- Total task times by category with option to apply time filters

## Commands

```
  start  Start a new task.
  end    End an existing task.
  list   Display a list of tasks.
  total  Sum the amount of time spent on your tasks.
  help   Print this message or the help of the given subcommand(s)
```

## Usage

### Starting tasks

Start a new task:
```
Usage: metronome.exe start [OPTIONS] <task>

Arguments:
  <task>  Name of the task to start

Options:
  -c, --category <category>  Specify a category for the new task.
  -h, --help                 Print help

```

#### Examples

**Start a task called "My Task" with category "My category":**

Input:
```
metronome start "My Task" -c "My category"
```

Output:
```
Task "My Task" started at Mon Apr 22 16:48:03 2024!
```

### Ending tasks

```
Usage: metronome.exe end [OPTIONS] [task]

Arguments:
  [task]  Name of the task to end.

Options:
  -l, --last  Ends the active task that was started most recently.
      --all   Ends all active tasks. Overrides a task name if one is given.
  -h, --help  Print help

```

#### Examples

**End a task called "My Task":**

Input:
```
metronome end "My Task"
```

Output:
```
Ending task "My Task" at Mon Apr 22 16:50:22 2024
Task "My Task" ended after 0h 2m 19s
```

**End the last started started task:**
```
metronome end --last
```

**End all active tasks:**

Input:
```
metronome end --all
```

Output:
```
Ended 3 active tasks at Mon Apr 22 16:51:21 2024.
```

### Listing tasks

```
Usage: metronome.exe list [OPTIONS]

Options:
  -a, --active           List the active tasks.
  -c, --complete         List the completed tasks.
      --all              List all tasks.
  -f, --filter <filter>  Apply a time range filter to the list of tasks. [possible values: d, day, w, week, m, month, q, quarter, s, semi, semiannual, y, year]
  -h, --help             Print help
```

#### Examples

**List all active tasks started in the last week:**

Input:
```
metronome list --complete -f week
```

Output:
```
|  ID  |                   TASK                   |           START TIME           |            END TIME            |   TOTAL TIME    |       CATEGORY       |
==============================================================================================================================================================
|  7   |                   Week                   |    Wed Apr 17 23:51:21 2024    |              NULL              |      NULL       |         Misc         |
|  8   |                   Day                    |    Mon Apr 22 01:51:21 2024    |              NULL              |      NULL       |      Category A      |
```

### Totaling task times

```
Usage: metronome.exe total [OPTIONS]

Options:
  -f, --filter <filter>      Only total tasks within the time range specified by a filter. [possible values: d, day, w, week, m, month, q, quarter, s, semi, semiannual, y, year]   
  -c, --category <category>  Only total tasks in specified categories.
  -h, --help                 Print help
```

#### Examples

**Totaling task times for events started in the last week:**

Input: 
```
metronome total -f w
```

Output:
```
|       CATEGORY       |   TOTAL TIME    |  PERCENTAGE  |
=========================================================
|      Category B      |    1h 16m 5s    |    62.84     |
|      Category A      |    0h 35m 0s    |    28.91     |
|         Misc         |    0h 10m 0s    |     8.26     |
=========================================================
|        TOTAL         |    2h 1m 5s     |    100.00    |
```








