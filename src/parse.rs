use std::fmt::Debug;

use crate::bnf::bnf_parser;
use crate::build::ParserBuilder;
use crate::input::CharStream;
use crate::input::Position;
use crate::input::Range;

#[derive(Clone, Debug)]
pub enum SymbolType {
    Sequence(Vec<SymbolIdent>),
    OneOf(Vec<SymbolIdent>),
    Optional(SymbolIdent),
    OneOrMore(SymbolIdent),
    ZeroOrMore(SymbolIdent),
    Terminal(char),
    AnyExcept(Vec<char>),
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub symbol_type: SymbolType,
    properties: SymbolProperties,
}

impl Symbol {
    pub fn add_ident(&mut self, ident: &SymbolIdent) {
        match &mut self.symbol_type {
            SymbolType::Sequence(s) => s.push(*ident),
            SymbolType::OneOf(s) => s.push(*ident),
            SymbolType::Optional(_) => panic!(),
            SymbolType::OneOrMore(_) => panic!(),
            SymbolType::ZeroOrMore(_) => panic!(),
            SymbolType::Terminal(_) => panic!(),
            SymbolType::AnyExcept(_) => panic!(),
        };
    }
}

#[derive(Clone, Debug)]
pub struct SymbolProperties {
    node_name: Option<String>,
    ignore: bool,
    raw: bool,
}

impl SymbolProperties {
    pub fn new(node_name: &str, raw: bool, ignore: bool) -> Option<Self> {
        Some(SymbolProperties {
            node_name: Some(node_name.to_owned()),
            raw,
            ignore,
        })
    }
}

