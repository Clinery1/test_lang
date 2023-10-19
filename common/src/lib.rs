use std::{
    ops::{
        Range,
        RangeInclusive,
    },
    cmp::Ordering,
};


pub mod error;


/// An index range into the source code
pub type Span = Range<usize>;

/// A location range denoting the start and end location (inclusive-inclusive)
pub type LocationSpan = RangeInclusive<Location>;


/// Line and column are zero-based
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}
impl PartialOrd for Location {
    fn partial_cmp(&self, o: &Self)->Option<Ordering> {
        if self.line == o.line {
            return self.column.partial_cmp(&o.column);
        }
        return self.line.partial_cmp(&o.line);
    }
}

/// Allows converting between source index spans and location spans
pub struct SpanConverter {
    line_spans: Vec<Span>,
}
impl SpanConverter {
    pub fn new(source: &str)->Self {
        let mut line_spans = Vec::new();
        let mut prev_start = 0;

        for (i, c) in source.char_indices() {
            match c {
                '\n'=>{
                    line_spans.push(prev_start..(i + 1));
                    prev_start = i + 1;
                },
                _=>{},
            }
        }
        line_spans.push(prev_start..source.len());

        SpanConverter {
            line_spans,
        }
    }

    /// Converts a Span to a LocationSpan
    pub fn convert(&self, span: Span)->LocationSpan {
        let mut start = None;
        let mut end = None;
        for (i, line_span) in self.line_spans.iter().enumerate() {
            if line_span.contains(&span.start) {
                start = Some(Location {
                    line: i,
                    column: span.start - line_span.start,
                });
            }
            if line_span.contains(&span.end) {
                end = Some(Location {
                    line: i,
                    column: span.end - line_span.start,
                });

                break;
            }
        }

        let start = start.unwrap();
        let end = end.unwrap();

        return start..=end;
    }
}
