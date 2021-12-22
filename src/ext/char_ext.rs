pub(crate) trait CharExt : Sized {
    fn as_char(self) -> char;
    fn is_ascii_word_character(self) -> bool { let ch = self.as_char(); ch.is_ascii_alphanumeric() || ch == '_' }
}

impl CharExt for char {
    fn as_char(self) -> char { self }
}

impl CharExt for u8 {
    fn as_char(self) -> char { self as _ }
}
