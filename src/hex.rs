pub fn format_hex(data: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        result.push_str(&format!("{:04X}  ", i * 16));

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            result.push_str(&format!("{:02X} ", byte));
            if j == 7 {
                result.push(' ');
            }
        }

        // Padding for incomplete lines
        if chunk.len() < 16 {
            for _ in 0..(16 - chunk.len()) {
                result.push_str("   ");
            }
            // Add the column-8 separator space if the line didn't reach it
            if chunk.len() <= 8 {
                result.push(' ');
            }
        }

        result.push_str("  ");

        // ASCII representation
        for byte in chunk {
            let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            result.push(ch);
        }

        result.push('\n');
    }
    result
}

pub fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Skip everything until we find a letter (command character)
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Parse space-separated hex bytes (e.g. "AA BB 0D 0A") into raw bytes.
/// Returns Err with a message describing the first invalid token.
pub fn parse_hex_input(input: &str) -> Result<Vec<u8>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Empty input".to_string());
    }

    let mut bytes = Vec::new();
    for token in trimmed.split_whitespace() {
        match u8::from_str_radix(token, 16) {
            Ok(b) => bytes.push(b),
            Err(_) => return Err(format!("Invalid hex byte: '{}'", token)),
        }
    }
    Ok(bytes)
}
