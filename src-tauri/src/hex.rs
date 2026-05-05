pub fn format_hex(data: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        result.push_str(&format!("{:04X}  ", i * 16));

        for (j, byte) in chunk.iter().enumerate() {
            result.push_str(&format!("{:02X} ", byte));
            if j == 7 {
                result.push(' ');
            }
        }

        if chunk.len() < 16 {
            for _ in 0..(16 - chunk.len()) {
                result.push_str("   ");
            }
            if chunk.len() <= 8 {
                result.push(' ');
            }
        }

        result.push_str("  ");

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
                chars.next();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_hex_handles_short_buffer() {
        let s = format_hex(b"AB");
        assert!(s.starts_with("0000  41 42 "), "got: {s:?}");
        assert!(s.trim_end().ends_with("AB"));
    }

    #[test]
    fn format_hex_aligns_two_full_rows() {
        let data: Vec<u8> = (0..32).collect();
        let s = format_hex(&data);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("0000  "));
        assert!(lines[1].starts_with("0010  "));
    }

    #[test]
    fn strip_ansi_removes_color_codes() {
        let input = "\x1b[31mred\x1b[0m and \x1b[1;32mbold green\x1b[0m";
        assert_eq!(strip_ansi_codes(input), "red and bold green");
    }

    #[test]
    fn strip_ansi_passes_through_plain_text() {
        assert_eq!(strip_ansi_codes("hello"), "hello");
    }

    #[test]
    fn parse_hex_input_accepts_spaced_bytes() {
        assert_eq!(parse_hex_input("AA BB 0D 0A").unwrap(), vec![0xAA, 0xBB, 0x0D, 0x0A]);
    }

    #[test]
    fn parse_hex_input_rejects_garbage() {
        assert!(parse_hex_input("AA ZZ").is_err());
    }

    #[test]
    fn parse_hex_input_rejects_empty() {
        assert!(parse_hex_input("   ").is_err());
    }
}
