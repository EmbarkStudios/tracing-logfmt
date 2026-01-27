use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SerializerError {
    InvalidKey,
    FmtError,
}

impl From<std::fmt::Error> for SerializerError {
    fn from(_: std::fmt::Error) -> Self {
        SerializerError::FmtError
    }
}

/// Serializes key/value pairs into logfmt format.
pub(crate) struct Serializer<W> {
    pub(crate) writer: W,
    writing_first_entry: bool,
    #[cfg(feature = "ansi_logs")]
    with_ansi_color: bool,
}

impl<W> Serializer<W>
where
    W: fmt::Write,
{
    #[inline]
    pub(crate) fn new(writer: W, #[cfg(feature = "ansi_logs")] with_ansi_color: bool) -> Self {
        Serializer {
            writer,
            writing_first_entry: true,
            #[cfg(feature = "ansi_logs")]
            with_ansi_color,
        }
    }

    #[cfg(not(feature = "ansi_logs"))]
    pub(crate) fn serialize_entry(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), SerializerError> {
        self.serialize_entry_with(key, value, |this, value| this.serialize_value(value))
    }

    #[cfg(feature = "ansi_logs")]
    pub(crate) fn serialize_entry(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), SerializerError> {
        if let "level" = key {
            self.serialize_entry_with(key, value, |this, value| this.serialize_level(value))
        } else {
            self.serialize_entry_with(key, value, |this, value| this.serialize_value(value))
        }
    }

    pub(crate) fn serialize_entry_no_quote(
        &mut self,
        key: &str,
        value: impl fmt::Debug,
    ) -> Result<(), SerializerError> {
        self.serialize_entry_with(key, value, |this, value| {
            this.serialize_value_no_quote(value)
        })
    }

    fn serialize_entry_with<F, T>(
        &mut self,
        key: &str,
        value: T,
        serialize_value: F,
    ) -> Result<(), SerializerError>
    where
        F: FnOnce(&mut Self, T) -> Result<(), SerializerError>,
    {
        self.serialize_key(key)?;
        self.writer.write_char('=')?;
        serialize_value(self, value)?;

        Ok(())
    }
    pub(crate) fn serialize_key(&mut self, key: &str) -> Result<(), SerializerError> {
        if !self.writing_first_entry {
            self.writer.write_char(' ')?;
        }
        self.writing_first_entry = false;

        let mut chars = key.chars().filter(|&ch| !need_quote(ch)).peekable();

        if chars.peek().is_none() {
            return Err(SerializerError::InvalidKey);
        }

        #[cfg(not(feature = "ansi_logs"))]
        {
            for c in chars {
                self.writer.write_char(c)?;
            }
        }

        #[cfg(feature = "ansi_logs")]
        {
            if self.with_ansi_color {
                let mut s = String::new();
                for c in chars {
                    s.push(c);
                }
                self.writer.write_str(
                    &nu_ansi_term::Color::Rgb(109, 139, 140)
                        .bold()
                        .paint(s)
                        .to_string(),
                )?;
            } else {
                for c in chars {
                    self.writer.write_char(c)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn serialize_value(&mut self, value: &str) -> Result<(), SerializerError> {
        if value.chars().any(need_quote) {
            self.writer.write_char('"')?;
            write!(self.writer, "{}", value.escape_debug())?;
            self.writer.write_char('"')?;
        } else {
            self.writer.write_str(value)?;
        }

        Ok(())
    }

    fn serialize_value_no_quote(&mut self, value: impl fmt::Debug) -> Result<(), SerializerError> {
        write!(self.writer, "{:?}", value)?;
        Ok(())
    }

    #[cfg(feature = "ansi_logs")]
    fn serialize_level(&mut self, value: &str) -> Result<(), SerializerError> {
        write!(self.writer, "{}", value)?;
        Ok(())
    }
}

#[inline]
pub(crate) fn need_quote(ch: char) -> bool {
    ch <= ' ' || matches!(ch, '=' | '"')
}

impl<W> std::io::Write for Serializer<W>
where
    W: fmt::Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(buf) = std::str::from_utf8(buf) {
            self.writer.write_str(buf).map_err(std::io::Error::other)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "ansi_logs"))]
    fn test_serialize_entries() {
        let mut output = String::new();
        let mut s = Serializer::new(&mut output);
        assert!(s.serialize_entry("key", "value").is_ok());
        assert!(s.serialize_entry("key2", "value2").is_ok());

        assert_eq!(output, "key=value key2=value2");
    }

    #[test]
    #[cfg(not(feature = "ansi_logs"))]
    fn test_serialize_entry() {
        let tests = vec![
            (("key", "value"), "key=value"),
            (("ke=y", "value="), "key=\"value=\""),
            (("key ", "value "), "key=\"value \""),
            (("ke\"y", "valu\"e"), "key=\"valu\\\"e\""),
            (("ke\ny", "valu\ne"), "key=\"valu\\ne\""),
        ];

        for ((k, v), expected_output) in tests {
            let mut output = String::new();
            let mut s = Serializer::new(&mut output);
            assert!(s.serialize_entry(k, v).is_ok());
            assert_eq!(output, expected_output,);
        }
    }

    #[test]
    #[cfg(feature = "ansi_logs")]
    fn test_serialize_entry() {
        let tests = vec![
            (("key", "value"), make_ansi_key_value("key", "=value")),
            (
                ("ke=y", "value="),
                make_ansi_key_value("key", "=\"value=\""),
            ),
            (
                ("key ", "value "),
                make_ansi_key_value("key", "=\"value \""),
            ),
            (("lev\"el", "info"), make_ansi_key_value("level", "=info")),
            (
                ("ke\ny", "valu\ne"),
                make_ansi_key_value("key", "=\"valu\\ne\""),
            ),
        ];

        for ((k, v), expected_output) in tests {
            let mut output = String::new();
            let mut s = Serializer::new(&mut output, true);
            assert!(s.serialize_entry(k, v).is_ok());
            assert_eq!(output, expected_output,);
        }

        fn make_ansi_key_value(key: &str, value: &str) -> String {
            let mut key = nu_ansi_term::Color::Rgb(109, 139, 140)
                .bold()
                .paint(key)
                .to_string();
            key.push_str(value);
            key
        }
    }

    #[test]
    #[cfg(not(feature = "ansi_logs"))]
    fn test_serialize_key() {
        let tests = vec![
            ("key", "key"),
            ("k ey", "key"),
            ("k\"ey", "key"),
            ("k=ey", "key"),
            ("k\ney", "key"),
        ];
        for (input, expected_output) in tests {
            let mut output = String::new();

            let mut s = Serializer::new(&mut output);
            assert!(s.serialize_key(input).is_ok());

            assert_eq!(output, expected_output);
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
            let mut output = String::new();

            #[cfg(not(feature = "ansi_logs"))]
            let mut s = Serializer::new(&mut output);
            #[cfg(feature = "ansi_logs")]
            let mut s = Serializer::new(&mut output, true);
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
            let mut output = String::new();

            #[cfg(not(feature = "ansi_logs"))]
            let mut s = Serializer::new(&mut output);
            #[cfg(feature = "ansi_logs")]
            let mut s = Serializer::new(&mut output, true);

            assert!(s.serialize_value(input).is_ok());

            assert_eq!(output, expected_output);
        }
    }
}
