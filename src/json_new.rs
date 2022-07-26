use crate::parse::Parser;

const JSON_BNF: &str = r#"
ROOT(root) = [ VALUE ];
VALUE(ignore) = $WHITESPACE (STRING | NUMBER | OBJECT | ARRAY | TRUE | FALSE | NULL) $WHITESPACE;

TRUE(raw) = 't' 'r' 'u' 'e';
FALSE(raw) = 'f' 'a' 'l' 's' 'e';
NULL(raw) = 'n' 'u' 'l' 'l';

NUMBER(raw) = [ '-' ] ( '0' | NUMBER_NONZERO ) [ NUMBER_F ] [ NUMBER_E ];
NUMBER_NONZERO(ignore) = DIGIT_NONZERO { DIGIT };
NUMBER_F(ignore) = '.' { DIGIT };
NUMBER_E(ignore) = ( 'E' | 'e' ) [ '-' | '+' ] { DIGIT };
DIGIT(ignore) = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9';
DIGIT_NONZERO(ignore) = '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9';

STRING(raw) = '"' { * | ESCAPED } '"';
ESCAPED(ignore) = '\' ('"' | '\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | ('u' HEX_DIGIT HEX_DIGIT HEX_DIGIT HEX_DIGIT));
HEX_DIGIT(ignore) = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F';

ARRAY = ( '[' $WHITESPACE ']' ) | ( '[' VALUE { ',' VALUE } ']' );

OBJECT = ( '{' $WHITESPACE '}' ) | ( '{' PROPERTY { ',' PROPERTY } '}' );
PROPERTY = $WHITESPACE STRING $WHITESPACE ':' VALUE;

"#;

pub fn json_parser() -> Parser {
    Parser::from_bnf(JSON_BNF)
}
