use std::collections::HashMap;

use crate::parse::Node;
use crate::parse::NodeData;
use crate::parse::Parser;
use crate::parse::SymbolIdent;
use crate::parse::SymbolProperties;

#[derive(Debug)]
pub struct ParserBuilder<'a> {
    stmt_registry: HashMap<String, StmtInfo<'a>>,
    parser: Parser,
}

#[derive(Debug)]
enum StmtInfo<'a> {
    NotBuiltYet {
        node: &'a Node,
        raw: bool,
        ignore: bool,
    },
    AlreadyBuild(SymbolIdent),
}

impl<'a> ParserBuilder<'a> {
    pub fn new() -> Self {
        ParserBuilder {
            stmt_registry: HashMap::new(),
            parser: Parser::new(),
        }
    }

    fn build_statement(
        &mut self,
        node: &Node,
        raw: bool,
        ignore: bool,
    ) -> SymbolIdent {
        assert_eq!(node.node_type, "STATEMENT");
        let identifier = get_stmt_identifier(node);
        self.build_expr(get_stmt_expr(node), Some(identifier), raw, ignore)
    }

    pub fn build(mut self, root_node: &'a Node) -> Parser {
        assert_eq!(root_node.node_type, "ROOT");
        let stmts = get_children_of_node(root_node);
        let mut root_node: Option<StmtInfo> = None;
        for stmt in stmts {
            let identifier = get_stmt_identifier(stmt);
            let stmt_info = get_stmt_info(stmt);
            if stmt_info.contains(&"root") {
                root_node = Some(StmtInfo::NotBuiltYet {
                    node: stmt,
                    raw: stmt_info.contains(&"raw"),
                    ignore: stmt_info.contains(&"ignore"),
                })
            }
            self.stmt_registry.insert(
                identifier.to_owned(),
                StmtInfo::NotBuiltYet {
                    node: stmt,
                    raw: stmt_info.contains(&"raw"),
                    ignore: stmt_info.contains(&"ignore"),
                },
            );
        }
        match root_node {
            Some(root_node) => {
                if let StmtInfo::NotBuiltYet { node, raw, ignore } = root_node {
                    let root_ident = self.build_statement(node, raw, ignore);
                    self.parser.root_node(&root_ident);
                }
            }
            None => panic!("No Root Node given"),
        }
        self.parser
    }

    fn build_expr(
        &mut self,
        node: &Node,
        name: Option<&str>,
        raw: bool,
        ignore: bool,
    ) -> SymbolIdent {
        match node.node_type.as_str() {
            "SEQUENCE" => self.build_sequence(node, name, raw, ignore),
            "TERMINAL" => self.build_terminal(node, name, raw, ignore),
            "IDENTIFIER" => self.build_identifier(get_raw_value_of_node(node)),
            "ONE_OF" => self.build_one_of(node, name, raw, ignore),
            "OPTIONAL" => self.build_optional(node, name, raw, ignore),
            "MANY" => self.build_many(node, name, raw, ignore),
            "ANY" => self.build_any(node, name, raw, ignore),
            "WHITESPACE_ID" => {
                let possible_chars = self.parser.one_of_chars(
                    " \n\r\t",
                    SymbolProperties::new("WHITESPACE_CHAR", false, true),
                );
                self.parser.zero_or_more(
                    &possible_chars,
                    SymbolProperties::new("WHITESPACE", false, true),
                )
            }
            a => panic!("Unknown Node Type {}", a),
        }
    }

    fn build_identifier(&mut self, identifier: &str) -> SymbolIdent {
        let stmt_info = self
            .stmt_registry
            .get(identifier)
            .unwrap_or_else(|| panic!("Usage of undeclared identifier {}", identifier));
        match *stmt_info {
            StmtInfo::NotBuiltYet { node, raw, ignore } => {
                self.build_statement(node, raw, ignore)
            }
            StmtInfo::AlreadyBuild(identifier) => identifier,
        }
    }

    fn build_terminal(
        &mut self,
        node: &Node,
        name: Option<&str>,
        _raw: bool,
        _ignore: bool,
    ) -> SymbolIdent {
        assert_eq!(node.node_type, "TERMINAL");
        let char = get_raw_value_of_node(node);
        match name {
            Some(name) => self.parser.terminal(
                char.chars().nth(1).unwrap(),
                SymbolProperties::new(name, true, false),
            ),
            None => self.parser.terminal(char.chars().nth(1).unwrap(), None),
        }
    }

