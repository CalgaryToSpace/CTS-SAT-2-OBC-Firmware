use crate::{EXECUTE_IMMEDIATELY, UnixTimestampMs, error::ParsedTelecommandErr};

pub struct ParsedTelecommand<'a> {
    pub command_name: &'a str,
    pub command_args_str: &'a str,
    pub tsexec_parsed: UnixTimestampMs,
}

// TODO: Make a more robust parser instead of simple string manipulation
pub fn extract_function_args_tags<'a>(
    input: &'a str,
) -> Result<ParsedTelecommand<'a>, ParsedTelecommandErr> {
    let command_name = input.trim().split('(').next().unwrap_or("");
    if command_name.is_empty() {
        return Err(ParsedTelecommandErr::EmptyTelecommandString);
    }

    // Parse arguments
    // Get everything in between the first '(' and the last ')'
    let command_args_str = match (input.find('('), input.rfind(')')) {
        (Some(start), Some(end)) if start < end => &input[start + 1..end],
        _ => {
            return Err(ParsedTelecommandErr::UnbalancedParentheses);
        }
    };

    // Parse tags
    let mut tsexec_parsed: UnixTimestampMs = EXECUTE_IMMEDIATELY;
    for (i, _) in input.match_indices('@') {
        let left = &input[i + 1..];
        if let Some(end) = left.find('=') {
            let tag_name = &left[..end];
            let after_equal = &left[end + 1..];
            if tag_name == "tsexec" {
                let end_idx = after_equal
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(after_equal.len());
                let tsexec_str = &after_equal[..end_idx];
                tsexec_parsed = tsexec_str
                    .parse::<UnixTimestampMs>()
                    .map_err(|_| ParsedTelecommandErr::ParseStrValueError)?;
            }
        }
    }

    Ok(ParsedTelecommand {
        command_name,
        command_args_str,
        tsexec_parsed,
    })
}
