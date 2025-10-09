use clap::{ArgAction, Parser, arg};
use crossterm::{event, terminal};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{DefaultTerminal, Frame};
use std::{fs, io, process::Command};

// use winit::{
//     application::ApplicationHandler,
//     event::WindowEvent,
//     event_loop::{ControlFlow, EventLoop},
//     window::Window,
// };

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /* Theoretically, when this becomes GUI only, the required can be set to false */
    #[arg(short = 'i', long = "input", action =ArgAction::Set/* , required=false */)]
    file_path: String,
}

#[derive(Debug)]
struct DEField<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Debug)]
struct DEGroup<'a> {
    name: &'a str,
    members: Vec<DEField<'a>>,
}

#[derive(Debug)]
struct DesktopEntry<'a> {
    groups: Vec<DEGroup<'a>>,
}

impl<'a> DesktopEntry<'a> {
    fn from_str(contents: &'a str) -> DesktopEntry<'a> {
        // Start parsing here
        let mut groups: Vec<DEGroup<'a>> = Vec::new();
        let mut current_group: Option<DEGroup<'a>> = None;

        for (line_no, raw_line) in contents.lines().enumerate() {
            let line = raw_line.trim();

            /* Skip empty or commented lines (Even `  # stuff` counts as commented, because of the
             * tirm function) */
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            /* Group parsing */
            if line.starts_with('[') && line.ends_with(']') {
                if let Some(g) = current_group.take() {
                    groups.push(g);
                }
                let name = line[1..line.len() - 1].trim();
                current_group = Some(DEGroup {
                    name,
                    members: Vec::new(),
                });
                continue;
            }

            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let value = v.trim();

                if let Some(g) = current_group.as_mut() {
                    g.members.push(DEField { key, value });
                } else {
                    /* Key-value-pair before any groups, I don't know if this is allowed but let's
                     * handle it anyways */
                    current_group = Some(DEGroup {
                        name: "",
                        members: vec![DEField { key, value }],
                    });
                }
            } else {
                eprintln!(
                    "Warning: ignoring malformed line {}: {:?}",
                    line_no + 1,
                    raw_line
                )
            }
        }

        if let Some(g) = current_group {
            groups.push(g);
        }

        DesktopEntry { groups }
    }
}

fn fuzzy_filter<'a>(items: &'a [&'a str], query: &str) -> Vec<(&'a str, i64)> {
    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(&'a str, i64)> = items
        .iter()
        .filter_map(|item| {
            /* fuzzy_match returns Option<i64> score (higher = better) */
            matcher.fuzzy_match(item, query).map(|score| (*item, score))
        })
        .collect();

    /* sort descending by score */
    results.sort_by(|a, b| b.1.cmp(&a.1));
    results
}

#[derive(Debug, Default)]
struct App {
    counter: u8,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        todo!()
    }
    fn handle_events(&mut self) -> io::Result<()> {
        todo!()
    }
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, event::Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("Hello, World!", frame.area());
}

fn main() -> std::io::Result<()> {
    // let args = Args::parse();
    // let contents = fs::read_to_string(args.file_path.clone()).unwrap();
    // let de = DesktopEntry::from_str(&contents);

    let terminal = ratatui::init();

    let result = App::default().run(terminal);

    ratatui::restore();
    result

    // let output = Command::new("yazi")
    //     // .arg("")
    //     .output()
    //     .expect("Failed to execute command");
}

#[cfg(test)]
mod tests {
    use crate::DesktopEntry;

    #[test]
    fn parse_test() {
        let sample = r#"
# example .desktop
[Desktop Entry]
Name=Hello Triangle
Comment=Shows a triangle
Exec=hello_triangle
Type=Application

[Hidden]
Hidden=true
"#;

        let de = DesktopEntry::from_str(sample);
        println!("{:?}", de);
    }
}
