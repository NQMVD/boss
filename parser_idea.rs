
// let command_output = "nu-parser = \"0.1.0\"    # nushell parser";
// let parse_pattern = "{name} = \"{version}\"    # {description}";
// let result = parse(command_output, parse_pattern).unwrap();

// approach:
// - identify the delimiters between the braces enclosed values
// - split the input string based on the delimiters
// - collect the chunks in a hashmap with the keys as the values enclosed in the braces
// - return the hashmap
fn parse(input: &str, pattern: &str) -> Result<String, String> {
    // identify the delimiters between the braces enclosed values
    let delimiters = vec!['{', '}'];

    // - split the input string based on the delimiters
    let chunks: Vec<&str> = input.split(|c| delimiters.contains(&c)).collect();

    // - collect the chunks in a hashmap with the keys as the values enclosed in the braces
    let mut hashmap = HashMap::new();
    for (i, chunk) in chunks.iter().enumerate() {
        if i % 2 == 0 {
            let key = chunk;
            let value = chunks.get(i + 1).expect("value should exist");
            hashmap.insert(key, value);
        }
    }

    // - return the hashmap
    let mut result = pattern.to_string();
    for (key, value) in hashmap {
        result = result.replace(key, value);
    }

    Err("not implemented".to_string())
}
