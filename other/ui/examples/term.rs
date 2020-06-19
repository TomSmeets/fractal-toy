use std::io::Stdout;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::screen::*;

use ui::{Nav, RenderCommand, UI};

type Screen = AlternateScreen<RawTerminal<Stdout>>;

fn gui(ui: &mut UI) {
    if ui.begin("a") {
        gui(ui);
        ui.end();
    }
    if ui.begin("b") {
        gui(ui);
        ui.end();
    }

    if ui.begin("c") {
        gui(ui);
        ui.end();
    }
}

fn show_gui(ui: &mut UI, w: &mut Screen) {
    ui.clear();
    gui(ui);
    write!(w, "{}", termion::cursor::Goto(1, 1)).unwrap();
    write!(w, "{}", termion::clear::All).unwrap();
    for cmd in ui.draw() {
        match cmd {
            RenderCommand::Text(indent, text) => {
                write!(w, "{}", termion::clear::CurrentLine).unwrap();

                for _ in 0..indent {
                    write!(w, "  ").unwrap();
                }

                write!(w, "{}", text).unwrap();

                write!(w, "\n\r").unwrap();
            },
            RenderCommand::Active(true) => write!(w, "{}", termion::style::Invert).unwrap(),
            RenderCommand::Active(false) => write!(w, "{}", termion::style::NoInvert).unwrap(),
            RenderCommand::Other(true) => write!(w, "{}", termion::style::Bold).unwrap(),
            RenderCommand::Other(false) => write!(w, "{}", termion::style::NoBold).unwrap(),
        }
    }
    write!(w, "{}", termion::clear::CurrentLine).unwrap();
    w.flush().unwrap();
}

fn main() {
    let stdin = stdin();
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    write!(screen, "{}", termion::cursor::Hide).unwrap();

    let mut gui = UI::new();

    show_gui(&mut gui, &mut screen);

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char('k') => gui.do_nav(Nav::Up),
            Key::Char('j') => gui.do_nav(Nav::Down),
            Key::Char('l') => gui.do_nav(Nav::Right),
            Key::Char('h') => gui.do_nav(Nav::Left),
            Key::Char(' ') => gui.do_nav(Nav::Close),
            _ => {},
        }
        show_gui(&mut gui, &mut screen);
    }

    write!(screen, "{}", termion::cursor::Show).unwrap();
}
