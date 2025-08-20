use std::{
    collections::HashMap,
    io::{self},
};
use uuid::Uuid;

#[derive(Debug)]
enum TodoError {
    NotFound,
    StorageError,
}
trait TodoStore {
    fn add_item(&mut self, title: String, content: String) -> ();
    fn list_items(&self) -> Result<Vec<TodoItem>, TodoError>;
    fn remove_item(&mut self, title: String) -> Result<(), TodoError>;
    fn find_by_title(&self, title: &str) -> Option<TodoItem>;
    fn complete_item(&mut self, id: &str) -> Result<(), TodoError>;
}

#[derive(Debug, PartialEq)]
enum Status {
    Todo,
    InProgress,
    Done,
}
#[derive(Debug)]
struct TodoItem {
    id: String,
    title: String,
    content: String,
    status: Status,
}

struct TodoStoreInMemory {
    items: HashMap<String, TodoItem>,
}
struct TodoStoreSqlite {
    sql_lite_connection: sqlite::Connection,
}

impl TodoStoreInMemory {
    fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}
trait Summary {
    fn summarize(&self) -> String;
}

impl Summary for TodoItem {
    fn summarize(&self) -> String {
        // In Rust, `if/else` is an expression that returns a value.
        // We can assign its result to a variable.
        let checkbox = if self.status == Status::Done {
            "[X]" // Value if true
        } else {
            "[ ]" // Value if false
        };

        format!("{}: {}: {}", checkbox, self.title, self.content)
    }
}
impl TodoStoreSqlite {
    pub fn new(db_path: &str) -> Self {
        let connection = sqlite::open(db_path).unwrap();

        // ensure table exists
        connection
            .execute(
                "
            CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT,
                status TEXT NOT NULL CHECK(status IN ('Todo', 'InProgress', 'Done')) DEFAULT 'Todo'
            );
        ",
            )
            .unwrap();

        TodoStoreSqlite {
            sql_lite_connection: connection,
        }
    }
}

impl TodoStore for TodoStoreSqlite {
    fn add_item(&mut self, title: String, content: String) {
        let query = "INSERT INTO todos (title, content, status) VALUES (?, ?, 'Todo')";
        let mut statement = self.sql_lite_connection.prepare(query).unwrap();
        statement.bind((1, title.as_str())).unwrap();
        statement.bind((2, content.as_str())).unwrap();
        statement.next().unwrap();
    }
    fn remove_item(&mut self, title: String) -> Result<(), TodoError> {
        let query = "DELETE FROM todos WHERE title = ?";
        let mut statement = self.sql_lite_connection.prepare(query).unwrap();
        statement.bind((1, title.as_str())).unwrap();

        statement.next().unwrap(); // executes the delete

        Ok(())
    }

    fn complete_item(&mut self, id: &str) -> Result<(), TodoError> {
        let query = "UPDATE todos SET status = ? WHERE id = ?";
        let mut statement = self.sql_lite_connection.prepare(query).unwrap();

        let id_num: i64 = id.parse().map_err(|_| TodoError::NotFound)?;
        statement.bind((1, "Done")).unwrap();
        statement.bind((2, id_num)).unwrap();

        statement.next().unwrap(); // run the update

        // check rows actually updated
        if self.sql_lite_connection.change_count() == 0 {
            Err(TodoError::NotFound)
        } else {
            Ok(())
        }
    }

    fn find_by_title(&self, title: &str) -> Option<TodoItem> {
        let query = "SELECT id, title, content, status FROM todos WHERE title = ?";
        let mut statement = self.sql_lite_connection.prepare(query).unwrap();
        statement.bind((1, title)).unwrap();

        if let sqlite::State::Row = statement.next().unwrap() {
            let id: i64 = statement.read(0).unwrap();
            let title: String = statement.read(1).unwrap();
            let content: String = statement.read(2).unwrap_or_default();
            let status_str: String = statement.read(3).unwrap();

            let status = match status_str.as_str() {
                "Todo" => Status::Todo,
                "InProgress" => Status::InProgress,
                "Done" => Status::Done,
                _ => Status::Todo, // fallback just in case
            };

            Some(TodoItem {
                id: id.to_string(),
                title,
                content,
                status,
            })
        } else {
            None
        }
    }

