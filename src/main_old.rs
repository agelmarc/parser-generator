// fn main_old() {
//     let number_token = number_token();
//     let string_token = string_token();
//     let true_token = TokenType::literal("true");
//     let false_token = TokenType::literal("false");
//     let null_token = TokenType::literal("null");

//     let array_begin = TokenType::literal("[");
//     let array_end = TokenType::literal("]");
//     let delim = TokenType::literal(",");

//     let obj_begin = TokenType::literal("{");
//     let obj_end = TokenType::literal("}");
//     let prop_delim = TokenType::literal(":");

//     let whitespace = whitespace();

//     let string = fs::read_to_string("large-file.json").unwrap();

//     let token_stream = TokenStream::new(&string)
//         .register_token(number_token)
//         .register_token(string_token)
//         .register_token(whitespace)
//         .register_token(true_token)
//         .register_token(false_token)
//         .register_token(null_token)
//         .register_token(array_begin)
//         .register_token(array_end)
//         .register_token(delim)
//         .register_token(obj_begin)
//         .register_token(obj_end)
//         .register_token(prop_delim);
//     for token in token_stream {
//         match token {
//             TokenResult::Token(token) => {}
//             TokenResult::UnexpectedSymbol => break,
//         }
//     }
// }
