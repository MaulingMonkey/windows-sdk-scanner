use crate::*;



pub(crate) fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();

    if let Some(ch) = chars.next() {
        if ch.is_ascii_alphanumeric() { // reject "_foo"
            chars.all(|ch| ch.is_ascii_word_character())
        } else {
            false
        }
    } else {
        false
    }
}
