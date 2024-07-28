use std::{fmt::Display, fs, path::PathBuf};

use anyhow::{bail, Result};
use chrono::Local;
use config::Config;
use reqwest::{Error, Method};
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG: &str = include_str!("../config.example.toml");

#[derive(Debug, Deserialize, Clone)]
pub struct Staff {
    pub id: i32,
    pub name: String,
}

impl Display for Staff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.id)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Customer {
    pub id: i32,
    pub nom: String,
    pub name2: String,
    pub prospect: bool
}

impl Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} [{}]", self.nom, self.name2, if self.prospect { "Prospect" } else { "Client" })
    }
}

#[derive(Debug, Deserialize)]
pub struct AceticsConfig {
    pub endpoint: String,
    pub token: String,
    pub default_staff_index: usize,
    pub staffs: Vec<Staff>,
}

pub struct Acetics {
    config: AceticsConfig,
    client: reqwest::Client,
}

impl Acetics {
    fn config_dir() -> PathBuf {
        dirs::config_dir().unwrap().join("acetics-cli")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load_config() -> Result<Self> {
        let config_path = Self::config_path();

        let settings = Config::builder()
            .add_source(config::File::from(config_path.clone()))
            // .add_source(config::File::with_name("config.toml"))
            .add_source(config::Environment::with_prefix("ACETICS"))
            .build();

        match settings {
            Ok(settings) => {
                let settings = settings.try_deserialize::<AceticsConfig>()?;

                Ok(Self {
                    config: settings,
                    client: reqwest::Client::new(),
                })
            }
            Err(e) => {
                if config_path.exists() {
                    Err(e.into())
                } else {
                    fs::create_dir_all(Self::config_dir())?;

                    std::fs::write(&config_path, DEFAULT_CONFIG)?;

                    bail!(
                        "Please edit the config file at {} and restart the application",
                        config_path.display()
                    );
                }
            }
        }
    }

    pub async fn json_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        path: &str,
        body: &T,
    ) -> Result<R> {
        let url = format!("{}/{}", self.config.endpoint, path);

        Ok(self
            .client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?
            .json()
            .await?)
    }

    pub fn staffs(&self) -> &[Staff] {
        &self.config.staffs
    }

    pub fn default_staff_index(&self) -> usize {
        self.config.default_staff_index
    }

    pub fn is_default_staff(&self, staff: &Staff) -> bool {
        staff.id == self.config.staffs[self.config.default_staff_index].id
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
    pub fn new(task_type: TaskType, title: String, description: String) -> Self {
        Self {
            fk_type: task_type as u8,

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

    pub fn with_assigned_staff(mut self, staff: Staff) -> Self {
        self.fk_assigned_staff = Some(staff.id);
        self
    }

    pub fn with_customer(mut self, customer: Option<Customer>) -> Self {
        if let Some(customer) = customer {
            self.fk_customer = Some(customer.id);
        } else {
            self.fk_customer = None;
        }
        self
    }
}
