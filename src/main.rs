mod google_tasks;

use std::{collections::HashMap, io};
use std::error::Error;
use async_trait::async_trait;
use hyper::body::HttpBody;
use crate::google_tasks::{GoogleTasks, TaskItem, TaskList};

// --- START: CUSTOM DESERIALIZATION FOR YUP-OAUTH2 ---
// This is required because yup-oauth2's internal 'time::OffsetDateTime' deserialization
// is strict about fractional seconds and the Google API might return varying precision.
// We need to provide a custom function that `yup-oauth2` will

// This line must be present to fulfill the 'custom-serde' feature requirement.
// It tells yup-oauth2 to use the deserialization provided by `setup_time_serde!`.
// If you put this in a different module, make sure it's accessible or placed correctly.
// For simplicity, placing it at the top of main.rs or a dedicated module is fine.
// The exact placement sometimes depends on your module structure, but near the top
// of main.rs (after `use` statements) is usually safe.
//
// If you face a `multiple-definitions-for-crate-level-item` error,
// move `yup_oauth2::setup_time_serde!();` outside of `main` function.
// It's a macro that defines functions, so it needs to be at module level.
// --- END: CUSTOM DESERIALIZATION FOR YUP-OAUTH2 ---


#[derive(Debug)]
enum TodoError {
    NotFound,
    StorageError,
}

#[async_trait]
trait TodoStore: Send + Sync {
    async fn add_item(&mut self, title: String, content: String) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn list_items(&self) -> Result<Vec<TodoItem>, Box<dyn Error + Send + Sync>>;
    async fn remove_item(&mut self, title: String) -> Result<(), TodoError>;
    async fn find_by_title(&self, title: &str) -> Option<TodoItem>;
    async fn complete_item(&mut self, title: &str) -> Result<(), TodoError>;
}

#[derive(Debug, PartialEq, Clone)]
enum Status {
    Todo,
    Done,
}

#[derive(Debug, Clone)]
struct TodoItem {
    id: String,
    title: String,
    content: String,
    status: Status,
    due: Option<String>, // Keep as string
}

// Helper for printing
trait Summary {
    fn summarize(&self) -> String;
}

impl Summary for TodoItem {
    fn summarize(&self) -> String {
        let checkbox = if self.status == Status::Done { "[X]" } else { "[ ]" };
        let due_str = self.due.as_deref().unwrap_or("No due date");
        format!("{} {}: {} (Due: {})", checkbox, self.title, self.content, due_str)
    }
}

// ---------------------- Google Tasks store ----------------------
struct TodoStoreGTask {
    tasks_api: GoogleTasks,
    id: String,
}

impl TodoStoreGTask {
    /// Initialize Google Tasks store
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        // Load OAuth secret
        let secret = yup_oauth2::read_application_secret("../target/rust_oauth.json").await?;
        let auth = yup_oauth2::InstalledFlowAuthenticator::builder(secret, yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk("tokencache.json")
            .build()
            .await?;
        // Fetch access token
        let token = auth.token(&["https://www.googleapis.com/auth/tasks"]).await?;
         let access_token = token.token().unwrap().to_string();

        let tasks_api = GoogleTasks::new(access_token);

        // Try to find an existing tasklist named "My Rust Tasks"
        let existing_lists = tasks_api.list_tasklists().await?;

        let tasklist = if let Some(found) = existing_lists
            .into_iter()
            .find(|x| x.title.as_deref() == Some("My Rust Tasks"))
        {
            found
        } else {
            tasks_api.create_tasklist("My Rust Tasks").await?
        };


        Ok(Self {
            tasks_api,
            id: tasklist.id.unwrap(),
        })
    }
}
#[async_trait]
impl TodoStore for TodoStoreGTask {
    /// Add a new task
    async fn add_item(&mut self, title: String, content: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let task = TaskItem {
            title: Some(title),
            notes: Some(content),
            status: Some("needsAction".to_string()),
            id: None,
            due: None,
        };
        self.tasks_api.create_task(&self.id, task).await.expect("Failed to create task");
        Ok(())
    }

