use iced::widget::{
    button, column, container, horizontal_space, row, text, text_input, vertical_space, Column, Row,
};
use iced::{Element, Length, Sandbox, Settings};

fn main() -> iced::Result {
    Metronome::run(Settings::default())
}
#[derive(Debug, Clone)]
enum Page {
    Start,
    ActiveList,
    EndLast,
}

#[derive(Debug, Clone, Default)]
struct Task {
    name: String,
    category: String,
}
struct Metronome {
    page: Page,
    new_task_info: Task,
    num_tasks_active: i64,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeTab(Page),
    EditTaskName(String),
    EditCategoryName(String),
    StartNewTask,
    EndLastTask,
}

impl Sandbox for Metronome {
    type Message = Message;

    fn new() -> Self {
        Self {
            page: Page::Start,
            new_task_info: Task::default(),
            num_tasks_active: 0,
        }
    }

    fn title(&self) -> String {
        String::from("Metronome")
    }

    fn update(&mut self, message: Self::Message) {
        let conn = rusqlite::Connection::open(metronome_core::set_database_location())
            .expect("Could not open database");
        match message {
            Message::ChangeTab(page) => self.page = page,
            Message::EditTaskName(task) => self.new_task_info.name = task,
            Message::EditCategoryName(category) => self.new_task_info.category = category,
            Message::StartNewTask => {
                let category = if self.new_task_info.category.is_empty() {
                    None
                } else {
                    Some(&self.new_task_info.category)
                };
                metronome_core::start_task(&conn, &self.new_task_info.name, category).expect("");
                self.num_tasks_active += 1;
                self.new_task_info = Task::default();
            }
            Message::EndLastTask => {
                metronome_core::end_last(&conn).expect("");
                self.num_tasks_active -= 1;
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let tabs: Row<Message> = row![
            button("Start").on_press(Message::ChangeTab(Page::Start)),
            button("Active Tasks").on_press(Message::ChangeTab(Page::ActiveList)),
            button("End").on_press(Message::ChangeTab(Page::EndLast)),
            horizontal_space()
        ];

        // Define elements by page
        let page_contents = match self.page {
            Page::Start => container(column![
                text("Start a new task."),
                text_input("Task Name", &self.new_task_info.name)
                    .on_input(Message::EditTaskName)
                    .on_submit(Message::StartNewTask),
                text_input("Category (Optional)", &self.new_task_info.category)
                    .on_input(Message::EditCategoryName),
                button("Start Task").on_press(Message::StartNewTask)
            ]),
            Page::ActiveList => container(column![text("On the active list page"),]),
            Page::EndLast => container(column![
                button("End last task").on_press(Message::EndLastTask)
            ]),
        };

        let page_contents = page_contents
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20);

        let contents: Column<Message> = column![tabs, page_contents,].into();

        container(contents)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(20)
            .into()
    }
}
