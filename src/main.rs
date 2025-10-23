use clap::{ArgAction, Parser, arg};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal,
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::{default, fs, io, process::Command};

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
enum DEKey {
    UNKNOWN,
    Exec,
    Name,
    Icon,
    Comment,
    Terminal,
    TryExec,
    Type,
    MimeType,
    Categories,
    Keywords,
    // add more keys as needed
}

impl Default for DEKey {
    fn default() -> Self {
        DEKey::UNKNOWN
    }
}

#[derive(Debug)]
struct DEField<'a> {
    r#type: DEKey,
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
    pub fn from_str(contents: &'a str) -> DesktopEntry<'a> {
        let mut groups: Vec<DEGroup<'a>> = Vec::new();
        let mut current_group: Option<DEGroup<'a>> = None;

        for (line_no, raw_line) in contents.lines().enumerate() {
            let line = raw_line.trim();

            // Skip empty or commented lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Group header like: [Desktop Entry]
            if line.starts_with('[') && line.ends_with(']') && line.len() >= 2 {
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

            // Key=Value lines
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let value = v.trim();
                let field_type = DesktopEntry::identify_type(key);

                if let Some(g) = current_group.as_mut() {
                    g.members.push(DEField {
                        r#type: field_type,
                        key,
                        value,
                    });
                } else {
                    // key/value before any group — create an anonymous group name = ""
                    current_group = Some(DEGroup {
                        name: "",
                        members: vec![DEField {
                            r#type: field_type,
                            key,
                            value,
                        }],
                    });
                }
            } else {
                eprintln!(
                    "Warning: ignoring malformed line {}: {:?}",
                    line_no + 1,
                    raw_line
                );
            }
        }

        // Push last group if present
        if let Some(g) = current_group {
            groups.push(g);
        }

        DesktopEntry { groups }
    }

    /// Simple mapping from key-name to DEKey. Extend this as needed.
    fn identify_type(key: &str) -> DEKey {
        match key {
            "Exec" => DEKey::Exec,
            "Name" => DEKey::Name,
            "Icon" => DEKey::Icon,
            "Comment" => DEKey::Comment,
            "Terminal" => DEKey::Terminal,
            "TryExec" => DEKey::TryExec,
            "Type" => DEKey::Type,
            "MimeType" => DEKey::MimeType,
            "Categories" => DEKey::Categories,
            "Keywords" => DEKey::Keywords,
            _ => DEKey::UNKNOWN,
        }
    }
}

// fn fuzzy_filter<'a>(items: &'a [&'a str], query: &str) -> Vec<(&'a str, i64)> {
//     let matcher = SkimMatcherV2::default();
//     let mut results: Vec<(&'a str, i64)> = items
//         .iter()
//         .filter_map(|item| {
//             /* fuzzy_match returns Option<i64> score (higher = better) */
//             matcher.fuzzy_match(item, query).map(|score| (*item, score))
//         })
//         .collect();
//
//     /* sort descending by score */
//     results.sort_by(|a, b| b.1.cmp(&a.1));
//     results
// }
#[derive(Debug, Default)]
enum AppMode {
    Input,
    #[default]
    Normal,
}

#[derive(Debug, Default)]
struct App {
    current_search: Option<String>,
    matches: Vec<(String, i64)>,
    exit: bool,
    mode: AppMode,
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
        frame.render_widget(self, frame.area());
    }
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event_mode(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event_mode(&mut self, key_event: KeyEvent) {
        match self.mode {
            AppMode::Normal => self.handle_normal_key_event(key_event),
            AppMode::Input => self.handle_input_key_event(key_event),
        }
    }

    fn handle_normal_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('/') => {
                self.mode = AppMode::Input;
            }
            // KeyCode::Esc => self.mode = AppMode::Normal,
            _ => {}
        }
    }

    fn handle_input_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.mode = AppMode::Normal,

            // Simple: if there's already a String, push the char; otherwise create one.
            KeyCode::Char(c) => {
                if let Some(s) = &mut self.current_search {
                    s.push(c);
                } else {
                    self.current_search = Some(c.to_string());
                }
            }

            // We might need to set `current_search` back to None if the last char is removed.
            // Use `take()` so we can move the String out, mutate it, then put it back (or leave None).
            KeyCode::Backspace => {
                match self.current_search.take() {
                    None => { /* nothing to do */ }
                    Some(mut s) => {
                        if s.len() > 1 {
                            s.pop();
                            self.current_search = Some(s);
                        } else {
                            // last char removed -> set to None
                            self.current_search = None;
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
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
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 1) Split the whole area vertically: top = searchbar (fixed height), bottom = main content
        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Top chunk: searchbar (placeholder)
        let search_title = Line::from(" Search ".bold());
        let search_block = Block::bordered()
            .title(search_title.centered())
            .border_set(border::THICK);

        // placeholder text for search field — no input logic included
        let search_text = match &self.current_search {
            None => Text::from(vec![Line::from(" / Type to search... ")]),
            Some(query) => Text::from(vec![Line::from(query.clone())]),
        };
        Paragraph::new(search_text)
            .centered()
            .block(search_block)
            .render(outer_chunks[0], buf);

        // 2) Bottom chunk: split horizontally into two columns (program list / description)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(outer_chunks[1]);

        // Left block (Programs)
        let name_title = Line::from(" Programs ".bold());
        let name_block = Block::bordered()
            .title(name_title.centered())
            .border_set(border::THICK);

        let program_list = Text::from(vec![Line::from(vec![
            "Some program 1 ".into(),
            // self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(program_list)
            .centered()
            .block(name_block)
            .render(chunks[0], buf);

        // Right block (Description)
        let desc_title = Line::from(" Description ".bold());
        let desc_block = Block::default()
            .title(desc_title.centered())
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let desc = Text::from(vec![Line::from(vec![" Description text ".into()])]);

        Paragraph::new(desc)
            .centered()
            .block(desc_block)
            .render(chunks[1], buf);
    }
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
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
    // println!("{:?}", de);

    let mut terminal = ratatui::init();

    let result = App::default().run(&mut terminal);

    ratatui::restore();
    result

    // let output = Command::new("yazi")
    //     // .arg("")
    //     .output()
    //     .expect("Failed to execute command");
}

#[cfg(test)]
mod tests {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Style, Stylize},
        widgets::Widget,
    };

    use crate::{App, DesktopEntry};

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

    fn render_test() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        assert_eq!(buf, expected);
    }
}
