use crate::token::TokenType;

pub fn number_token() -> TokenType {
    let mut number_token = TokenType::new();
    let minus = number_token.add_node_literal('-', false);
    let zero = number_token.add_node_literal('0', true);
    let digit_nonzero = number_token.add_node_choice(
        vec!['1', '2', '3', '4', '5', '6', '7', '8', '9'],
        true,
        false,
    );
    let digit = number_token.add_node_choice(
        vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
        true,
        false,
    );
    let digit_e = number_token.add_node_choice(
        vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
        true,
        false,
    );
    let digit_d = number_token.add_node_choice(
        vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
        true,
        false,
    );

    let exp = number_token.add_node_choice(vec!['e', 'E'], false, false);
    let minus_e = number_token.add_node_literal('-', false);
    let plus = number_token.add_node_literal('+', false);
    let dot = number_token.add_node_literal('.', false);

    number_token.graph.extend_with_edges(&[
        (number_token.in_node, minus),
        (number_token.in_node, zero),
        (number_token.in_node, digit_nonzero),
        (minus, zero),
        (minus, digit_nonzero),
        (digit_nonzero, digit),
        (digit, digit),
        // Decimal
        (zero, dot),
        (digit_nonzero, dot),
        (digit, dot),
        (dot, digit_d),
        (digit_d, digit_d),
        //Exponential
        (digit_d, exp),
        (zero, exp),
        (digit_nonzero, exp),
        (digit, exp),
        (exp, minus_e),
        (exp, plus),
        (minus_e, digit_e),
        (plus, digit_e),
        (exp, digit_e),
        (digit_e, digit_e),
    ]);
    number_token
}

pub fn string_token() -> TokenType {
    let mut string = TokenType::new();
    let start_quote = string.add_node_literal('"', false);
    let end_quote = string.add_node_literal('"', true);

    let any = string.add_node_choice(vec!['"', '\\'], false, true);
    let backslash = string.add_node_literal('\\', false);
    let escape = string.add_node_choice(
        vec!['"', '\\', '/', 'b', 'f', 'n', 'r', 't'],
        false,
        false,
    );

    string.graph.extend_with_edges(&[
        (string.in_node, start_quote),
        (start_quote, end_quote),
        (start_quote, any),
        (start_quote, backslash),
        (backslash, escape),
        (any, any),
        (any, backslash),
        (escape, backslash),
        (escape, any),
        (any, end_quote),
        (escape, end_quote),
    ]);

    string
}

pub fn whitespace() -> TokenType {
    TokenType::one_or_more_of(&[' ', '\n', '\r', '\t'])
}
