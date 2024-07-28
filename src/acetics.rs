use std::fmt::Display;

use chrono::{Duration, Local, NaiveDateTime};
use reqwest::{Error, Method, Response};
use serde::{Deserialize, Serialize};
use config::Config;

#[derive(Debug, Deserialize)]
pub struct AceticsConfig {
    pub endpoint: String,
    pub token: String,
}

pub struct Acetics {
    config: AceticsConfig,
    client: reqwest::Client,
}

impl Acetics {
    pub fn load_config() -> Result<Self, Box<dyn std::error::Error>> {
        let settings = Config::builder()
            .add_source(config::File::with_name("config.toml"))
            .add_source(config::Environment::with_prefix("ACETICS"))
            .build()?;

        let settings = settings.try_deserialize::<AceticsConfig>()?;

        Ok(Self {
            config: settings,
            client: reqwest::Client::new(),
        })
    }

    pub async fn json_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        path: &str,
        body: &T,
    ) -> Result<R, Error> {
        let url = format!("{}/{}", self.config.endpoint, path);

        self.client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?
            .json()
            .await
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskStatus {
    Ongoing,
    Closed,
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Ongoing => write!(f, "En cours"),
            TaskStatus::Closed => write!(f, "Terminé"),
        }
    }
}

pub enum TaskType {
    CustomerCall = 1,
    Technical = 2,
    Administrative = 3,
    Reminder = 4,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    fk_type: u8,
    fk_assigned_staff: Option<i32>,
    fk_assigned_group: Option<i32>,
    fk_customer: Option<i32>,
    fk_contract: Option<i32>,

    title: String,
    description: String,

    due_date: chrono::NaiveDateTime,
    work_time: Option<String>,
    estimated_time: Option<String>,

    priority: TaskPriority,
    status: TaskStatus,
}

impl Task {
    pub fn new(taskType: TaskType, title: String, description: String) -> Self {
        Self {
            fk_type: taskType as u8,

            fk_assigned_staff: Some(8),
            fk_assigned_group: None,
            fk_customer: None,
            fk_contract: None,

            title,
            description,

            due_date: Local::now().naive_local(),
            work_time: None,
            estimated_time: None,

            priority: TaskPriority::Normal,
            status: TaskStatus::Ongoing,
        }
    }

    pub fn with_due_date(mut self, due_date: chrono::NaiveDateTime) -> Self {
        self.due_date = due_date;
        self
    }

    pub fn with_work_time(mut self, work_time: Option<String>) -> Self {
        self.work_time = work_time;
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_estimated_time(mut self, estimated_time: Option<String>) -> Self {
        self.estimated_time = estimated_time;
        self
    }
}
