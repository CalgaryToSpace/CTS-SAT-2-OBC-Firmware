pub fn extract_function_and_args(input: &str) -> (&str, &str) {
    //
    let command_name = input.trim().split('(').next().unwrap_or("");
    let command_args_str = input
        .trim()
        .strip_prefix(command_name)
        .and_then(|s| s.strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("")
        .trim();

    (command_name, command_args_str)
}

pub fn has_valid_command_structure(input: &str) -> bool {
    let Some(open) = input.find('(') else {
        return false;
    };

    // A command must have a name and end with ')'.
    if open == 0 || !input.ends_with(')') {
        return false;
    }

    let mut depth = 0usize;

    for (index, byte) in input.bytes().enumerate().skip(open) {
        match byte {
            b'(' => depth += 1,

            b')' => {
                if depth == 0 {
                    return false;
                }

                depth -= 1;

                // The outer closing parenthesis must be last.
                if depth == 0 && index != input.len() - 1 {
                    return false;
                }
            }

            _ => {}
        }
    }

    depth == 0
}
