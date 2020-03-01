use ::termion::screen::AlternateScreen;

use std::io::{stdin, stdout, Stdout, Write};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::*;

// TODO split into tokens
// group tokens such as parens and strings
// angle brackets are harder, as they can also be just lt and gt'
// paragraphs
// lines
//
//
// note that tokoens can span multiple real lines, these will be seen as one virtual line
//
// below is
//  two lines
//  one paragraph
//
//
// #[derive(Debug)]
// enum Mode {
//     Normal,
//     Insert,
// }

#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
}

struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    fn from(s: &str) -> Buffer {
        Buffer {
            lines: s
                .split(|s: char| s.is_whitespace())
                .map(|x| x.to_string())
                .collect(),
        }
    }

    fn to_string(&self) -> String {
        self.lines.join("\n\n")
    }
}

struct Editor {
    mode: Mode,
    quit: bool,
    line: i32,
    buffer: Buffer,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::from("nothing\n"),
            line: 0,
            mode: Mode::Normal,
            quit: false,
        }
    }

    fn handle(&mut self, c: Key) {
        match self.mode {
            Mode::Normal => match c {
                Key::Char('i') => self.mode = Mode::Insert,
                Key::Char('j') => self.line += 1,
                Key::Char('k') => self.line -= 1,
                Key::Char('q') => self.quit = true,
                _ => (),
            },

            Mode::Insert => match c {
                Key::Char(c) => {
                    let l = self.buffer.lines.get_mut(self.line as usize);
                    if let Some(l) = l {
                        l.push(c);
                        let s = self.buffer.to_string();
                        self.buffer = Buffer::from(&s);
                    }
                }
                Key::Esc => {
                    self.mode = Mode::Normal;
                }
                _ => (),
            },
        }
    }

    fn display<T: std::io::Write>(&self, out: &mut termion::raw::RawTerminal<T>) {
        let (sz_x, sz_y) = termion::terminal_size().unwrap();

        write!(out, "{}", termion::clear::All).unwrap();

        {
            // status
            write!(out, "{}", termion::cursor::Goto(1, sz_y)).unwrap();
            write!(out, "mode: {:?}", self.mode).unwrap();
        }

        write!(out, "{}", termion::cursor::Goto(1, 1)).unwrap();
        let mut i = 0;
        let parts = self
            .buffer
            .lines
            .iter()
            .enumerate()
            .skip((self.line - 1).max(0) as usize)
            .take(4);
        for (part_i, part) in parts {
            write!(
                out,
                "{}>>> chunck: {} <<<<{}",
                termion::style::Underline,
                part_i,
                termion::style::Reset
            )
            .unwrap();
            i += 1;
            if part_i as i32 == self.line {
                write!(out, "{}", termion::style::Invert).unwrap();
            }
            for (j, l) in part.split("\n").enumerate() {
                write!(out, "{}", termion::cursor::Goto(1, i as u16 + 1)).unwrap();
                write!(out, "{}", l).unwrap();
                write!(out, "{}", " ".repeat(sz_x as usize - l.len())).unwrap();
                i += 1;
            }
            write!(out, "{}", termion::style::Reset).unwrap();
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Split(Vec<Token>),
    Word(String),
}

impl Token {
    fn from(s: &str) -> Token {
        let mut dst = String::new();
        let chars = s.chars();
        let mut toks = Vec::new();

        for c in chars {
            if c == ' ' {
                toks.push(Token::Word(dst.clone()));
                dst.clear();
            }
            dst.push(c);
        }
        toks.push(Token::Word(dst.clone()));
        dst.clear();

        if toks.len() == 1 {
            Token::Word(dst)
        } else {
            Token::Split(toks)
        }
    }
}

fn char_cat(c: char) -> i32 {
    let mut i = 1;

    if c.is_alphanumeric() { return i }
    i += 1;
    if c.is_whitespace()  { return i }
    i += 1;
    if c == ';' { return i; }
    i += 1;
    i
}

fn split(input: &str) -> Vec<String> {
    let mut xs = Vec::new();
    let mut word = String::new();

    let mut current_cat = -1;

    for c in input.chars() {
        let cat = char_cat(c);

        if cat != current_cat {
            if current_cat != -1 {
                xs.push(word.clone());
            }
            word.clear();
            current_cat = cat;
        }

        word.push(c);
    }
    xs.push(word);
    xs
}

#[test]
fn test_split(){
    assert_eq!(split("hello"), ["hello"] );
    assert_eq!(split("hello world"), ["hello", " ", "world"] );
    assert_eq!(split("hello.world"), ["hello", ".", "world"] );

    assert_eq!(split("a..hi.?type.x->y: 5"), ["a", "..", "hi", ".?", "type", ".", "x", "->", "y", ":", " ", "5"] );
    assert_eq!(split("{a = b;}"), ["{", "a", " ", "=", " ", "b", ";", "}"] );
}

#[test]
fn test_token_0() {
    use Token::*;

    let ti = Token::from("hello");
    let to = Word("hello".to_string());
    assert_eq!(ti, to);
}
#[test]
fn test_token_1() {
    use Token::*;

    let ti = Token::from("hello world");
    let to = Split(vec![
        Word("hello".to_string()),
        Word(" ".to_string()),
        Word("hello".to_string()),
    ]);
    assert_eq!(ti, to);
}

#[test]
fn test_token_2() {
    use Token::*;

    let ti = Token::from("hello { world }");
    let to = Split(vec![
        Word("hello".to_string()),
        Word(" ".to_string()),
        Split(vec![
            Word("{".to_string()),
            Word(" ".to_string()),
            Word("world".to_string()),
            Word(" ".to_string()),
            Word("}".to_string()),
        ]),
    ]);
    assert_eq!(ti, to);
}

fn tokens(s: &str) -> Vec<String> {
    let cs: Vec<_> = s.chars().collect();

    let mut tokens = Vec::new();

    let mut tok = String::new();

    let mut is_whitespace = false;

    for c in cs {
        if (c == '\n' || c == ' ') != is_whitespace {
            tokens.push(tok);
            tok = String::new();
            is_whitespace = !is_whitespace;
        }

        tok.push(c);
    }
    tokens.push(tok);
    tokens
}

fn main() {
    let input_string = std::fs::read_to_string("src/main.rs").unwrap();

    {
        let ts = tokens(&input_string[..100]);
        println!("{:#?}", ts);
    }

    if false {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "{}", termion::clear::All).unwrap();
        write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
        write!(stdout, "hello world").unwrap();
        stdout.flush().unwrap();

        let mut editor = Editor::new();
        editor.buffer = Buffer::from(&input_string);

        editor.display(&mut stdout);
        stdout.flush().unwrap();

        for c in stdin.keys() {
            editor.handle(c.unwrap());
            editor.display(&mut stdout);
            stdout.flush().unwrap();
            if editor.quit {
                break;
            }
        }
        write!(stdout, "{}", termion::clear::All).unwrap();
    }
}
