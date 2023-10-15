use string_interner::DefaultSymbol as Symbol;
use logos::Span;
use crate::{
    ast::{
        BinaryOp,
        UnaryOp,
    },
    lexer::Token,
};


#[derive(Debug)]
/// A simple error type enum. Will probably have to write a `Display` impl for it later, but
/// `Debug` is enough for now.
pub enum ErrorType {
    ExpectedToken(Token),
    ExpectedIdent,
    UnclosedParen,
    UnclosedCurly,
    UnclosedSquare,
    UnexpectedToken,
    UnexpectedEOF,
    LineEnding,
    VarExistsInScope,
    VarDoesNotExist,
    VarUndefined,
    CannotReassign,
    CannotMutate,
    BinaryOperationNotSupported(BinaryOp),
    UnaryOperationNotSupported(UnaryOp),
    NoField(Symbol),
    CannotCall,
    CannotIndex,
    ArrayOutOfBounds,
    InvalidIndexType,
    InvalidType,
    InvalidFunctionArgs,
    FunctionExists,
    TooManyParams,
    TooManyArgs,
    TypeHasNoFields,
}


#[derive(Debug)]
/// A simple error type that should handle my needs for the foreseeable future
pub struct Error {
    pub err_type: ErrorType,
    start: usize,
    end: usize,
}
impl Error {
    #[inline]
    /// Create a new error
    pub fn new(span: Span, err_type: ErrorType)->Self {
        Error {
            err_type,
            start: span.start,
            end: span.end,
        }
    }

    #[inline]
    /// Create a new `UnexpectedEOF` error
    pub fn eof(span: Span)->Self {
        Self::new(span, ErrorType::UnexpectedEOF)
    }

    #[inline]
    /// Create a new `BinaryOperationNotSupported` error
    pub fn binary(span: Span, op: BinaryOp)->Self {
        Self::new(span, ErrorType::BinaryOperationNotSupported(op))
    }

    #[inline]
    /// Create a new `UnaryOperationNotSupported` error
    pub fn unary(span: Span, op: UnaryOp)->Self {
        Self::new(span, ErrorType::UnaryOperationNotSupported(op))
    }

    #[inline]
    /// Create a new `UnexpectedToken` error
    pub fn token(span: Span)->Self {
        Self::new(span, ErrorType::UnexpectedToken)
    }

    #[inline]
    /// Create a new `ExpectedIdent` error
    pub fn ident(span: Span)->Self {
        Self::new(span, ErrorType::ExpectedIdent)
    }

    /// Print the error to STDERR
    pub fn print(&self, source: &str) {
        // check to make sure this error fits within the source string (sanity check)
        if self.end > source.len() {
            return;
        }

        // create a list of inclusive ranges for each line
        let mut lines = Vec::new();
        let mut line_start = 0;
        for (i, c) in source.char_indices() {
            if c=='\n' {
                lines.push(line_start..=i);
                line_start = i + 1;
            }
        }
        // add the last line
        lines.push(line_start..=source.len());

        // find which line start and end are contained in
        let mut start_line = usize::MAX;
        let mut end_line = usize::MAX;
        for (i, line) in lines.iter().enumerate() {
            if line.contains(&self.start) {
                start_line = i;
            }
            if line.contains(&(self.end - 1)) {
                end_line = i;
                break;
            }
        }

        // assert that start and end have a line number (assumption: there are fewer than
        // usize::MAX lines)
        debug_assert_ne!(start_line, usize::MAX);
        debug_assert_ne!(end_line, usize::MAX);

        // find how many lines this error encompasses and the offsets from the start of the line
        // for self.{start,end}
        let line_delta = end_line - start_line;
        let start_offset = self.start - lines[start_line].start();
        let end_offset = self.end - lines[end_line].start();

        if line_delta == 0 {    // single line error
            // get the source code for the line
            let line = &source[lines[start_line].clone()];

            // convert the line number to a string so we can measure its length
            let line_num = (start_line + 1).to_string();
            let number_width = line_num.len();

            // print a newline if the line doesn't have one
            if line.ends_with('\n') {
                eprint!("{} │ {}", line_num, line);
            } else {
                eprintln!("{} │ {}", line_num, line);
            }

            // find the difference between the start and end points. subtract one because it
            // otherwise looks weird
            let start_end_delta = (self.end - self.start).saturating_sub(1);

            if start_end_delta > 1 {
                // if the difference is more than 1 character, then line characters showing the start
                // and end
                eprintln!("{:number_width$}   {:start_offset$}╰{:─>start_end_delta$}", " ", "", "╯");
            } else {
                // otherwise, just print a carat to show the error location
                eprintln!("{:number_width$}   {:start_offset$}^", " ", "");
            }

            // print the error message on another line
            eprintln!("{:number_width$}   {:start_offset$} {:?}", " ", "", self.err_type);
        } else {    // multi line error
            // get the length of the longest line number (the ending line number)
            let line_num_max = end_line.to_string().len().max(3);

            // slice the source code lines
            let line0 = &source[lines[start_line].clone()];
            let line1 = &source[lines[end_line].clone()];

            // print the start line and line number
            eprint!("{:>line_num_max$} │ {}", start_line,line0);

            // print where the error happens and the error message
            eprintln!("{:>line_num_max$} ├─{0:─>start_offset$}╯ {:?}", "", self.err_type);

            if line_delta > 1 {
                // if there are more than 2 lines, then print a `...` showing there are hidden
                // lines
                eprintln!("...");
            } else {
                // otherwise just print a blank line with no number for spacing
                eprintln!("{:>line_num_max$} │", "");
            }

            // print the second line and a newline if it doesn't have one
            if line1.ends_with('\n') {
                eprint!("{:>line_num_max$} │ {}", end_line, line1);
            } else {
                eprintln!("{:>line_num_max$} │ {}", end_line, line1);
            }

            // print the line characters pointing to where the error ends
            eprintln!("{:>line_num_max$} ╰─{:─>end_offset$}", "", "╯");
        }
    }
}
