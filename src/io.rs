use std::fmt::Display;

pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) source: String,
}

impl File {
    pub(crate) fn eof_span(&self) -> Span<'_> {
        Span { file: self, start: self.source.len(), length: 0 }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Location<'file> {
    file: &'file File,
    index: usize,
}

#[derive(Copy, Clone)]
pub(crate) struct Span<'file> {
    file: &'file File,
    start: usize,
    length: usize,
}
impl Span<'_> {
    pub(crate) fn new(file: &File, start: usize, length: usize) -> Span<'_> {
        Span { file, start, length }
    }

    pub(crate) fn new_from_start_and_end(file: &File, start: usize, end: usize) -> Span<'_> {
        Span { file, start, length: end - start }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Located<'file, T>(pub(crate) Span<'file>, pub(crate) T);

impl Display for Location<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, col) = get_line_col(self.file, self.index);
        write!(f, "{}:{}:{}", self.file.name, line, col)
    }
}
impl Display for Span<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start_line, start_col) = get_line_col(self.file, self.start);
        let (end_line, end_col) = get_line_col(self.file, self.start + self.length);

        if start_line == end_line {
            if start_col == end_col {
                // zero length span
                write!(f, "{}:{}:{}", self.file.name, start_line, start_line)
            } else {
                // span contained entirely within one line
                write!(f, "{}:{}:{}-{}", self.file.name, start_line, start_col, end_col)
            }
        } else {
            // span that stretches across multiple lines
            write!(f, "{}:({}:{})-({}:{})", self.file.name, start_line, start_col, end_line, end_col)
        }
    }
}

fn get_line_col(file: &File, index: usize) -> (usize, usize) {
    let line = file.source[..index].chars().filter(|c| *c == '\n').count() + 1;
    let col = file.source[..index].chars().rev().take_while(|c| *c != '\n').count() + 1;
    (line, col)
}

#[cfg(test)]
mod test {
    #[test]
    fn line_col() {
        use crate::io::{get_line_col, File};

        let test_line_col = |file_contents: &'static str, index, expected_line, expected_col| {
            let file = File { name: "test_line_col generated file".to_string(), source: file_contents.to_string() };
            assert_eq!(get_line_col(&file, index), (expected_line, expected_col));
        };

        test_line_col("abc\n", 0, 1, 1);
        test_line_col("abc\n", 1, 1, 2);
        test_line_col("abc\n", 2, 1, 3);
        test_line_col("abc\n", 3, 1, 4);
        test_line_col("abc\nabcde", 4, 2, 1);
        test_line_col("abc\nabcde", 5, 2, 2);
    }
}
