use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct CharStream {
    chars: Vec<char>,
    index: usize,

    line: usize,
    col: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Position(pub usize, pub usize, pub usize);

impl Position {
    pub fn new(start: usize, end: usize, idx: usize) -> Position {
        Position(start, end, idx)
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self::Display::fmt(&self, f)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Clone)]
pub struct Range(pub Position, pub Position);

impl Range {
    pub fn new(start: Position, end: Position) -> Range {
        Range(start, end)
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} to {}", self.0, self.1)
    }
}

impl Debug for Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self::Display::fmt(&self, f)
    }
}

impl CharStream {
    pub fn from(s: &str) -> CharStream {
        CharStream {
            chars: s.chars().collect(),
            index: 0,
            line: 1,
            col: 1,
        }
    }
    pub fn get_pos(&self) -> Position {
        Position::new(self.line, self.col, self.index)
    }

    pub fn get_loc(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    pub fn since_pos(&mut self, pos: Position) -> (&[char], Range) {
        (
            &self.chars[pos.2..self.index],
            Range::new(pos, Position(self.line, self.col, self.index)),
        )
    }

    pub fn set_pos(&mut self, pos: Position) {
        self.line = pos.0;
        self.col = pos.1;
        self.index = pos.2;
    }

    fn next(&mut self) -> Option<char> {
        let char = self.chars.get(self.index).copied();
        self.index += 1;
        if let Some(c) = char {
            self.col += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            }
        }
        char
    }
    pub fn peek(&self) -> Option<&char> {
        self.chars.get(self.index)
    }
    pub fn report_unknown(&mut self) -> ! {
        panic!(
            "Unknown symbol {} at {}:{}",
            self.next().unwrap(),
            self.line,
            self.col
        )
    }
}

impl Iterator for CharStream {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[cfg(test)]
mod tests {
    use super::CharStream;

    #[test]
    fn it_works() {
        let mut stream = CharStream::from("Hello World!");

        assert_eq!('H', stream.next().unwrap());
        assert_eq!('e', stream.next().unwrap());
        assert_eq!('l', stream.next().unwrap());
        assert_eq!('l', stream.next().unwrap());
        assert_eq!('o', stream.next().unwrap());
        assert_eq!(' ', stream.next().unwrap());
        assert_eq!('W', stream.next().unwrap());
        assert_eq!('o', stream.next().unwrap());
        assert_eq!('r', stream.next().unwrap());
        assert_eq!('l', stream.next().unwrap());
        assert_eq!('d', stream.next().unwrap());
        assert_eq!('!', stream.next().unwrap());
        assert_eq!(None, stream.next())
    }
    #[test]
    fn correct_line_col() {
        let mut stream = CharStream::from("AB\nCD");

        // Line
        // | 0 1 2 <- Column
        // 1   A B
        // 2 n C D

        assert_eq!((1, 1), stream.get_loc());
        stream.next(); // A
        assert_eq!((1, 2), stream.get_loc());
        stream.next(); // B
        assert_eq!((1, 3), stream.get_loc());
        stream.next(); // \n
        assert_eq!((2, 1), stream.get_loc());
        stream.next(); // C
        assert_eq!((2, 2), stream.get_loc());
        stream.next(); // D
        assert_eq!((2, 3), stream.get_loc());
        stream.next(); // None
        assert_eq!((2, 3), stream.get_loc());
    }
}
