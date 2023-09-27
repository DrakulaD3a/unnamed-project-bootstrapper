use crossterm::{
    cursor,
    event::Event,
    execute,
    style::{self, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use std::{
    fmt::Display,
    io::{stdout, Write},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

struct Command<'a> {
    command: &'a str,
    automatic_new_folder: bool,
}

#[derive(Debug, EnumIter)]
enum ProjectLanguage {
    Rust,
    Web,
    Cpp,
    Ocaml,
    Haskell,
}

impl ProjectLanguage {
    fn get_command_to_initialize(&self) -> Option<Command> {
        use ProjectLanguage as L;
        match self {
            L::Rust => Some(Command {
                command: "cargo new",
                automatic_new_folder: true,
            }),
            L::Web => None,
            L::Cpp => None,
            L::Ocaml => Some(Command {
                command: "dune init project",
                automatic_new_folder: true,
            }),
            L::Haskell => Some(Command {
                command: "stack init",
                automatic_new_folder: false,
            }),
        }
    }
}

impl Display for ProjectLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ProjectLanguage as L;
        match self {
            L::Rust => write!(f, "rust"),
            L::Web => write!(f, "web"),
            L::Cpp => write!(f, "cpp"),
            L::Ocaml => write!(f, "ocaml"),
            L::Haskell => write!(f, "haskell"),
        }
    }
}

fn main() {
    let mut stdout = stdout();

    enable_raw_mode().unwrap();
    execute!(stdout, cursor::Hide).unwrap();

    let language = get_selected_language(&mut stdout).unwrap();

    execute!(stdout, cursor::Show).unwrap();
    disable_raw_mode().unwrap();
}

fn print_selection(stdout: &mut std::io::Stdout, selected: usize) -> Result<(), std::io::Error> {
    crossterm::queue!(
        stdout,
        cursor::MoveTo(0, 0),
        style::Print("What language do you want to use?")
    )?;

    for (index, language) in ProjectLanguage::iter().enumerate() {
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

fn get_selected_language(stdout: &mut std::io::Stdout) -> Result<ProjectLanguage, std::io::Error> {
    let mut selected = 0;
    loop {
        stdout
            .execute(terminal::Clear(terminal::ClearType::All))
            .unwrap();

        print_selection(stdout, selected).unwrap();

        if let Event::Key(key) = crossterm::event::read().unwrap() {
            use crossterm::event::KeyCode;
            match key.code {
                KeyCode::Up => selected -= 1,
                KeyCode::Down => selected += 1,
                KeyCode::Enter => break,
                _ => {}
            }
        }
    }

    // FIXME: handle possible errors
    Ok(ProjectLanguage::iter().nth(selected).unwrap())
}
