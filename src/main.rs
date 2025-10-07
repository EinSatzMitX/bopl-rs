use clap::{ArgAction, Parser, arg};
use std::{fs, process::Command};

use winit::{
    application::ApplicationHandler,
    // dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /* Theoretically, when this becomes GUI only, the required can be set to false */
    #[arg(short = 'i', long = "input", action =ArgAction::Set, required=true)]
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

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested; stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(args.file_path.clone()).unwrap();
    let de = DesktopEntry::from_str(&contents);

    println!("{:?}", de);

    // let output = Command::new("yazi")
    //     // .arg("")
    //     .output()
    //     .expect("Failed to execute command");
    //
    // let event_loop = EventLoop::new().unwrap();
    //
    // event_loop.set_control_flow(ControlFlow::Wait);
    //
    // let mut app = App::default();
    // let _ = event_loop.run_app(&mut app);
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
