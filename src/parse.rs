use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::input::CharStream;
use crate::input::Position;
use crate::input::Range;

#[derive(Clone)]
pub enum SymbolType {
    Sequence(Vec<Symbol>),
    OneOf(Vec<Symbol>),
    Optional(Symbol),
    OneOrMore(Symbol),
    ZeroOrMore(Symbol),
    Terminal(char),
}

#[derive(Clone)]
pub struct SymbolInner {
    pub symbol_type: SymbolType,
    pub node_name: Option<String>,
    raw: bool,
}

#[derive(Clone)]
pub struct Symbol(pub Rc<RefCell<SymbolInner>>);

#[derive(Debug)]
pub enum Data {
    ListData(Vec<Node>),
    Raw(String),
    NoData,
}

#[derive(Debug)]
pub struct Node {
    node_type: String,
    loc: Range,
    data: Data,
}

pub enum AdvanceResult {
    // DataOnly(Data),
    Ok(Node),
    Err,
}

type DataResult = Result<Data, ()>;

impl Symbol {
    fn try_advance(&self, stream: &mut CharStream) -> AdvanceResult {
        let start_pos = stream.get_pos();
        let data: DataResult = match &self.0.borrow().symbol_type {
            SymbolType::Sequence(s) => Symbol::try_advance_sequence(s, stream),
            SymbolType::OneOf(s) => Symbol::try_advance_one_of(s, stream),
            SymbolType::Optional(s) => Symbol::try_advance_optional(s, stream),
            SymbolType::OneOrMore(s) => {
                Symbol::try_advance_one_or_more(s, stream)
            }
            SymbolType::ZeroOrMore(s) => {
                Symbol::try_advance_zero_or_more(s, stream)
            }
            SymbolType::Terminal(c) => Symbol::try_advance_terminal(c, stream),
        };
        return match data {
            Ok(d) => {
                let (raw, range) = stream.since_pos(start_pos);
                AdvanceResult::Ok(Node {
                    node_type: if let Some(name) = &self.0.borrow().node_name {
                        name.clone()
                    } else {
                        match self.0.borrow().symbol_type {
                            SymbolType::Sequence(_) => "Sequence".to_owned(),
                            SymbolType::OneOf(_) => "OneOf".to_owned(),
                            SymbolType::Optional(_) => "Optional".to_owned(),
                            SymbolType::OneOrMore(_) => "OneOrMore".to_owned(),
                            SymbolType::ZeroOrMore(_) => {
                                "ZeroOrMore".to_owned()
                            }
                            SymbolType::Terminal(_) => "Terminal".to_owned(),
                        }
                    },
                    loc: range,
                    data: if self.0.borrow().raw {
                        Data::Raw(raw.into_iter().collect())
                    } else {
                        d
                    },
                })
            }
            Err(_) => {
                stream.set_pos(start_pos);
                AdvanceResult::Err
            }
        };
    }

    fn try_advance_one_or_more(
        symbol: &Symbol,
        stream: &mut CharStream,
    ) -> DataResult {
        let mut once = false;
        let mut data = Vec::new();
        loop {
            match symbol.try_advance(stream) {
                AdvanceResult::Ok(node) => {
                    if once == false {
                        once = true;
                    }
                    data.push(node)
                }
                AdvanceResult::Err => {
                    return if once {
                        Ok(Data::ListData(data))
                    } else {
                        Err(())
                    };
                }
            }
        }
    }

    fn try_advance_zero_or_more(
        symbol: &Symbol,
        stream: &mut CharStream,
    ) -> DataResult {
        let mut data = Vec::new();
        loop {
            match symbol.try_advance(stream) {
                AdvanceResult::Ok(d) => data.push(d),
                AdvanceResult::Err => {
                    return DataResult::Ok(Data::ListData(data))
                }
            }
        }
    }

    fn try_advance_optional(
        symbol: &Symbol,
        stream: &mut CharStream,
    ) -> DataResult {
        return match symbol.try_advance(stream) {
            AdvanceResult::Ok(n) => DataResult::Ok(Data::ListData(vec![n])),
            AdvanceResult::Err => DataResult::Ok(Data::NoData),
        };
    }

    fn try_advance_one_of(
        symbols: &[Symbol],
        stream: &mut CharStream,
    ) -> DataResult {
        for symbol in symbols {
            if let AdvanceResult::Ok(n) = symbol.try_advance(stream) {
                return DataResult::Ok(Data::ListData(vec![n]));
            }
        }
        DataResult::Err(())
    }

    fn try_advance_sequence(
        symbols: &[Symbol],
        stream: &mut CharStream,
    ) -> DataResult {
        let mut data = Vec::new();
        for symbol in symbols {
            match symbol.try_advance(stream) {
                AdvanceResult::Ok(n) => data.push(n),
                AdvanceResult::Err => return DataResult::Err(()),
            };
        }
        DataResult::Ok(Data::ListData(data))
    }

    fn try_advance_terminal(c: &char, stream: &mut CharStream) -> DataResult {
        if let Some(next_char) = stream.peek() {
            if c == next_char {
                stream.next();
                return DataResult::Ok(Data::Raw(c.to_string()));
            } else {
                DataResult::Err(())
            }
        } else {
            DataResult::Err(())
        }
    }
}