    fn build_any(
        &mut self,
        node: &Node,
        name: Option<&str>,
        raw: bool,
        ignore: bool,
    ) -> SymbolIdent {
        assert_eq!(node.node_type, "ANY");
        match name {
            Some(name) => self.parser.any_except(
                &['"', '\\'],
                SymbolProperties::new(name, raw, ignore),
            ),
            None => self.parser.any_except(&['"', '\\'], None),
        }
    }
    fn build_many(
        &mut self,
        node: &Node,
        name: Option<&str>,
        raw: bool,
        ignore: bool,
    ) -> SymbolIdent {
        assert_eq!(node.node_type, "MANY");
        let children = get_children_of_node(node);
        assert_eq!(children.len(), 1);
        let child = &children[0];

        let id = self.build_expr(child, None, false, true);
        match name {
            Some(name) => self
                .parser
                .zero_or_more(&id, SymbolProperties::new(name, raw, ignore)),
            None => self.parser.zero_or_more(&id, None),
        }
    }
    fn build_optional(
        &mut self,
        node: &Node,
        name: Option<&str>,
        raw: bool,
        ignore: bool,
    ) -> SymbolIdent {
        assert_eq!(node.node_type, "OPTIONAL");
        let children = get_children_of_node(node);
        assert_eq!(children.len(), 1);
        let child = &children[0];

        let id = self.build_expr(child, None, false, true);
        match name {
            Some(name) => self
                .parser
                .optional(&id, SymbolProperties::new(name, raw, ignore)),
            None => self.parser.optional(&id, None),
        }
    }

    fn build_one_of(
        &mut self,
        node: &Node,
        name: Option<&str>,
        raw: bool,
        ignore: bool,
    ) -> SymbolIdent {
        assert_eq!(node.node_type, "ONE_OF");
        let children = get_children_of_node(node);
        let symbol = match name {
            Some(name) => {
                let seq_id = self
                    .parser
                    .one_of(vec![], SymbolProperties::new(name, raw, ignore));
                self.stmt_registry
                    .insert(name.to_owned(), StmtInfo::AlreadyBuild(seq_id));
                seq_id
            }
            None => self.parser.one_of(vec![], None),
        };
        for child in children {
            let id = self.build_expr(child, None, false, true);
            self.parser.add_ident(&symbol, &id)
            // Build the expression.
        }
        symbol
    }
    fn build_sequence(
        &mut self,
        node: &Node,
        name: Option<&str>,
        raw: bool,
        ignore: bool,
    ) -> SymbolIdent {
        assert_eq!(node.node_type, "SEQUENCE");
        let children = get_children_of_node(node);
        let symbol = match name {
            Some(name) => {
                let seq_id = self
                    .parser
                    .sequence(vec![], SymbolProperties::new(name, raw, ignore));
                self.stmt_registry
                    .insert(name.to_owned(), StmtInfo::AlreadyBuild(seq_id));
                seq_id
            }
            None => self.parser.sequence(vec![], None),
        };
        for child in children {
            let id = self.build_expr(child, None, false, true);
            self.parser.add_ident(&symbol, &id)
            // Build the expression.
        }
        symbol
    }
}

pub fn get_children_of_node(node: &Node) -> &[Node] {
    match &node.data {
        NodeData::Children(c) => c,
        NodeData::Raw(_) => panic!("is a raw node"),
    }
}

pub fn get_raw_value_of_node(node: &Node) -> &str {
    match &node.data {
        NodeData::Children(_) => panic!("is not a raw node"),
        NodeData::Raw(s) => s,
    }
}

fn get_stmt_identifier(node: &Node) -> &str {
    assert_eq!(node.node_type, "STATEMENT");
    let children = get_children_of_node(node);
    get_raw_value_of_node(&children[0])
}

fn get_stmt_info(node: &Node) -> Vec<&str> {
    assert_eq!(node.node_type, "STATEMENT");
    let info_node = match &node.data {
        NodeData::Children(c) => {
            if c.len() == 2 {
                return Vec::new();
            }
            &c[1]
        }
        NodeData::Raw(_) => panic!("Statement node is raw"),
    };
    assert_eq!(info_node.node_type, "STMT_INFO");
    let children = get_children_of_node(info_node);
    children
        .iter()
        .map(get_raw_value_of_node)
        .collect()
}

fn get_stmt_expr(node: &Node) -> &Node {
    assert_eq!(node.node_type, "STATEMENT");
    match &node.data {
        NodeData::Children(c) => {
            if c.len() == 2 {
                &c[1]
            } else {
                &c[2]
            }
        }
        NodeData::Raw(_) => panic!("Statement node is raw"),
    }
}
