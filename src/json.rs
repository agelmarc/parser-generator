use crate::parse::Parser;
use crate::parse::Symbol;
use crate::parse::SymbolProperties;
#[allow(non_snake_case)]

pub fn json() -> Parser {
    // Literals
    let mut parser = Parser::new();
    let TRUE = parser
        .sequence_chars("true", SymbolProperties::new("TRUE", true, false)); //.node_name("True").raw();
    let FALSE = parser
        .sequence_chars("false", SymbolProperties::new("FALSE", true, false));
    let NULL = parser
        .sequence_chars("null", SymbolProperties::new("NULL", true, false));
    let WHITESPACE_CHAR = parser.one_of_chars(" \n\r\t", None);
    let WHITESPACE = parser.zero_or_more(
        &WHITESPACE_CHAR,
        SymbolProperties::new("WHITESPACE", true, true),
    );
    // Number
    let DIGIT_NONZERO = parser.one_of_chars("123456789", None);
    let DIGIT = parser.one_of_chars("0123456789", None);
    let TAIL = parser.zero_or_more(&DIGIT, None);
    let NUMBER_NONZERO = parser.sequence(
        vec![&DIGIT_NONZERO, &TAIL],
        SymbolProperties::new("NUMBER_NONZERO", false, true),
    );
    let DOT = parser.terminal('.', None);
    let TAIL_F = parser.one_or_more(&DIGIT, None);
    let NUMBER_F = parser.sequence(
        vec![&DOT, &TAIL_F],
        SymbolProperties::new("NUMBER_F", false, true),
    );
    let E = parser.one_of_chars("Ee", None);
    let SIGN = parser.one_of_chars("-+", None);
    let SIGN_OPT = parser.optional(&SIGN, None);

    let NUMBER_E = parser.sequence(
        vec![&E, &SIGN_OPT, &TAIL_F],
        SymbolProperties::new("NUMBER_E", false, true),
    );
    let MINUS = parser.terminal('-', None);
    let MINUS_OPT = parser.optional(&MINUS, None);
    let F_OPT = parser.optional(&NUMBER_F, None);
    let E_OPT = parser.optional(&NUMBER_E, None);
    let ZERO = parser.terminal('0', None);
    let N = parser.one_of(vec![&ZERO, &NUMBER_NONZERO], None);
    let NUMBER = parser.sequence(
        vec![&MINUS_OPT, &N, &F_OPT, &E_OPT],
        SymbolProperties::new("NUMBER", true, false),
    );

    let STRING_DELIM = parser.terminal('"', None);
    let CODEPOINT = parser.any_except(&['"', '\\'], None);
    let BACKSLASH = parser.terminal('\\', None);
    let ESCAPE_CHAR = parser.one_of_chars("\"\\/bfnrt", None);
    let ESCAPED = parser.sequence(vec![&BACKSLASH, &ESCAPE_CHAR], None);
    let STRING_CONTENT = parser.one_of(vec![&CODEPOINT, &ESCAPED], None);
    let BLA = parser.zero_or_more(&STRING_CONTENT, None);
    let STRING = parser.sequence(
        vec![&STRING_DELIM, &BLA, &STRING_DELIM],
        SymbolProperties::new("STRING", true, false),
    );

    let VALUE_TYPE =
        parser.one_of(vec![&NUMBER, &TRUE, &FALSE, &NULL, &STRING], None);
    let VALUE = parser.sequence(
        vec![&WHITESPACE, &VALUE_TYPE, &WHITESPACE],
        SymbolProperties::new("VALUE", false, true),
    );
    let ARRAY_BEGIN = parser.terminal('[', None);
    let ARRAY_END = parser.terminal(']', None);
    let ARRAY_EMPTY =
        parser.sequence(vec![&ARRAY_BEGIN, &WHITESPACE, &ARRAY_END], None);
    let ARRAY_DELIM = parser.terminal(',', None);
    let ARRAY_ITEM = parser.sequence(vec![&ARRAY_DELIM, &VALUE], None);
    let ARRAY_ITEM_OPT = parser.zero_or_more(&ARRAY_ITEM, None);

    let ARRAY_NONEMPTY = parser.sequence(
        vec![&ARRAY_BEGIN, &VALUE, &ARRAY_ITEM_OPT, &ARRAY_END],
        None,
    );
    let ARRAY = parser.one_of(
        vec![&ARRAY_EMPTY, &ARRAY_NONEMPTY],
        SymbolProperties::new("ARRAY", false, false),
    );

    let PROP_DELIM = parser.terminal(':', None);
    let PROP = parser.sequence(
        vec![&WHITESPACE, &STRING, &WHITESPACE, &PROP_DELIM, &VALUE],
        SymbolProperties::new("PROPERTY", false, false),
    );

    let OBJ_BEGIN = parser.terminal('{', None);
    let OBJ_END = parser.terminal('}', None);
    let OBJ_ITEM = parser.sequence(vec![&ARRAY_DELIM, &PROP], None);
    let OBJ_ITEM_OPT = parser.zero_or_more(&OBJ_ITEM, None);
    let OBJ_EMPTY =
        parser.sequence(vec![&OBJ_BEGIN, &WHITESPACE, &OBJ_END], None);
    let OBJ_NONEMPTY =
        parser.sequence(vec![&OBJ_BEGIN, &PROP, &OBJ_ITEM_OPT, &OBJ_END], None);
    let OBJECT = parser.one_of(
        vec![&OBJ_EMPTY, &OBJ_NONEMPTY],
        SymbolProperties::new("OBJECT", false, false),
    );

    let val = parser.get_symbol_mut(&VALUE_TYPE);
    match &mut val.symbol_type {
        crate::parse::SymbolType::Sequence(_) => todo!(),
        crate::parse::SymbolType::OneOf(a) => {
            a.push(ARRAY.clone());
            a.push(OBJECT.clone())
        }
        crate::parse::SymbolType::Optional(_) => todo!(),
        crate::parse::SymbolType::OneOrMore(_) => todo!(),
        crate::parse::SymbolType::ZeroOrMore(_) => todo!(),
        crate::parse::SymbolType::Terminal(_) => todo!(),
        crate::parse::SymbolType::AnyExcept(_) => todo!(),
    }
    let ROOT = parser
        .sequence(vec![&VALUE], SymbolProperties::new("ROOT", false, false));
    parser.root_node(&ROOT);
    return parser;
}
