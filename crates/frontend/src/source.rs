use std::ops::Range;

pub type Span = Range<usize>;
pub type Spanned<T> = (T, Span);

#[must_use]
pub fn empty_span(offset: usize) -> Span {
    offset..offset
}

#[must_use]
pub fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut line_start = 0;

    for (index, character) in source.char_indices() {
        if index >= offset {
            break;
        }

        if character == '\n' {
            line += 1;
            line_start = index + character.len_utf8();
        }
    }

    let column = source.get(line_start..offset).map_or(1, |line| line.chars().count() + 1);
    (line, column)
}
