use petgraph::graph::NodeIndex;
use petgraph::Graph;

use crate::input::CharStream;
use crate::input::Range;

#[derive(Debug, Default)]
pub enum NodeType {
    In,
    #[default]
    Junction,
    Literal(char),
    Choice(Vec<char>, bool),
}

#[derive(Debug, Default)]
pub struct Node {
    pub node_type: NodeType,
    pub can_end: bool,
}

impl Node {
    fn try_advance(&self, stream: &mut CharStream) -> AdvanceResult {
        match &self.node_type {
            NodeType::In => unreachable!(),
            NodeType::Junction => return AdvanceResult::More(self.can_end),
            NodeType::Literal(c) => {
                if let Some(next_char) = stream.peek() {
                    if c == next_char {
                        stream.next();
                        return AdvanceResult::More(self.can_end);
                    } else {
                        return AdvanceResult::Err;
                    }
                } else {
                    return AdvanceResult::Err;
                }
            }
            NodeType::Choice(chars, b) => {
                if let Some(next_char) = stream.peek() {
                    if b ^ chars.contains(next_char) {
                        stream.next();
                        return AdvanceResult::More(self.can_end);
                    } else {
                        return AdvanceResult::Err;
                    }
                } else {
                    return AdvanceResult::Err;
                }
            }
        }
    }
}

#[derive(Debug)]
enum AdvanceResult {
    More(bool),
    Err,
}

#[derive(Debug)]
pub struct TokenType {
    pub graph: Graph<Node, ()>,
    pub in_node: NodeIndex,
}

impl TokenType {
    pub fn verify(&self, stream: &mut CharStream) -> Result<Token, ()> {
        let mut curr_index = self.in_node;
        let mut can_end = false;

        stream.save();

        'iter: loop {
            let possible_nodes = self.graph.neighbors(curr_index);

            for node in possible_nodes {
                let w = self.graph.node_weight(node).unwrap();
                match w.try_advance(stream) {
                    AdvanceResult::More(e) => {
                        can_end = e;
                        curr_index = node;
                        continue 'iter;
                    }
                    AdvanceResult::Err => {
                        continue;
                    }
                }
            }
            if can_end {
                let (raw, range) = stream.since_save().unwrap();
                return Ok(Token {
                    // token_type: self,
                    range,
                    raw: raw.iter().collect(),
                });
            }
            stream.rewind();
            return Err(());
        }
    }
}

// Constructors
impl TokenType {
    pub fn new() -> Self {
        // let mut graph = Graph::new();
        let mut graph = Graph::with_capacity(1_000_000, 1_000_000);
        let in_node = graph.add_node(Node {
            node_type: NodeType::In,
            can_end: false,
        });
        Self { graph, in_node }
    }

    pub fn add_node_literal(&mut self, seq: char, can_end: bool) -> NodeIndex {
        self.graph.add_node(Node {
            node_type: NodeType::Literal(seq),
            can_end,
        })
    }

    pub fn add_node_choice(
        &mut self,
        seq: Vec<char>,
        can_end: bool,
        exclude: bool,
    ) -> NodeIndex {
        self.graph.add_node(Node {
            node_type: NodeType::Choice(seq, exclude),
            can_end,
        })
    }

    pub fn one_or_more_of(seq: &[char]) -> Self {
        let mut this = Self::new();
        let choice = this.add_node_choice(
            seq.into_iter().copied().collect(),
            true,
            false,
        );
        this.graph
            .extend_with_edges(&[(this.in_node, choice), (choice, choice)]);
        this
    }

    pub fn literal(seq: &str) -> Self {
        let mut this = Self::new();
        let mut prev_node = this.in_node;

        for c in seq.chars() {
            let node = this.add_node_literal(c, false);
            this.graph.add_edge(prev_node, node, ());
            prev_node = node;
        }
        this.graph[prev_node].can_end = true;

        this
    }
}

#[derive(Debug)]
pub struct TokenStream {
    char_stream: CharStream,
    tokens: Vec<TokenType>,
}

impl TokenStream {
    pub fn new(input: &str) -> Self {
        Self {
            char_stream: CharStream::from(input),
            tokens: Vec::new(),
        }
    }
    pub fn register_token(mut self, token: TokenType) -> Self {
        self.tokens.push(token);
        self
    }
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct Token {
    // token_type: TokenType,
    pub range: Range,
    pub raw: String,
}

#[derive(Debug)]
pub enum TokenResult {
    Token(Token),
    UnexpectedSymbol,
}

impl Iterator for TokenStream {
    type Item = TokenResult;
    fn next(&mut self) -> Option<Self::Item> {
        for token_type in &self.tokens {
            match token_type.verify(&mut self.char_stream) {
                Ok(token) => return Some(TokenResult::Token(token)),
                Err(_) => continue,
            };
        }
        if let Some(_) = self.char_stream.peek() {
            Some(TokenResult::UnexpectedSymbol)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Position;

    #[test]
    fn literal() {
        let token = TokenType::literal("test");
        let mut test_stream = CharStream::from("test");

        assert!(matches!(token.verify(&mut test_stream), Ok(_)));
    }

    #[test]
    fn literal_rewind() {
        let token = TokenType::literal("test");
        let mut test_stream = CharStream::from("tesa");

        assert!(matches!(token.verify(&mut test_stream), Err(())));
        assert_eq!(Position(1, 1), test_stream.get_pos());
    }

    #[test]
    fn literal_multi() {
        let token1 = TokenType::literal("apfel");
        let token2 = TokenType::literal("baum");
        let mut test_stream = CharStream::from("apfelbaum");

        assert!(matches!(token1.verify(&mut test_stream), Ok(_)));
        assert!(matches!(token2.verify(&mut test_stream), Ok(_)));
    }

    #[test]
    fn literal_multi_rewind() {
        let token1 = TokenType::literal("apfel");
        let token2 = TokenType::literal("baum");
        let mut test_stream = CharStream::from("apfelbauer");

        assert!(matches!(token1.verify(&mut test_stream), Ok(_)));
        assert!(matches!(token2.verify(&mut test_stream), Err(())));
        assert_eq!(Position(1, 6), test_stream.get_pos());
    }
    #[test]
    fn whitespace() {
        let literal = TokenType::literal("a");
        let mut whitespace = TokenType::new();
        let space = whitespace.add_node_literal(' ', true);
        let lf = whitespace.add_node_literal('\n', true);

        whitespace.graph.extend_with_edges(&[
            (whitespace.in_node, space),
            (whitespace.in_node, lf),
            (space, space),
            (space, lf),
            (lf, space),
            (lf, lf),
        ]);

        let mut test_stream = CharStream::from("a  \n\na\n ");

        literal.verify(&mut test_stream).unwrap();
        whitespace.verify(&mut test_stream).unwrap();
        literal.verify(&mut test_stream).unwrap();
        whitespace.verify(&mut test_stream).unwrap();
    }
}
