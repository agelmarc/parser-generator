use syntax::input::CharStream;
use syntax::json_new::json;
use syntax::parse::parse;

fn main() {
    let root_symbol = json();
    let mut stream = CharStream::from(r#"[[[1]]]"#);

    println!("{:#?}", parse(&root_symbol, &mut stream));
}
