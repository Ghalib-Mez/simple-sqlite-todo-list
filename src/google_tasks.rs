// src/google_tasks.rs
use reqwest::Client;
use serde::{Deserialize, Serialize, Deserializer}; // Import Deserializer
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskList {
    pub id: Option<String>,
    pub title: Option<String>,
}

// Custom deserialization for the 'due' field to ensure it's always a String.
// This function will be called by serde to parse the 'due' field.
// It directly attempts to deserialize into an Option<String>, bypassing any
// potentially strict date/time parsing logic from other crates.
fn deserialize_due_as_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // Simply try to deserialize the value as an Option<String>.
    // This will work regardless of the exact timestamp format, as long as it's a string.
    Option::deserialize(deserializer)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskItem {
    pub id: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    // For updating a task, the API expects 'due' in RFC3339 format,
    // so keeping it as String and ensuring our custom deserializer handles incoming is good.
    #[serde(default, deserialize_with = "deserialize_due_as_string")]
    pub due: Option<String>,
}


pub struct GoogleTasks {
    client: Client,
    token: String,
}

impl GoogleTasks {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    // List tasklists
    pub async fn list_tasklists(&self) -> Result<Vec<TaskList>, Box<dyn Error>> {
        let resp = self
            .client
            .get("https://tasks.googleapis.com/tasks/v1/users/@me/lists")
            .bearer_auth(&self.token)
            .send()
            .await?;

        #[derive(Debug, Deserialize)]
        struct TaskListsResponse {
            items: Vec<TaskList>,
        }

        let tasklists_response: TaskListsResponse = resp.json().await?;
        Ok(tasklists_response.items)
    }

    // Create a new tasklist
    pub async fn create_tasklist(&self, title: &str) -> Result<TaskList, Box<dyn Error>> {
        let new_list = TaskList {
            id: None,
            title: Some(title.to_string()),
        };
        let resp = self
            .client
            .post("https://tasks.googleapis.com/tasks/v1/users/@me/lists")
            .bearer_auth(&self.token)
            .json(&new_list)
            .send()
            .await?;
        let tasklist = resp.json::<TaskList>().await?;
        Ok(tasklist)
    }

    // List tasks from a tasklist
    pub async fn list_tasks(&self, tasklist_id: &str) -> Result<Vec<TaskItem>, Box<dyn Error>> {
        let url = format!("https://tasks.googleapis.com/tasks/v1/lists/{}/tasks", tasklist_id);
        let resp = self.client.get(&url).bearer_auth(&self.token).send().await?;
        let json: Value = resp.json().await?; // Use Value for flexibility, then deserialize

        #[derive(Debug, Deserialize)]
        struct TasksResponse {
            #[serde(default)] // Default to empty vec if 'items' is missing
            items: Vec<TaskItem>,
        }

        // Handle cases where 'items' might be completely absent from the JSON
        let tasks_response: TasksResponse = serde_json::from_value(json).unwrap_or_else(|_| TasksResponse { items: vec![] });
        Ok(tasks_response.items)
    }


    // Create a task in a tasklist
    pub async fn create_task(
        &self,
        tasklist_id: &str,
        task: TaskItem,
    ) -> Result<TaskItem, Box<dyn Error>> {
        let url = format!(
            "https://tasks.googleapis.com/tasks/v1/lists/{}/tasks",
            tasklist_id
        );
        let resp = self
            .client.post(&url)
            .bearer_auth(&self.token)
            .json(&task)
            .send()
            .await?;
        let task_created = resp.json::<TaskItem>().await?;
        Ok(task_created)
    }

    // NEW: Delete a task from a tasklist
    pub async fn delete_task(
        &self,
        tasklist_id: &str,
        task_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "https://tasks.googleapis.com/tasks/v1/lists/{}/tasks/{}",
            tasklist_id, task_id
        );
        self.client
            .delete(&url)
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?; // Check if the response was successful (2xx)
        Ok(())
    }

    // NEW: Update a task in a tasklist
    pub async fn update_task(
        &self,
        tasklist_id: &str,
        task_id: &str,
        task_update: TaskItem, // Accepts a TaskItem with updated fields
    ) -> Result<TaskItem, Box<dyn Error>> {
        let url = format!(
            "https://tasks.googleapis.com/tasks/v1/lists/{}/tasks/{}",
            tasklist_id, task_id
        );
        let resp = self
            .client
            .put(&url) // Use PUT for full replacement, PATCH for partial (PUT is simpler here)
            .bearer_auth(&self.token)
            .json(&task_update)
            .send()
            .await?;
        let updated_task = resp.json::<TaskItem>().await?;
        Ok(updated_task)
    }
}