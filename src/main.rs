mod acetics;

use chrono::{Duration, Local, NaiveDate, Timelike};
use inquire::{
    formatter::DEFAULT_DATE_FORMATTER, ui::{Color, RenderConfig, Styled}, Confirm, CustomType, Editor, Select, SelectPromptAction, Text
};

use acetics::{Acetics, Task, TaskStatus, TaskType};
use reqwest::Method;
use std::ops::Add;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let acetics = Acetics::load_config()?;

    create_call_task_menu(&acetics).await?;

    // let ans = Confirm::new("Do you live in Brazil?")
    //     .with_default(false)
    //     .with_help_message("This data is stored for good reasons")
    //     .prompt();
    //
    // match ans {
    //     Ok(true) => println!("That's awesome!"),
    //     Ok(false) => println!("That's too bad, I've heard great things about it."),
    //     Err(_) => println!("Error with questionnaire, try again later"),
    // }

    Ok(())
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

async fn create_call_task_menu(acetics: &Acetics) -> Result<(), Box<dyn std::error::Error>> {
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
        .with_render_config(description_render_config())
        .prompt()?;

    let title = Text::new("Titre:").prompt()?;

    let end_time = Text::new("Nouvelle tâche - heure de fin:")
        .with_default(&Local::now().format("%H:%M").to_string())
        .prompt()?;
    let work_time =
        end_time.parse::<chrono::NaiveTime>()? - start_time.parse::<chrono::NaiveTime>()?;

    let status: TaskStatus =
        Select::new("Statut:", vec![TaskStatus::Closed, TaskStatus::Ongoing]).prompt()?;

    let due_date = if let TaskStatus::Closed = status {
        Local::now().naive_local()
    } else {
        let date: NaiveDate = CustomType::<NaiveDate>::new("Date d'échéance:")
            .with_placeholder("dd/mm/yyyy")
            .with_parser(&|i| NaiveDate::parse_from_str(i, "%d/%m/%Y").map_err(|_e| ()))
            .with_formatter(DEFAULT_DATE_FORMATTER)
            .with_error_message("Please type a valid date.")
            .with_default(Local::now().naive_local().into())
            .prompt()?;

        let time = Text::new("Heure d'échéance:").with_default(&Local::now().add(Duration::hours(1)).format("%H:%M").to_string()).prompt()?;
        let time = time.parse::<chrono::NaiveTime>()?;
        date.and_hms_opt(time.hour(), time.minute(), 0).unwrap()
    };

    let task = Task::new(TaskType::CustomerCall, title, description)
        .with_work_time(Some(format_duration_to_hhmm(work_time)))
        .with_due_date(due_date)
        .with_status(status);

    let res = acetics
        .json_request::<Task, serde_json::Value>(Method::POST, "tasks/create", &task)
        .await?;

    println!("{:?}", res);

    Ok(())
}

// async fn get_request(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
//     let client = reqwest::Client::new();
//     let response = client
//         .get(config.acetics_endpoint.clone() + "/api/v1/users")
//         .header("Authorization", "Bearer 1234567890")
//         .send()
//         .await?;
// }