    fn list_items(&self) -> Result<Vec<TodoItem>, TodoError> {
        let query = "SELECT id, title, content, status FROM todos";
        let mut statement = self.sql_lite_connection.prepare(query).unwrap();

        let mut items = Vec::new();

        while let sqlite::State::Row = statement.next().unwrap() {
            let id: i64 = statement.read(0).unwrap();
            let title: String = statement.read(1).unwrap();
            let content: String = statement.read::<String, _>(2).unwrap_or_default();
            let status_str: String = statement.read(3).unwrap();

            let status = match status_str.as_str() {
                "Todo" => Status::Todo,
                "InProgress" => Status::InProgress,
                "Done" => Status::Done,
                _ => Status::Todo, // fallback
            };

            items.push(TodoItem {
                id: id.to_string(),
                title,
                content,
                status,
            });
        }

        if items.is_empty() {
            Err(TodoError::NotFound)
        } else {
            Ok(items)
        }
    }
}
/**
impl TodoStore for TodoStoreInMemory {
    fn add_item(&mut self, title: String, content: String) {
        let todo_item = TodoItem {
            id: Uuid::new_v4().to_string(),
            content,
            title,
            status: Status::Todo,
        };
        self.items.insert(todo_item.id.clone(), todo_item);
    }
    fn list_items(&self) -> Result<Vec<TodoItem>, TodoError> {
        if self.items.is_empty() {
            Err(TodoError::NotFound)
        } else {
            Ok(self.items.values().collect())
        }
    }

    fn find_by_title(&self, title: &str) -> Option<TodoItem> {
        let lowercased_title = title.to_lowercase();

        self.items
            .values()
            // .find() already returns Option<&TodoItem>, which is exactly what we need.
            .find(|item| item.title.to_lowercase() == lowercased_title)
    }
    fn complete_item(&mut self, id: &str) -> Result<(), TodoError> {
        if let Some(item) = self.items.get_mut(id) {
            item.status = Status::Done;
            Ok(())
        } else {
            Err(TodoError::NotFound)
        }
    }
} */

fn main() {
    println!("TODO CLI");
    let mut todo_store: Box<dyn TodoStore> = Box::new(TodoStoreSqlite::new("todos.db"));

    loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed To parse Response");

        let args: Vec<String> = shlex::split(&input.trim()).unwrap_or_default();

        // shlex gives us owned Strings, so we convert them to &str for slicing
        let args_str: Vec<&str> = args.iter().map(AsRef::as_ref).collect();

        if args_str.is_empty() {
            println!("No command given, please try again.");
            continue;
        }

        // We now use our new args_str vector
        let command = args_str[0];
        let params = &args_str[1..];

        match command {
            "add" => {
                if params.len() < 2 {
                    println!("Usage: add <title> <content>");
                } else {
                    // The first parameter is always the title
                    let title = params[0].to_string();

                    // Take all the rest of the parameters (from index 1 to the end)
                    // and join them together with spaces to form the content.
                    let content = params[1..].join(" ");
                    todo_store.add_item(title.clone(), content);
                    println!("Added: {}", title);
                }
            }
            "complete" => {
                if params.len() != 1 {
                    println!("Usage: complete \"<title to TodoItem for completion>\"");
                } else {
                    let title_to_find = params[0];
                    let found_items = todo_store.find_by_title(title_to_find);
                    match found_items {
                        Some(item) => {
                            let item_id = item.id.clone(); // or item.id if it's Copy
                            let result = todo_store.complete_item(&item_id);
                            match result {
                                Ok(()) => {
                                    println!("Completed Succesfully");
                                }
                                Err(e) => {
                                    println!("Failed to complete item: {:?}", e);
                                }
                            }
                        }
                        None => {
                            // handle case where item not found
                        }
                    }
                }
            }
            "remove" => {
                if params.len() < 1 {
                    println!("Usage: remove <title> ");
                } else {
                    let title: String = params[0].to_owned();

                    match todo_store.remove_item(title) {
                        Ok(()) => {
                            println!("Removed Succesfully")
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            }
            "list" => {
                if params.len() > 1 {
                    println!("Usage: list ");
                } else {
                    match todo_store.list_items() {
                        Ok(items) => {
                            println!("--- TODO List ---");
                            for item in items {
                                println!("- {}", item.summarize());
                            }
                            println!("-----------------");
                        }
                        Err(TodoError::NotFound) => {
                            println!("Your TODO list is empty. Use 'add' to create one.");
                        }
                        Err(TodoError::StorageError) => {
                            println!("A storage error occurred.");
                        }
                    }
                }
            }
            "quit" => {
                println!("Quit");
                break;
            }
            _ => println!("Unknown command"),
        }
    }
}
