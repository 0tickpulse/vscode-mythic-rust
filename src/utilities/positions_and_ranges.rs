use std::fmt::{Debug, Display};

use marked_yaml::{Marker, Span};
use tower_lsp::lsp_types::{Position, Range};

#[derive(PartialEq, Eq, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct CustomPosition {
    pub line: u32,
    pub character: u32,
}

impl Display for CustomPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Position({}, {})", self.line, self.character)
    }
}

impl CustomPosition {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
    pub fn from_marker(marker: &Marker) -> Self {
        Self {
            line: marker.line() as u32,
            character: marker.column() as u32,
        }
    }
    /// Creates a new position from a zero-based offset given a source string.
    pub fn from_offset(mut offset: u32, source: &str) -> Self {
        if offset == 0 {
            return Self::new(0, 0);
        }

        let lines = source.lines().collect::<Vec<&str>>();
        let mut line = 0;
        let mut character = 0;
        for (i, current_line) in lines.iter().enumerate() {
            if offset < current_line.len() as u32 {
                line = i as u32;
                character = offset;
                break;
            }
            offset -= current_line.len() as u32 + 1;
            line = i as u32 + 1;
            character = offset;
        }
        Self { line, character }
    }
    /// Converts a position to a zero-based offset given a source string.
    /// This is the inverse of [`Self::from_offset`]
    pub fn to_offset(&self, source: &str) -> u32 {
        let lines = source.lines().collect::<Vec<&str>>();
        let mut offset = 0;
        for (i, current_line) in lines.iter().enumerate() {
            if i as u32 == self.line {
                offset += self.character;
                break;
            }
            offset += current_line.len() as u32 + 1;
        }
        offset
    }
    pub fn set_line(&mut self, line: u32) -> &Self {
        self.line = line;
        self
    }
    pub fn add_line(&mut self, line: u32) -> &Self {
        self.line += line;
        self
    }
    pub fn subtract_line(&mut self, line: u32) -> &Self {
        self.line -= line;
        self
    }
    pub fn set_character(&mut self, character: u32) -> &Self {
        self.character = character;
        self
    }
    pub fn add_character(&mut self, character: u32) -> &Self {
        self.character += character;
        self
    }
    pub fn subtract_character(&mut self, character: u32) -> &Self {
        self.character -= character;
        self
    }
    pub fn to_position(&self) -> Position {
        Position {
            line: self.line,
            character: self.character,
        }
    }
    pub fn add(&self, other: &Self) -> Self {
        Self {
            line: self.line + other.line,
            character: self.character + other.character,
        }
    }
    pub fn add_offset(&self, offset: u32, source: &str) -> Self {
        Self::from_offset(self.to_offset(source) + offset, source)
    }
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        if self.line < other.line {
            std::cmp::Ordering::Less
        } else if self.line > other.line {
            std::cmp::Ordering::Greater
        } else if self.character < other.character {
            std::cmp::Ordering::Less
        } else if self.character > other.character {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
    pub fn create_range_with_offset(&self, offset: u32, source: &str) -> CustomRange {
        CustomRange::new(self.clone(), self.add_offset(offset, source))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct CustomRange {
    pub start: CustomPosition,
    pub end: CustomPosition,
}

impl Display for CustomRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Range({}, {})", self.start, self.end)
    }
}

impl CustomRange {
    pub fn new(start: CustomPosition, end: CustomPosition) -> Self {
        Self { start, end }
    }
    pub fn length(&self, source: &str) -> u32 {
        // if start is more than end, return 1
        if self.start.compare(&self.end) == std::cmp::Ordering::Greater {
            return 1;
        }
        self.end.to_offset(source) - self.start.to_offset(source)
    }
    pub fn from_range(range: &core::ops::Range<usize>, source: &str) -> Self {
        Self {
            start: CustomPosition::from_offset(range.start as u32, source),
            end: CustomPosition::from_offset(range.end as u32, source),
        }
    }
    pub fn from_span(span: Span) -> Self {
        Self {
            // TODO: Fix this unwrap
            start: CustomPosition::from_marker(
                &span
                    .start()
                    .map(|marker| {
                        Marker::new(marker.source(), marker.line() - 1, marker.column() - 1)
                    })
                    .unwrap_or(Marker::new(0, 0, 1)),
            ),
            end: CustomPosition::from_marker(
                &span
                    .end()
                    .map(|marker| {
                        Marker::new(marker.source(), marker.line() - 1, marker.column() - 1)
                    })
                    .unwrap_or(Marker::new(0, 0, 0)),
            ),
        }
    }
    pub fn get_from(&self, source: &str) -> String {
        let start = self.start.to_offset(source);
        let end = self.end.to_offset(source);
        source[start as usize..end as usize].to_string()
    }
    pub fn set_start(&mut self, start: CustomPosition) -> &Self {
        self.start = start;
        self
    }
    pub fn set_end(&mut self, end: CustomPosition) -> &Self {
        self.end = end;
        self
    }
    pub fn add(&self, other: &Self) -> Self {
        Self {
            start: self.start.add(&other.start),
            end: self.end.add(&other.end),
        }
    }
    pub fn add_offset_to_start(&self, offset: u32, source: &str) -> Self {
        Self {
            start: self.start.add_offset(offset, source),
            end: self.end,
        }
    }
    pub fn add_offset_to_end(&self, offset: u32, source: &str) -> Self {
        Self {
            start: self.start,
            end: self.end.add_offset(offset, source),
        }
    }
    pub fn add_offset(&self, offset: u32, source: &str) -> Self {
        Self {
            start: self.start.add_offset(offset, source),
            end: self.end.add_offset(offset, source),
        }
    }
    pub fn contains(&self, position: &CustomPosition) -> bool {
        self.start.compare(position) == std::cmp::Ordering::Less
            && self.end.compare(position) == std::cmp::Ordering::Greater
    }
    pub fn to_range(&self) -> Range {
        Range {
            start: self.start.to_position(),
            end: self.end.to_position(),
        }
    }
}
