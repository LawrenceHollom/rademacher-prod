pub fn split_list(text: &str) -> Vec<&str> {
    let mut depth = 0;
    let mut last_index = 0;
    let mut args = vec![];
    for (i, c) in text.as_bytes().iter().enumerate() {
        if *c == '(' as u8 {
            depth += 1;
        } else if *c == ')' as u8 {
            depth -= 1;
        } else if *c == ',' as u8 && depth == 0 {
            args.push(&text[last_index..i]);
            last_index = i + 1;
        }
    }
    args.push(&text[last_index..text.len()].trim().trim_end_matches(')'));
    args
}

pub fn parse_function_like(text: &str) -> (&str, Vec<&str>) {
    match text.split_once('(') {
        Some((func, args_string)) => {
            (func, split_list(args_string))
        }
        None => (text, vec![]),
    }
}
