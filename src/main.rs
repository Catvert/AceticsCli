mod acetics;

use chrono::{Duration, Local, NaiveDate, Timelike};
use inquire::{
    formatter::DEFAULT_DATE_FORMATTER,
    ui::{Color, RenderConfig, Styled},
    validator::Validation,
    Confirm, CustomType, Editor, Select, Text,
};

use acetics::{Acetics, Staff, Task, TaskStatus, TaskType};
use reqwest::Method;
use std::{ffi::OsStr, io::Write, ops::Add};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    clear_terminal();

    let acetics = Acetics::load_config()?;

    create_call_task_menu(&acetics).await?;

    Ok(())
}

fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    std::io::stdout().flush().unwrap();
}

fn description_render_config() -> RenderConfig<'static> {
    RenderConfig::default()
        .with_canceled_prompt_indicator(Styled::new("<skipped>").with_fg(Color::DarkYellow))
}

fn format_duration_to_hhmm(duration: Duration) -> String {
    let total_minutes = duration.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{:02}:{:02}", hours, minutes)
}

fn staff_select(acetics: &Acetics, prompt: &str, default: Option<usize>) -> Result<Staff> {
    let staff: Staff = Select::new(prompt, acetics.staffs().to_vec())
        .with_vim_mode(true)
        .with_starting_cursor(default.unwrap_or(0))
        .prompt()?;
    Ok(staff)
}

async fn create_call_task_menu(acetics: &Acetics) -> Result<()> {
    let start_time = Text::new("Nouvelle tâche - heure de début:")
        .with_default(&Local::now().format("%H:%M").to_string())
        .prompt()?;

    let description = Editor::new("Description:")
        .with_formatter(&|submission| {
            let char_count = submission.chars().count();
            if char_count == 0 {
                String::from("<skipped>")
            } else if char_count <= 20 {
                submission.into()
            } else {
                let mut substr: String = submission.chars().take(17).collect();
                substr.push_str("...");
                substr
            }
        })
        .with_file_extension(".md")
        .with_editor_command(OsStr::new("nvim"))
        .with_render_config(description_render_config())
        .prompt()?;

    let title = Text::new("Titre:")
        .with_validator(|i: &str| {
            if i.is_empty() {
                Ok(Validation::Invalid("Le champ est obligatoire".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let end_time = Text::new("Nouvelle tâche - heure de fin:")
        .with_default(&Local::now().format("%H:%M").to_string())
        .prompt()?;
    let work_time =
        end_time.parse::<chrono::NaiveTime>()? - start_time.parse::<chrono::NaiveTime>()?;

    let staff: Staff = staff_select(&acetics, "Assigner à:", Some(acetics.default_staff_index()))?;

    let status: TaskStatus =
        Select::new("Statut:", vec![TaskStatus::Closed, TaskStatus::Ongoing]).with_starting_cursor(if acetics.is_default_staff(&staff) { 0 } else { 1 }).prompt()?;

    let due_date = if let TaskStatus::Closed = status {
        Local::now().naive_local()
    } else {
        let date: NaiveDate = CustomType::<NaiveDate>::new("Date d'échéance:")
            .with_placeholder("dd/mm/yyyy")
            .with_starting_input(&Local::now().naive_local().format("%d/%m/%Y").to_string())
            .with_parser(&|i| NaiveDate::parse_from_str(i, "%d/%m/%Y").map_err(|_e| ()))
            .with_formatter(DEFAULT_DATE_FORMATTER)
            .with_error_message("Please type a valid date.")
            .prompt()?;

        let time = Text::new("Heure d'échéance:")
            .with_default(
                &Local::now()
                    .add(Duration::hours(1))
                    .format("%H:%M")
                    .to_string(),
            )
            .prompt()?;
        let time = time.parse::<chrono::NaiveTime>()?;
        date.and_hms_opt(time.hour(), time.minute(), 0).unwrap()
    };

    let confirm = Confirm::new("Voulez-vous enregistrer la tâche ?")
        .with_default(true)
        .prompt()?;

    if confirm {
        let task = Task::new(TaskType::CustomerCall, title, description)
            .with_assigned_staff(staff)
            .with_work_time(Some(format_duration_to_hhmm(work_time)))
            .with_due_date(due_date)
            .with_status(status);

        let res = acetics
            .json_request::<Task, serde_json::Value>(Method::POST, "tasks/create", &task)
            .await?;

        println!("{:?}", res);
    }

    Ok(())
}
