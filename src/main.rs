use crossterm::{
    cursor,
    event::{Event, KeyCode},
    execute,
    style::{self, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::{
    collections::HashMap, env, fmt::Display, fs, io::Write, os::unix::process::CommandExt,
    path::PathBuf, process::Command as Cmd,
};

#[derive(Debug)]
enum MyError {
    Io(std::io::Error),
    GracefulShutdown,
}

impl From<std::io::Error> for MyError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

enum CommandExists {
    Exists(Command),
    NotExists(Vec<&'static str>),
}

struct Command {
    command: &'static str,
    args: Vec<&'static str>,
    automatic_new_folder: bool,
}

lazy_static::lazy_static! {
    static ref LANGUAGES: HashMap<ProjectLanguage, CommandExists> = {
        HashMap::from([
            (ProjectLanguage::Rust, CommandExists::Exists(Command {
                command: "cargo",
                args: vec!["new"],
                automatic_new_folder: true,
            })),
            (ProjectLanguage::Web, CommandExists::NotExists(vec!["index.html"])),
            (ProjectLanguage::Cpp, CommandExists::NotExists(vec!["src", "main.cpp"])),
            (ProjectLanguage::Ocaml, CommandExists::Exists(Command {
                command: "dune",
                args: vec!["init", "project"],
                automatic_new_folder: true,
            })),
            (ProjectLanguage::Haskell, CommandExists::Exists(Command {
                command: "cabal",
                args: vec!["init"],
                automatic_new_folder: false,
            })),
        ])
    };
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum ProjectLanguage {
    Rust,
    Web,
    Cpp,
    Ocaml,
    Haskell,
}

impl Display for ProjectLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::Web => write!(f, "web"),
            Self::Cpp => write!(f, "cpp"),
            Self::Ocaml => write!(f, "ocaml"),
            Self::Haskell => write!(f, "haskell"),
        }
    }
}

fn main() {
    let mut stdout = std::io::stdout();

    // Setting up the terminal for better usability
    execute!(stdout, terminal::EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();

    let project_name = match get_project_name(&mut stdout) {
        Ok(name) => name,
        Err(MyError::GracefulShutdown) => exit_program_gracefully(&mut stdout),
        Err(MyError::Io(e)) => panic!("{e}"),
    };

    let language = get_selected_language(&mut stdout).unwrap();

    let project_dir = std::env::current_dir().unwrap().join(&project_name);
    match LANGUAGES.get(&language).unwrap() {
        CommandExists::Exists(command) if command.automatic_new_folder => {
            Cmd::new(command.command)
                .args(&command.args)
                .arg(&project_name)
                .exec();
        }
        CommandExists::Exists(command) => {
            fs::create_dir(&project_dir).unwrap();
            env::set_current_dir(&project_dir).unwrap();

            Cmd::new(command.command)
                .args(&command.args)
                .arg(&project_name)
                .exec();
        }
        CommandExists::NotExists(file) => {
            fs::create_dir(&project_name).unwrap();
            env::set_current_dir(&project_dir).unwrap();

            let mut file_copy = file.clone();

            let file_name = file_copy.pop().unwrap();
            let path: PathBuf = file_copy.iter().collect();
            fs::create_dir_all(&path).unwrap();

            env::set_current_dir(project_dir.join(&path)).unwrap();
            let mut file = fs::File::create(file_name).unwrap();
            file.write_all(b"test").unwrap();
        }
    }

    // Returning the terminal to the normal state
    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();

    println!("Done!");
}

fn clear_screen(stdout: &mut std::io::Stdout) -> Result<(), MyError> {
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    crossterm::queue!(stdout, cursor::MoveTo(0, 0),)?;

    stdout.flush()?;

    Ok(())
}

fn get_project_name(stdout: &mut std::io::Stdout) -> Result<String, MyError> {
    clear_screen(stdout)?;

    let mut project_name = String::new();
    while let Event::Key(key) = crossterm::event::read().unwrap() {
        let project_name = &mut project_name;
        match key.code {
            KeyCode::Char('c') if key.modifiers == crossterm::event::KeyModifiers::CONTROL => {
                return Err(MyError::GracefulShutdown)
            }
            KeyCode::Enter => return Ok(project_name.clone()),
            KeyCode::Backspace => {
                project_name.pop();
            }
            KeyCode::Char(c) => project_name.push(c),
            _ => {}
        }

        // FIXME: I don't like this .clone() and also cleaning screen on every key press is bad
        clear_screen(stdout)?;
        crossterm::queue!(stdout, style::Print(project_name.clone().white()))?;

        stdout.flush()?;
    }

    Ok(project_name)
}

fn print_selection(stdout: &mut std::io::Stdout, selected: usize) -> Result<(), MyError> {
    crossterm::queue!(stdout, style::Print("What language do you want to use?"))?;

    for (index, language) in LANGUAGES.iter().enumerate() {
        let language = language.0;
        crossterm::queue!(
            stdout,
            // FIXME: handle possible errors
            cursor::MoveTo(0, (index + 1).try_into().unwrap()),
            style::PrintStyledContent(if index == selected {
                format!("> {language}\n").yellow()
            } else {
                format!("  {language}\n").magenta()
            })
        )?;
    }

    stdout.flush()?;

    Ok(())
}

fn get_selected_language(stdout: &mut std::io::Stdout) -> Result<ProjectLanguage, MyError> {
    execute!(stdout, cursor::Hide).unwrap();
    let mut selected = 0;
    loop {
        clear_screen(stdout)?;

        print_selection(stdout, selected).unwrap();

        if let Event::Key(key) = crossterm::event::read().unwrap() {
            match key.code {
                KeyCode::Up => selected -= 1,
                KeyCode::Down => selected += 1,
                KeyCode::Enter => break,
                _ => {}
            }
        }
    }

    execute!(stdout, cursor::Show).unwrap();
    // FIXME: handle possible errors
    Ok(*LANGUAGES.iter().nth(selected).unwrap().0)
}

fn exit_program_gracefully(stdout: &mut std::io::Stdout) -> ! {
    // Returning the terminal to the normal state
    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
    println!("Done!");
    std::process::exit(0)
}
