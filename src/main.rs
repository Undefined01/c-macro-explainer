use std::io::Read;

fn main() {
    // Read lines until EOF
    let mut input = String::new();
    let mut line = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let result = c_macro_explainer::preprocess(&input);
    println!("Preprocessed code:\n{}", result);
}
