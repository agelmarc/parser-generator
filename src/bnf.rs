use crate::parse::Parser;
use crate::parse::SymbolProperties;

#[allow(non_snake_case)]
pub fn bnf_parser() -> Parser {
    let mut parser = Parser::new();

    // TERMINALS
    let TERMINAL_DELIM = parser.terminal('\'', None);
    let STATEMENT_TERM = parser.terminal(';', None);
    let EQUALS = parser.terminal('=', None);
    let IDENT_CHAR = parser.one_of_chars("ABCDEFGHIJKLMNOPQRSTUVWXYZ_", None);
    let LC_CHAR = parser.one_of_chars("abcdefghijklmnopqrstuvwxyz", None);
    let SEP = parser.terminal(' ', None);
    let PIPE = parser.terminal('|', None);
    let COMMA = parser.terminal(',', None);
    let CHAR = parser.any_except(&[], None);
    let OPT_BEGIN = parser.terminal('[', None);
    let OPT_END = parser.terminal(']', None);
    let MANY_BEGIN = parser.terminal('{', None);
    let MANY_END = parser.terminal('}', None);
    let PAR_BEGIN = parser.terminal('(', None);
    let PAR_END = parser.terminal(')', None);
    let ONE_OF_SEP = parser.sequence(vec![&SEP, &PIPE, &SEP], None);
    let WHITESPACE_CHAR = parser.one_of_chars(
        " \n\r\t",
        SymbolProperties::new("WHITESPACE_CHAR", false, true),
    );
    let IDENT_WHITESPACE = parser.sequence_chars(
        "$WHITESPACE",
        SymbolProperties::new("WHITESPACE_ID", true, false),
    );
    let ANY = parser.terminal('*', SymbolProperties::new("ANY", true, false));

    let WHITESPACE = parser.zero_or_more(
        &WHITESPACE_CHAR,
        SymbolProperties::new("WHITESPACE", false, true),
    );

    let IDENTIFIER = parser.one_or_more(
        &IDENT_CHAR,
        SymbolProperties::new("IDENTIFIER", true, false),
    );
    let TERMINAL = parser.sequence(
        vec![&TERMINAL_DELIM, &CHAR, &TERMINAL_DELIM],
        SymbolProperties::new("TERMINAL", true, false),
    );

    // EXPRESSION
    let EXPRESSION = parser.one_of(
        vec![&TERMINAL, &IDENTIFIER, &ANY, &IDENT_WHITESPACE],
        SymbolProperties::new("EXPR", false, true),
    );
    let EXPR_SEQ = parser.one_of(
        vec![&TERMINAL, &IDENTIFIER, &ANY, &IDENT_WHITESPACE],
        SymbolProperties::new("EXPR_SEQ", false, true),
    );
    let EXPR_ONE_OF = parser.one_of(
        vec![&TERMINAL, &IDENTIFIER, &ANY, &IDENT_WHITESPACE],
        SymbolProperties::new("EXPR_ONE_OF", false, true),
    );
    let EXPR_OPT = parser.one_of(
        vec![&TERMINAL, &IDENTIFIER, &ANY, &IDENT_WHITESPACE],
        SymbolProperties::new("EXPR_OPT", false, true),
    );
    let EXPR_MANY = parser.one_of(
        vec![&TERMINAL, &IDENTIFIER, &IDENT_WHITESPACE],
        SymbolProperties::new("EXPR_MANY", false, true),
    );

    // SEQUENCE
    let SEQ_ITEM = parser.sequence(
        vec![&SEP, &EXPR_SEQ],
        SymbolProperties::new("SEQ_ITEM", false, true),
    );
    let SEQ_ITEM_OPT = parser.zero_or_more(
        &SEQ_ITEM,
        SymbolProperties::new("SEQ_ITEM_OPT", false, true),
    );
    let SEQ = parser.sequence(
        vec![&EXPR_SEQ, &SEP, &EXPR_SEQ, &SEQ_ITEM_OPT],
        SymbolProperties::new("SEQUENCE", false, false),
    );
    let SEQ_PAR = parser.sequence(
        vec![&PAR_BEGIN, &WHITESPACE, &SEQ, &WHITESPACE, &PAR_END],
        SymbolProperties::new("SEQ_PAR", false, true),
    );

    // ONE OF
    let ONE_OF_ITEM = parser.sequence(
        vec![&ONE_OF_SEP, &EXPR_ONE_OF],
        SymbolProperties::new("ONE_OF_ITEM", false, true),
    );
    let ONE_OF_ITEM_OPT = parser.zero_or_more(
        &ONE_OF_ITEM,
        SymbolProperties::new("ONE_OF_ITEM_OPT", false, true),
    );
    let ONE_OF = parser.sequence(
        vec![&EXPR_ONE_OF, &ONE_OF_SEP, &EXPR_ONE_OF, &ONE_OF_ITEM_OPT],
        SymbolProperties::new("ONE_OF", false, false),
    );
    let ONE_OF_PAR = parser.sequence(
        vec![&PAR_BEGIN, &WHITESPACE, &ONE_OF, &WHITESPACE, &PAR_END],
        SymbolProperties::new("ONE_OF_PAR", false, true),
    );

    // OPTIONAL
    let OPT = parser.sequence(
        vec![&OPT_BEGIN, &WHITESPACE, &EXPR_OPT, &WHITESPACE, &OPT_END],
        SymbolProperties::new("OPTIONAL", false, false),
    );

    // MANY
    let MANY = parser.sequence(
        vec![&MANY_BEGIN, &WHITESPACE, &EXPR_MANY, &WHITESPACE, &MANY_END],
        SymbolProperties::new("MANY", false, false),
    );
    // STATEMENT
    let STMT_INFO_FLAG = parser
        .one_or_more(&LC_CHAR, SymbolProperties::new("FLAG", true, false));
    let STMT_INFO_FLAG_ITEM =
        parser.sequence(vec![&COMMA, &STMT_INFO_FLAG], None);
    let STMT_INFO_FLAG_ITEM_OPT =
        parser.zero_or_more(&STMT_INFO_FLAG_ITEM, None);
    let STMT_INFO = parser.sequence(
        vec![
            &PAR_BEGIN,
            &STMT_INFO_FLAG,
            &STMT_INFO_FLAG_ITEM_OPT,
            &PAR_END,
        ],
        SymbolProperties::new("STMT_INFO", false, false),
    );
    let STMT_INFO_OPT = parser.optional(&STMT_INFO, None);
    let STATEMENT = parser.sequence(
        vec![
            &WHITESPACE,
            &IDENTIFIER,
            &STMT_INFO_OPT,
            &WHITESPACE,
            &EQUALS,
            &WHITESPACE,
            &EXPRESSION,
            &WHITESPACE,
            &STATEMENT_TERM,
            &WHITESPACE,
        ],
        SymbolProperties::new("STATEMENT", false, false),
    );

    let ROOT = parser
        .one_or_more(&STATEMENT, SymbolProperties::new("ROOT", false, false));

    let val = parser.get_symbol_mut(&EXPRESSION);
    match &mut val.symbol_type {
        crate::parse::SymbolType::Sequence(_) => todo!(),
        crate::parse::SymbolType::OneOf(a) => {
            a.insert(0, OPT);
            a.insert(0, ONE_OF);
            a.insert(0, SEQ);
        }
        crate::parse::SymbolType::Optional(_) => todo!(),
        crate::parse::SymbolType::OneOrMore(_) => todo!(),
        crate::parse::SymbolType::ZeroOrMore(_) => todo!(),
        crate::parse::SymbolType::Terminal(_) => todo!(),
        crate::parse::SymbolType::AnyExcept(_) => todo!(),
    }

    let val = parser.get_symbol_mut(&EXPR_SEQ);
    match &mut val.symbol_type {
        crate::parse::SymbolType::Sequence(_) => todo!(),
        crate::parse::SymbolType::OneOf(a) => {
            a.insert(0, MANY);
            a.insert(0, OPT);
            a.insert(0, ONE_OF_PAR);
        }
        crate::parse::SymbolType::Optional(_) => todo!(),
        crate::parse::SymbolType::OneOrMore(_) => todo!(),
        crate::parse::SymbolType::ZeroOrMore(_) => todo!(),
        crate::parse::SymbolType::Terminal(_) => todo!(),
        crate::parse::SymbolType::AnyExcept(_) => todo!(),
    }

    let val = parser.get_symbol_mut(&EXPR_ONE_OF);
    match &mut val.symbol_type {
        crate::parse::SymbolType::Sequence(_) => todo!(),
        crate::parse::SymbolType::OneOf(a) => {
            a.insert(0, MANY);
            a.insert(0, OPT);
            a.insert(0, SEQ_PAR);
        }
        crate::parse::SymbolType::Optional(_) => todo!(),
        crate::parse::SymbolType::OneOrMore(_) => todo!(),
        crate::parse::SymbolType::ZeroOrMore(_) => todo!(),
        crate::parse::SymbolType::Terminal(_) => todo!(),
        crate::parse::SymbolType::AnyExcept(_) => todo!(),
    }

    let val = parser.get_symbol_mut(&EXPR_OPT);
    match &mut val.symbol_type {
        crate::parse::SymbolType::Sequence(_) => todo!(),
        crate::parse::SymbolType::OneOf(a) => {
            a.insert(0, MANY);
            a.insert(0, ONE_OF);
            a.insert(0, SEQ);
        }
        crate::parse::SymbolType::Optional(_) => todo!(),
        crate::parse::SymbolType::OneOrMore(_) => todo!(),
        crate::parse::SymbolType::ZeroOrMore(_) => todo!(),
        crate::parse::SymbolType::Terminal(_) => todo!(),
        crate::parse::SymbolType::AnyExcept(_) => todo!(),
    }

    let val = parser.get_symbol_mut(&EXPR_MANY);
    match &mut val.symbol_type {
        crate::parse::SymbolType::Sequence(_) => todo!(),
        crate::parse::SymbolType::OneOf(a) => {
            a.insert(0, OPT);
            a.insert(0, ONE_OF);
            a.insert(0, SEQ);
        }
        crate::parse::SymbolType::Optional(_) => todo!(),
        crate::parse::SymbolType::OneOrMore(_) => todo!(),
        crate::parse::SymbolType::ZeroOrMore(_) => todo!(),
        crate::parse::SymbolType::Terminal(_) => todo!(),
        crate::parse::SymbolType::AnyExcept(_) => todo!(),
    }
    parser.root_node(&ROOT);
    parser
}
