use clap::Parser;
use std::{env, fs, process};

mod flags;

fn main() {
    let args = flags::Cli::parse();

    let run = || -> Result<(), toado::Error> {
        let app_dir = match init_directory() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to initialize application directory: {e}");
                return Err(e);
            }
        };

        let app = match toado::Server::open(&format!("{app_dir}/database")) {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Failed to create application server: {e}");
                return Err(e);
            }
        };

        if let Err(e) = app.init() {
            eprintln!("Failed to initialize application server: {e}");
            return Err(e);
        };

        // If command provided, execute and exit application
        if let Some(command) = args.command {
            let message = match handle_command(command, app) {
                Ok(message) => message,
                Err(e) => {
                    eprintln!("Failed to execute command: {e}");
                    return Err(e);
                }
            };

            // If command returns message, print to stdout
            if let Some(message) = message {
                println!("{message}")
            }

            return Ok(());
        }

        // TODO: If no command provided, run TUI
        println!("toado");
        Ok(())
    };

    if let Err(e) = run() {
        if let Some(e) = e.source() {
            eprintln!("Caused by: {e}")
        }
        process::exit(1)
    }
}

/// Creates the directory for application files if one does not exist
pub fn init_directory() -> Result<String, toado::Error> {
    // Get user home directory
    let home_dir = env::var("HOME")?;
    let app_dir = format!("{home_dir}/.local/share/toado");

    // Create application directory if it doesn't exist
    fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
}

/// Handle application commands from the CLI
fn handle_command(
    command: flags::Commands,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let message = match command {
        flags::Commands::Add(args) => on_add(args, app)?,
        flags::Commands::Delete(_args) => return Err(Into::into("deletion is not implemented")),
        flags::Commands::Ls(_args) => return Err(Into::into("listing is not implemented")),
    };

    Ok(message)
}

fn on_add(args: flags::AddArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        let (task_id, task_name) = create_task(args, app)?;
        Ok(Some(format!("Created task {task_name} with id {task_id}")))
    } else {
        Err(Into::into("project adding not implemented"))
    }
}

fn create_task(args: flags::AddArgs, app: toado::Server) -> Result<(i64, String), toado::Error> {
    let theme = dialoguer::theme::ColorfulTheme::default();

    let name = option_or_input(
        args.name,
        dialoguer::Input::with_theme(&theme).with_prompt("Name"),
    )?;

    let priority = option_or_input(
        args.item_priority,
        dialoguer::Input::with_theme(&theme).with_prompt("Priority"),
    )?;

    let start_time = if args.optional {
        None
    } else {
        option_or_input_option(
            args.start_time,
            dialoguer::Input::with_theme(&theme)
                .with_prompt("Start Time (optional)")
                .allow_empty(true),
        )?
    };

    let end_time = if args.optional {
        None
    } else {
        option_or_input_option(
            args.end_time,
            dialoguer::Input::with_theme(&theme)
                .with_prompt("End Time (optional)")
                .allow_empty(true),
        )?
    };

    let repeat = if args.optional {
        None
    } else {
        option_or_input_option(
            args.repeat,
            dialoguer::Input::with_theme(&theme)
                .with_prompt("Repeats (optional)")
                .allow_empty(true),
        )?
    };

    let task_id = app.add_task(toado::AddTaskArgs {
        name: String::from(&name),
        priority,
        status: toado::ItemStatus::Incomplete,
        start_time,
        end_time,
        repeat,
        notes: None,
    })?;

    Ok((task_id, name))
}

/// Return the `T` of an `Option<T>` if `Option<T>` is `Some<T>`, otherwise, prompt the user for an
/// input of type `T`.
///
/// # Errors
///
/// Returns error if getting user input fails
fn option_or_input<T>(value: Option<T>, input: dialoguer::Input<T>) -> Result<T, toado::Error>
where
    T: Clone + ToString + std::str::FromStr,
    <T as std::str::FromStr>::Err: ToString,
{
    match value {
        Some(value) => Ok(value),
        None => Ok(input.interact_text()?),
    }
}

/// Return the `Some(String)` of an `Option<String>` if `Option<String>` is `Some(String)`, otherwise,
/// prompt the user for an input of type `String`. If user input is blank, return `None`
/// TODO: Make this function generic. ie. `Result<Option<T>, toado::Error>`
///
/// # Errors
///
/// Returns error if getting user input fails
fn option_or_input_option(
    value: Option<String>,
    input: dialoguer::Input<String>,
) -> Result<Option<String>, toado::Error> {
    match value {
        Some(value) => Ok(Some(value)),
        None => {
            let user_input = input.interact_text()?;

            Ok(if !user_input.is_empty() {
                Some(user_input)
            } else {
                None
            })
        }
    }
}