impl Default for SymbolProperties {
    fn default() -> Self {
        Self {
            node_name: None,
            raw: false,
            ignore: true,
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub node_type: String,
    pub loc: Range,
    pub data: NodeData,
}

#[derive(Debug)]
pub enum NodeData {
    Children(Vec<Node>),
    Raw(String),
}

impl Node {
    pub fn new(node_type: &str, loc: Range, data: NodeData) -> Self {
        Node {
            node_type: node_type.to_owned(),
            loc,
            data,
        }
    }
}

pub enum AdvanceResult {
    NewNode(Node),
    Ok(Vec<Node>),
    Err,
}

pub enum DataResult {
    Data(Vec<Node>),
    Err,
}

fn join_and_wrap(strings: Vec<String>, sep: &str) -> String {
    let should_wrap = strings.len() > 1;
    let mut joined = strings.join(sep);
    if should_wrap {
        joined.insert_str(0, "( ");
        joined.push_str(" )");
    }
    joined
}

impl Symbol {
    pub fn repr(&self, p: &Parser) -> String {
        match &self.symbol_type {
            SymbolType::AnyExcept(_) => "ANY".to_owned(),
            SymbolType::Sequence(s) => {
                let reprs = s
                    .iter()
                    .map(|id| p.get_symbol(id))
                    .map(|sym| {
                        if let Some(name) = &sym.properties.node_name {
                            name.clone()
                        } else {
                            sym.repr(p)
                        }
                    })
                    .collect::<Vec<String>>();
                join_and_wrap(reprs, " ")
            }
            SymbolType::OneOf(s) => {
                let reprs = s
                    .iter()
                    .map(|id| p.get_symbol(id))
                    .map(|sym| {
                        if let Some(name) = &sym.properties.node_name {
                            name.clone()
                        } else {
                            sym.repr(p)
                        }
                    })
                    .collect::<Vec<String>>();
                join_and_wrap(reprs, " | ")
            }
            SymbolType::Optional(id) => {
                let asd = p.get_symbol(id);

                format!(
                    "[ {} ]",
                    if let Some(name) = &asd.properties.node_name {
                        name.clone()
                    } else {
                        p.get_symbol(id).repr(p)
                    }
                )
            }
            SymbolType::OneOrMore(id) => {
                format!("{{ {} }}+", p.get_symbol(id).repr(p))
            }
            SymbolType::ZeroOrMore(id) => {
                format!("{{ {} }}", p.get_symbol(id).repr(p))
            }
            SymbolType::Terminal(c) => format!("\'{}\'", c.escape_debug()),
        }
    }
}

impl Symbol {
    fn name(&self) -> String {
        if let Some(name) = &self.properties.node_name {
            name.to_owned()
        } else {
            match self.symbol_type {
                SymbolType::Sequence(_) => "Sequence".to_owned(),
                SymbolType::OneOf(_) => "OneOf".to_owned(),
                SymbolType::Optional(_) => "Optional".to_owned(),
                SymbolType::OneOrMore(_) => "OneOrMore".to_owned(),
                SymbolType::ZeroOrMore(_) => "ZeroOrMore".to_owned(),
                SymbolType::Terminal(_) => "Terminal".to_owned(),
                SymbolType::AnyExcept(_) => "AnyExcept".to_owned(),
            }
        }
    }

    fn try_advance(
        &self,
        stream: &mut CharStream,
        p: &Parser,
    ) -> AdvanceResult {
        let start_pos = stream.get_pos();
        let data_result: DataResult = match &self.symbol_type {
            SymbolType::Sequence(s) => {
                Symbol::try_advance_sequence(s, stream, p)
            }
            SymbolType::OneOf(s) => Symbol::try_advance_one_of(s, stream, p),
            SymbolType::Optional(s) => {
                Symbol::try_advance_optional(s, stream, p)
            }
            SymbolType::OneOrMore(s) => {
                Symbol::try_advance_one_or_more(s, stream, p)
            }
            SymbolType::ZeroOrMore(s) => {
                Symbol::try_advance_zero_or_more(s, stream, p)
            }
            SymbolType::Terminal(c) => {
                Symbol::try_advance_terminal(c, stream, p)
            }
            SymbolType::AnyExcept(c) => {
                Symbol::try_advance_any_except(c, stream, p)
            }
        };
        return match data_result {
            DataResult::Data(node) => {
                // Got New Node. If the current symbol is ignored in AST,
                // Only Forward the Data contained in that node.
                if self.properties.ignore {
                    AdvanceResult::Ok(node)
                } else {
                    let (raw, range) = stream.since_pos(start_pos);
                    let data = if self.properties.raw {
                        NodeData::Raw(raw.iter().collect())
                    } else {
                        NodeData::Children(node)
                    };
                    AdvanceResult::NewNode(Node::new(&self.name(), range, data))
                }
            }
            DataResult::Err => {
                stream.set_pos(start_pos);
                AdvanceResult::Err
            }
        };
    }

    fn try_advance_one_or_more(
        symbol: &SymbolIdent,
        stream: &mut CharStream,
        p: &Parser,
    ) -> DataResult {
        let mut once = false;
        let mut data = Vec::new();
        loop {
            match p.get_symbol(symbol).try_advance(stream, p) {
                AdvanceResult::NewNode(node) => {
                    if !once {
                        once = true;
                    }
                    data.push(node)
                }
                AdvanceResult::Ok(mut nodes) => {
                    if !once {
                        once = true
                    };
                    data.append(&mut nodes)
                }
                AdvanceResult::Err => {
                    return if once {
                        DataResult::Data(data)
                    } else {
                        DataResult::Err
                    };
                }
            }
        }
    }

    fn try_advance_zero_or_more(
        symbol: &SymbolIdent,
        stream: &mut CharStream,
        p: &Parser,
    ) -> DataResult {
        let mut data = Vec::new();
        loop {
            match p.get_symbol(symbol).try_advance(stream, p) {
                AdvanceResult::NewNode(node) => data.push(node),
                AdvanceResult::Ok(mut nodes) => data.append(&mut nodes),
                AdvanceResult::Err => return DataResult::Data(data),
            }
        }
    }

    fn try_advance_optional(
        symbol: &SymbolIdent,
        stream: &mut CharStream,
        p: &Parser,
    ) -> DataResult {
        return match p.get_symbol(symbol).try_advance(stream, p) {
            AdvanceResult::NewNode(node) => DataResult::Data(vec![node]),
            AdvanceResult::Ok(nodes) => DataResult::Data(nodes),
            AdvanceResult::Err => DataResult::Data(vec![]),
        };
    }

    fn try_advance_one_of(
        symbols: &[SymbolIdent],
        stream: &mut CharStream,
        p: &Parser,
    ) -> DataResult {
        for symbol in symbols {
            match p.get_symbol(symbol).try_advance(stream, p) {
                AdvanceResult::NewNode(node) => {
                    return DataResult::Data(vec![node])
                }
                AdvanceResult::Ok(nodes) => return DataResult::Data(nodes),
                AdvanceResult::Err => continue,
            }
        }
        DataResult::Err
    }

    fn try_advance_sequence(
        symbols: &[SymbolIdent],
        stream: &mut CharStream,
        p: &Parser,
    ) -> DataResult {
        let mut data = Vec::new();
        for symbol in symbols {
            let symbol = p.get_symbol(symbol);
            match symbol.try_advance(stream, p) {
                AdvanceResult::Ok(mut nodes) => data.append(&mut nodes),
                AdvanceResult::Err => return DataResult::Err,
                AdvanceResult::NewNode(node) => data.push(node),
            };
        }
        DataResult::Data(data)
    }

    fn try_advance_terminal(
        c: &char,
        stream: &mut CharStream,
        _p: &Parser,
    ) -> DataResult {
        if let Some(next_char) = stream.peek() {
            if c == next_char {
                stream.next();
                DataResult::Data(vec![])
            } else {
                DataResult::Err
            }
        } else {
            DataResult::Err
        }
    }

    fn try_advance_any_except(
        chars: &[char],
        stream: &mut CharStream,
        _p: &Parser,
    ) -> DataResult {
        if let Some(next_char) = stream.peek() {
            if !chars.contains(next_char) {
                stream.next();
                DataResult::Data(vec![])
            } else {
                DataResult::Err
            }
        } else {
            DataResult::Err
        }
    }
}
#[derive(Debug)]
pub struct Parser {
    symbol_registry: Vec<Symbol>,
    root_node: Option<SymbolIdent>,
}

#[derive(Copy, Clone, Debug)]
pub struct SymbolIdent(pub usize);

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            symbol_registry: Vec::new(),
            root_node: None,
        }
    }

    pub fn from_bnf(bnf: &str) -> Self {
        let mut stream = CharStream::from(bnf);
        let bnf_parser = bnf_parser();
        let root_node = match bnf_parser.parse(&mut stream) {
            ParseResult::Ok(n) => n,
            ParseResult::Err(_) => panic!("BNF Parse failed"),
        };
        let parser_builder = ParserBuilder::new();
        parser_builder.build(&root_node)
    }

    pub fn get_symbol(&self, idx: &SymbolIdent) -> &Symbol {
        &self.symbol_registry[idx.0]
    }

    pub fn get_symbol_mut(&mut self, idx: &SymbolIdent) -> &mut Symbol {
        &mut self.symbol_registry[idx.0]
    }

    pub fn insert_symbol(&mut self, symbol: Symbol) -> SymbolIdent {
        let idx = self.symbol_registry.len();
        self.symbol_registry.push(symbol);
        SymbolIdent(idx)
    }
    pub fn root_node(&mut self, root: &SymbolIdent) {
        // if self.get_symbol(root).properties.ignore {
        //     panic!("Root symbol cant be ignored")
        // }
        self.root_node = Some(*root);
    }

    pub fn parse(&self, stream: &mut CharStream) -> ParseResult {
        match self
            .get_symbol(&self.root_node.expect("No root node defined"))
            .try_advance(stream, self)
        {
            AdvanceResult::Ok(_) => {
                panic!("Root node is set to be ignored");
            }
            AdvanceResult::Err => ParseResult::Err(stream.get_pos()),
            AdvanceResult::NewNode(n) => {
                return match stream.peek() {
                    Some(_) => ParseResult::Err(stream.get_pos()),
                    None => ParseResult::Ok(n),
                }
            }
        }
    }
}