    /// List all tasks
    async fn list_items(&self) -> Result<Vec<TodoItem>, Box<dyn Error + Send + Sync>> {
        let tasks = self.tasks_api.list_tasks(&self.id).await.expect("Failed to list tasks");

        let todo_items = tasks.into_iter().map(|t| {
            let status = match t.status.as_deref() {
                Some("completed") => Status::Done,
                _ => Status::Todo,
            };
            TodoItem {
                id: t.id.unwrap_or_default(),
                title: t.title.unwrap_or_default(),
                content: t.notes.unwrap_or_default(),
                status,
                due: t.due,
            }
        }).collect();

        Ok(todo_items)
    }

    async fn remove_item(&mut self, title: String) -> Result<(), TodoError> {
        if let Some(item_to_remove) = self.find_by_title(&title).await {
            let task_id = item_to_remove.id;
            self.tasks_api.delete_task(&self.id, &task_id)
                .await
                .map_err(|_| TodoError::NotFound)?;
            Ok(())
        } else {
            Err(TodoError::NotFound)
        }
    }

    async fn find_by_title(&self, title: &str) -> Option<TodoItem> {
        let tasks = self.tasks_api.list_tasks(&self.id).await.ok()?;

        let matching_task = tasks.into_iter().find(|item| item.title.as_deref() == Some(title));

        matching_task.map(|t| {
            let status = match t.status.as_deref() {
                Some("completed") => Status::Done,
                _ => Status::Todo,
            };
            TodoItem {
                id: t.id.unwrap_or_default(),
                title: t.title.unwrap_or_default(),
                content: t.notes.unwrap_or_default(),
                status,
                due: t.due,
            }
        })
    }

    async fn complete_item(&mut self, title: &str) -> Result<(), TodoError> {
        if let Some(mut item_to_complete) = self.find_by_title(title).await {
            item_to_complete.status = Status::Done;

            let task_update = TaskItem {
                id: Some(item_to_complete.id.clone()),
                title: Some(item_to_complete.title),
                notes: Some(item_to_complete.content),
                status: Some("completed".to_string()),
                due: item_to_complete.due,
            };

            self.tasks_api.update_task(&self.id, &item_to_complete.id, task_update)
                .await
                .map_err(|_| TodoError::NotFound)?;
            Ok(())
        } else {
            Err(TodoError::NotFound)
        }
    }
}


// ---------------------- Main ----------------------
#[tokio::main]
async fn main() {
    println!("TODO CLI");

    // Initialize Google Tasks store
    let store = match TodoStoreGTask::new().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to initialize Google Tasks store: {}", e);
            return;
        }
    };

    let mut todo_store: Box<dyn TodoStore + Send + Sync> = Box::new(store);

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let args: Vec<&str> = input.trim().split_whitespace().collect();
        if args.is_empty() { continue; }

        match args[0] {
            "add" => {
                if args.len() < 3 {
                    println!("Usage: add <title> <content>");
                    continue;
                }
                let title = args[1].to_string();
                let content = args[2..].join(" ");
                todo_store.add_item(title, content).await.unwrap();
                println!("Added task.");
            }
            "complete" => {
                if args.len() < 2 {
                    println!("Usage: complete <title>");
                    continue;
                }
                let title = args[1].to_string();
                todo_store.complete_item(title.as_str()).await.unwrap();
                println!("Completed task.");
            }
            "delete" => {
                if args.len() < 2 {
                    println!("Usage: delete <title>");
                    continue;
                }
                let title = args[1].to_string();
                todo_store.remove_item(title).await.unwrap();
                println!("Deleted task.");
            }
            "list" => {
                let items = todo_store.list_items().await.unwrap();
                println!("--- TODO List ---");
                for item in items {
                    println!("{}", item.summarize());
                }
                println!("-----------------");
            }
            "quit" => break,
            _ => println!("Unknown command"),
        }
    }
}
