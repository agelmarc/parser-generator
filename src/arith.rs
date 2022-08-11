use crate::parse::Parser;

const BNF: &str = r#"

ROOT(root) = [ EXPR ];

EXPR(ignore) = PLUS | NUMBER;

PLUS = EXPR_PLUS '+' EXPR_PLUS;

EXPR_PLUS = NUMBER;
NUMBER(raw) = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9';
"#;

pub fn arith_parser() -> Parser {
    Parser::from_bnf(BNF)
}