impl Symbol {
    pub fn terminal(char: char) -> Symbol {
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::Terminal(char),
            raw: false,
            node_name: None,
        })))
    }
    pub fn one_of(symbols: Vec<&Symbol>) -> Symbol {
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::OneOf(
                symbols.into_iter().map(|s| s.clone()).collect(),
            ),
            raw: false,
            node_name: None,
        })))
    }

    pub fn sequence(symbols: Vec<&Symbol>) -> Symbol {
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::Sequence(
                symbols.into_iter().map(|s| s.clone()).collect(),
            ),
            raw: false,
            node_name: None,
        })))
    }
    pub fn optional(symbol: &Symbol) -> Symbol {
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::Optional(symbol.clone()),
            raw: false,
            node_name: None,
        })))
    }
    pub fn one_or_more(symbol: &Symbol) -> Symbol {
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::OneOrMore(symbol.clone()),
            raw: false,
            node_name: None,
        })))
    }
    pub fn zero_or_more(symbol: &Symbol) -> Symbol {
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::ZeroOrMore(symbol.clone()),
            raw: false,
            node_name: None,
        })))
    }
    pub fn sequence_chars(string: &str) -> Symbol {
        let chars = string.chars();
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::Sequence(
                chars.map(|c| Symbol::terminal(c.clone())).collect(),
            ),
            raw: false,
            node_name: Some(string.to_owned()),
        })))
    }
    pub fn one_of_chars(string: &str) -> Symbol {
        let chars = string.chars();
        Symbol(Rc::new(RefCell::new(SymbolInner {
            symbol_type: SymbolType::OneOf(
                chars.map(|c| Symbol::terminal(c.clone())).collect(),
            ),
            raw: false,
            node_name: None,
        })))
    }
}

impl Symbol {
    pub fn node_name(self, name: &str) -> Self {
        self.0.borrow_mut().node_name = Some(name.to_owned());
        self
    }
    pub fn raw(self) -> Self {
        self.0.borrow_mut().raw = true;
        self
    }
}

pub fn parse(root: &Symbol, stream: &mut CharStream) -> ParseResult {
    match root.try_advance(stream) {
        AdvanceResult::Ok(n) => {
            return match stream.peek() {
                Some(_) => ParseResult::Err(stream.get_pos()),
                None => ParseResult::Ok(n),
            }
        }
        AdvanceResult::Err => ParseResult::Err(stream.get_pos()),
    }
}

#[derive(Debug)]
pub enum ParseResult {
    Ok(Node),
    Err(Position),
}

#[cfg(test)]
mod tests {
    use super::*;

    //Helpers
    fn should_parse(root: &Symbol, stream: &mut CharStream) {
        assert!(matches!(parse(root, stream), ParseResult::Ok(_)));
    }

    fn shouldnt_parse(root: &Symbol, stream: &mut CharStream) {
        assert!(matches!(parse(root, stream), ParseResult::Err(_)));
    }

    #[test]
    fn literal() {
        let symbol = Symbol::sequence_chars("test");
        let mut stream = CharStream::from("test");

        should_parse(&symbol, &mut stream);
        assert_eq!((1, 5), stream.get_loc());
    }

    #[test]
    fn no_eof_after_parse() {
        let symbol = Symbol::terminal('a');
        let mut stream = CharStream::from("ab");

        shouldnt_parse(&symbol, &mut stream);
    }

    #[test]
    fn literal_rewind() {
        let symbol = Symbol::sequence_chars("test");
        let mut stream = CharStream::from("tesa");

        shouldnt_parse(&symbol, &mut stream);
        assert_eq!((1, 1), stream.get_loc());
    }

    #[test]
    fn one_of_chars() {
        let symbol = Symbol::one_of_chars("abc");
        let mut stream = CharStream::from("a");

        should_parse(&symbol, &mut stream);
    }

    #[test]
    fn one_of() {
        // { 'a' }+ | { 'b' }+
        let symbol = Symbol::one_of(vec![
            &Symbol::one_or_more(&Symbol::terminal('a')),
            &Symbol::one_or_more(&Symbol::terminal('b')),
        ]);
        let mut stream1 = CharStream::from("aaa");
        let mut stream2 = CharStream::from("bbb");

        should_parse(&symbol, &mut stream1);
        should_parse(&symbol, &mut stream2);
    }
    #[test]
    fn optional() {
        let symbol = Symbol::optional(&Symbol::terminal('-'));
        let mut stream1 = CharStream::from("-");
        let mut stream2 = CharStream::from("");
        let mut stream3 = CharStream::from("a");
        should_parse(&symbol, &mut stream1);
        should_parse(&symbol, &mut stream2);
        shouldnt_parse(&symbol, &mut stream3);
    }

    #[test]
    fn one_or_more() {
        let symbol = Symbol::one_or_more(&Symbol::one_of_chars("abc"));
        let mut stream = CharStream::from("abaabaca");
        should_parse(&symbol, &mut stream);
    }
}
