# Rust Parser Generator

A parser generator written in Rust. This library uses a [BNF](https://en.wikipedia.org/wiki/Backus%E2%80%93Naur%20form)-like syntax for defining grammars. A Parser for a given grammar defined in a string `bnf_string` can be constructed by `Parser::from_bnf(&bnf_string)`.

As an example, the [grammar of JSON](https://www.json.org/) can be defined as

```rs
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

```

The Parser can be used by

```rs

let json_parser = Parser::from_bnf(JSON_BNF);
let input_str = "{\"key\":3}";
let result = json_parser.parse(&mut CharStream::from(input_str));
println!("{:#?}", result);

```

and outputs

```
Ok(
    Node {
        node_type: "ROOT",
        loc: 1:1 to 1:10,
        data: Children(
            [
                Node {
                    node_type: "OBJECT",
                    loc: 1:1 to 1:10,
                    data: Children(
                        [
                            Node {
                                node_type: "PROPERTY",
                                loc: 1:2 to 1:9,
                                data: Children(
                                    [
                                        Node {
                                            node_type: "STRING",
                                            loc: 1:2 to 1:7,
                                            data: Raw(
                                                "\"key\"",
                                            ),
                                        },
                                        Node {
                                            node_type: "NUMBER",
                                            loc: 1:8 to 1:9,
                                            data: Raw(
                                                "3",
                                            ),
                                        },
                                    ],
                                ),
                            },
                        ],
                    ),
                },
            ],
        ),
    },
)
```
