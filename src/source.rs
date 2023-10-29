use std::{fmt::Display, ops::Add};

pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) lines: Vec<String>,
    _dont_construct: (),
}

impl File {
    pub(crate) fn new(name: String, source: String) -> Self {
        let lines = source.lines().map(String::from).collect();
        Self { name, source, lines, _dont_construct: () }
    }

    pub(crate) fn eof_span(&self) -> Span<'_> {
        Span { file: self, start: self.source.len(), end: self.source.len() }
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
    end: usize,
}
impl Span<'_> {
    pub(crate) fn new_from_start_and_end(file: &File, start: usize, end: usize) -> Span<'_> {
        assert!(start <= end, "cannot have span that ends earlier than it starts");
        Span { file, start, end }
    }

    pub(crate) fn file(&self) -> &File {
        self.file
    }

    pub(crate) fn start(&self) -> usize {
        self.start
    }

    pub(crate) fn end(&self) -> usize {
        self.end
    }
}

impl<'a> Add for Span<'a> {
    type Output = Span<'a>;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(std::ptr::eq(self.file, rhs.file), "cannot join two spans from different files");
        Span { file: self.file, start: std::cmp::min(self.start, rhs.start), end: std::cmp::max(self.end, rhs.end) }
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
        let (end_line, end_col) = get_line_col(self.file, self.end);

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
        use crate::source::{get_line_col, File};

        let test_line_col = |file_contents: &'static str, index, expected_line, expected_col| {
            let file = File::new("test_line_col generated file".to_string(), file_contents.to_string());
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
