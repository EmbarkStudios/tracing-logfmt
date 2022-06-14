#[derive(Debug, PartialEq)]
pub enum SerializerError {
    InvalidKey,
}

/// Serializes key/value pairs into logfmt format.
pub struct Serializer {
    pub output: String,
}

impl Serializer {
    pub fn new() -> Self {
        Serializer {
            output: String::new(),
        }
    }

    pub fn serialize_entry(&mut self, key: &str, value: &str) -> Result<(), SerializerError> {
        if !self.output.is_empty() {
            self.output += " ";
        }

        self.serialize_key(key)?;
        self.output += "=";
        self.serialize_value(value)?;

        Ok(())
    }

    fn serialize_key(&mut self, key: &str) -> Result<(), SerializerError> {
        let key: &str = &key
            .chars()
            .filter(|&ch| !Self::need_quote(ch))
            .collect::<String>();

        if key.is_empty() {
            return Err(SerializerError::InvalidKey);
        }

        self.output += key;
        Ok(())
    }

    fn serialize_value(&mut self, value: &str) -> Result<(), SerializerError> {
        if value.chars().any(Self::need_quote) {
            self.output += &format!(r#""{}""#, value.escape_debug());
        } else {
            self.output += value;
        }

        Ok(())
    }

    fn need_quote(ch: char) -> bool {
        ch <= ' ' || matches!(ch, '=' | '"')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_entries() {
        let mut s = Serializer::new();
        assert!(s.serialize_entry("key", "value").is_ok());
        assert!(s.serialize_entry("key2", "value2").is_ok());

        assert_eq!(s.output, "key=value key2=value2");
    }

    #[test]
    fn test_serialize_entry() {
        let tests = vec![
            (("key", "value"), "key=value"),
            (("ke=y", "value="), "key=\"value=\""),
            (("key ", "value "), "key=\"value \""),
            (("ke\"y", "valu\"e"), "key=\"valu\\\"e\""),
            (("ke\ny", "valu\ne"), "key=\"valu\\ne\""),
        ];

        for ((k, v), expected_output) in tests {
            let mut s = Serializer::new();
            assert!(s.serialize_entry(k, v).is_ok());
            assert_eq!(s.output, expected_output,);
        }
    }

    #[test]
    fn test_serialize_key() {
        let tests = vec![
            ("key", "key"),
            ("k ey", "key"),
            ("k\"ey", "key"),
            ("k=ey", "key"),
            ("k\ney", "key"),
        ];
        for (input, expected_output) in tests {
            let mut s = Serializer::new();
            assert!(s.serialize_key(input).is_ok());

            assert_eq!(s.output, expected_output);
        }
    }

    #[test]
    fn test_serialize_key_invalid() {
        let tests = vec![
            ("", SerializerError::InvalidKey),
            (" ", SerializerError::InvalidKey),
            ("=", SerializerError::InvalidKey),
            ("\"", SerializerError::InvalidKey),
        ];

        for (input, expected_error) in tests {
            let mut s = Serializer::new();
            assert_eq!(s.serialize_key(input), Err(expected_error));
        }
    }

    #[test]
    fn test_serialize_value() {
        let tests = vec![
            ("", ""),
            ("v", "v"),
            (" ", r#"" ""#),
            ("=", r#""=""#),
            (r#"\"#, r#"\"#),
            (r#"""#, r#""\"""#),
            (r#"\""#, r#""\\\"""#),
            ("\n", r#""\n""#),
            ("\x00", r#""\0""#),
            ("\x10", r#""\u{10}""#),
            ("\x1F", r#""\u{1f}""#),
            ("µ", r#"µ"#),
            ("åäö", r#"åäö"#),
        ];

        for (input, expected_output) in tests {
            let mut s = Serializer::new();
            assert!(s.serialize_value(input).is_ok());

            assert_eq!(s.output, expected_output);
        }
    }
}