impl Parser {
    pub fn terminal(
        &mut self,
        char: char,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };

        self.insert_symbol(Symbol {
            symbol_type: SymbolType::Terminal(char),
            properties,
        })
    }

    pub fn any_except(
        &mut self,
        chars: &[char],
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };

        self.insert_symbol(Symbol {
            symbol_type: SymbolType::AnyExcept(chars.to_vec()),
            properties,
        })
    }
    pub fn one_of(
        &mut self,
        symbols: Vec<&SymbolIdent>,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };
        self.insert_symbol(Symbol {
            symbol_type: SymbolType::OneOf(
                symbols.into_iter().copied().collect(),
            ),
            properties,
        })
    }

    pub fn add_ident(
        &mut self,
        ident: &SymbolIdent,
        to_be_added: &SymbolIdent,
    ) {
        let symbol = self.get_symbol_mut(ident);
        symbol.add_ident(to_be_added);
    }

    pub fn sequence(
        &mut self,
        symbols: Vec<&SymbolIdent>,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };
        self.insert_symbol(Symbol {
            symbol_type: SymbolType::Sequence(
                symbols.into_iter().copied().collect(),
            ),
            properties,
        })
    }
    pub fn optional(
        &mut self,
        symbol: &SymbolIdent,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };
        self.insert_symbol(Symbol {
            symbol_type: SymbolType::Optional(*symbol),
            properties,
        })
    }
    pub fn one_or_more(
        &mut self,
        symbol: &SymbolIdent,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };

        self.insert_symbol(Symbol {
            symbol_type: SymbolType::OneOrMore(*symbol),
            properties,
        })
    }
    pub fn zero_or_more(
        &mut self,
        symbol: &SymbolIdent,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };

        self.insert_symbol(Symbol {
            symbol_type: SymbolType::ZeroOrMore(*symbol),
            properties,
        })
    }
    pub fn sequence_chars(
        &mut self,
        string: &str,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };
        let chars = string.chars();
        let symbol_type = SymbolType::Sequence(
            chars.map(|c| self.terminal(c, None)).collect(),
        );
        self.insert_symbol(Symbol {
            symbol_type,
            properties,
        })
    }
    pub fn one_of_chars(
        &mut self,
        string: &str,
        props: Option<SymbolProperties>,
    ) -> SymbolIdent {
        let chars = string.chars();
        let properties = if let Some(p) = props {
            p
        } else {
            Default::default()
        };
        let symbol_type =
            SymbolType::OneOf(chars.map(|c| self.terminal(c, None)).collect());
        self.insert_symbol(Symbol {
            symbol_type,
            properties,
        })
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

    #[test]
    fn literal() {
        let mut parser = Parser::new();
        let symbol = parser
            .sequence_chars("test", SymbolProperties::new("", false, false));
        parser.root_node(&symbol);

        let mut stream = CharStream::from("test");

        assert!(matches!(parser.parse(&mut stream), ParseResult::Ok(_)));
        assert_eq!((1, 5), stream.get_loc());
    }

    #[test]
    fn no_eof_after_parse() {
        let mut parser = Parser::new();
        let symbol =
            parser.terminal('a', SymbolProperties::new("", false, false));
        parser.root_node(&symbol);

        let mut stream = CharStream::from("ab");

        assert!(matches!(parser.parse(&mut stream), ParseResult::Err(_)));
    }

    #[test]
    fn literal_rewind() {
        let mut parser = Parser::new();
        let symbol = parser
            .sequence_chars("test", SymbolProperties::new("", false, false));
        parser.root_node(&symbol);

        let mut stream = CharStream::from("tesa");

        assert!(matches!(parser.parse(&mut stream), ParseResult::Err(_)));
        assert_eq!((1, 1), stream.get_loc());
    }

    #[test]
    fn one_of_chars() {
        let mut parser = Parser::new();
        let symbol =
            parser.one_of_chars("abc", SymbolProperties::new("", true, false));
        parser.root_node(&symbol);

        let mut stream = CharStream::from("a");

        assert!(matches!(parser.parse(&mut stream), ParseResult::Ok(_)));
    }

    #[test]
    fn one_of() {
        let mut parser = Parser::new();
        // { 'a' }+ | { 'b' }+
        let a = parser.terminal('a', None);
        let b = parser.terminal('b', None);

        let a_multi = parser.one_or_more(&a, None);
        let b_multi = parser.one_or_more(&b, None);

        let symbol = parser.one_of(
            vec![&a_multi, &b_multi],
            SymbolProperties::new("", false, false),
        );
        parser.root_node(&symbol);

        let mut stream1 = CharStream::from("aaa");
        let mut stream2 = CharStream::from("bbb");

        assert!(matches!(parser.parse(&mut stream1), ParseResult::Ok(_)));
        assert!(matches!(parser.parse(&mut stream2), ParseResult::Ok(_)));
    }

    #[test]
    fn optional() {
        let mut parser = Parser::new();
        let minus = parser.terminal('-', None);
        let symbol =
            parser.optional(&minus, SymbolProperties::new("", false, false));
        parser.root_node(&symbol);

        let _stream1 = CharStream::from("-");
        let _stream2 = CharStream::from("");
        let mut stream3 = CharStream::from("a");

        // assert!(matches!(parser.parse(&mut stream1), ParseResult::Ok(_)));
        // assert!(matches!(parser.parse(&mut stream2), ParseResult::Ok(_)));
        assert!(matches!(parser.parse(&mut stream3), ParseResult::Err(_)));
    }

    #[test]
    fn one_or_more() {
        let mut parser = Parser::new();
        let abc = parser.one_of_chars("abc", None);
        let symbol =
            parser.one_or_more(&abc, SymbolProperties::new("", false, false));
        parser.root_node(&symbol);

        let mut stream = CharStream::from("abaabaca");

        assert!(matches!(parser.parse(&mut stream), ParseResult::Ok(_)));
    }
}
