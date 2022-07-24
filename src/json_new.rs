use std::borrow::BorrowMut;

use crate::parse::Symbol;
#[allow(non_snake_case)]

pub fn json() -> Symbol {
    // Literals
    let TRUE = Symbol::sequence_chars("true").node_name("True").raw();
    let FALSE = Symbol::sequence_chars("false");
    let NULL = Symbol::sequence_chars("null");

    // Number
    let DIGIT_NONZERO = Symbol::one_of_chars("123456789");
    let DIGIT = Symbol::one_of_chars("0123456789");
    let NUMBER_NONZERO =
        Symbol::sequence(vec![&DIGIT_NONZERO, &Symbol::zero_or_more(&DIGIT)]);
    let NUMBER_F = Symbol::sequence(vec![
        &Symbol::terminal('.'),
        &Symbol::one_or_more(&DIGIT),
    ]);
    let NUMBER_E = Symbol::sequence(vec![
        &Symbol::one_of_chars("Ee"),
        &Symbol::optional(&Symbol::one_of_chars("-=")),
        &Symbol::one_or_more(&DIGIT),
    ]);
    let NUMBER = Symbol::sequence(vec![
        &Symbol::optional(&Symbol::terminal('-')),
        &Symbol::one_of(vec![&Symbol::terminal('0'), &NUMBER_NONZERO]),
        &Symbol::optional(&NUMBER_F),
        &Symbol::optional(&NUMBER_E),
    ])
    .node_name("Number")
    .raw();

    let STRING =
        Symbol::sequence(vec![&Symbol::terminal('"'), &Symbol::terminal('"')]);

    let VALUE = Symbol::one_of(vec![&NUMBER, &TRUE, &FALSE, &NULL]);

    let ARRAY = Symbol::one_of(vec![
        &Symbol::sequence(vec![&Symbol::terminal('['), &Symbol::terminal(']')]),
        &Symbol::sequence(vec![
            &Symbol::terminal('['),
            &VALUE,
            &Symbol::zero_or_more(&Symbol::sequence(vec![
                &Symbol::terminal(','),
                &VALUE,
            ])),
            &Symbol::terminal(']'),
        ]),
    ])
    .node_name("Array");

    let PROPERTY =
        Symbol::sequence(vec![&NUMBER, &Symbol::terminal(':'), &VALUE]);

    let OBJECT = Symbol::one_of(vec![
        &Symbol::sequence(vec![&Symbol::terminal('{'), &Symbol::terminal('}')]),
        &Symbol::sequence(vec![
            &Symbol::terminal('{'),
            &PROPERTY,
            &Symbol::one_or_more(&Symbol::sequence(vec![
                &Symbol::terminal(','),
                &PROPERTY,
            ])),
            &Symbol::terminal('}'),
        ]),
    ]);

    {
        let mut value_inner = VALUE.0.as_ref().borrow_mut();
        match &mut value_inner.symbol_type {
            crate::parse::SymbolType::Sequence(_) => todo!(),
            crate::parse::SymbolType::OneOf(a) => {
                a.push(ARRAY.clone());
                a.push(OBJECT.clone())
            }
            crate::parse::SymbolType::Optional(_) => todo!(),
            crate::parse::SymbolType::OneOrMore(_) => todo!(),
            crate::parse::SymbolType::ZeroOrMore(_) => todo!(),
            crate::parse::SymbolType::Terminal(_) => todo!(),
        };
    }
    let ROOT_NODE = VALUE.node_name("Value");
    return ROOT_NODE;
}
