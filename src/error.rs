#![allow(dead_code)]


use string_interner::DefaultSymbol as Symbol;
use logos::Span;
use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    ops::RangeInclusive,
};
use crate::{
    ast::{
        BinaryOp,
        UnaryOp,
    },
    lexer::Token,
};


/// A simple error type enum. Will probably have to write a `Display` impl for it later, but
/// `Debug` is enough for now.
#[derive(Debug, Clone, PartialEq)]
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
    InvalidFunctionArgs(usize, usize),
    FunctionRedefined,
    TooManyParams,
    TooManyArgs,
    TypeHasNoFields,
    FieldExists,
    MethodRedefined,
    AssociatedMethodRedefined,
    UndefinedClass,
    ClassHasNoAssociated,
    ConstructorRedefined,
    ConstructorRequired,
}
impl ErrorType {
    pub fn err_num(&self)->u16 {
        use ErrorType::*;
        match self {
            ExpectedToken(..)=>0,
            ExpectedIdent=>1,
            UnclosedParen=>2,
            UnclosedCurly=>3,
            UnclosedSquare=>4,
            UnexpectedToken=>5,
            UnexpectedEOF=>6,
            LineEnding=>7,
            VarExistsInScope=>7,
            VarDoesNotExist=>8,
            VarUndefined=>9,
            CannotReassign=>10,
            CannotMutate=>11,
            BinaryOperationNotSupported(..)=>12,
            UnaryOperationNotSupported(..)=>13,
            NoField(..)=>14,
            CannotCall=>15,
            CannotIndex=>16,
            ArrayOutOfBounds=>17,
            InvalidIndexType=>18,
            InvalidType=>19,
            InvalidFunctionArgs(..)=>20,
            FunctionRedefined=>21,
            TooManyParams=>22,
            TooManyArgs=>23,
            TypeHasNoFields=>24,
            FieldExists=>25,
            MethodRedefined=>26,
            AssociatedMethodRedefined=>27,
            UndefinedClass=>28,
            ClassHasNoAssociated=>29,
            ConstructorRedefined=>30,
            ConstructorRequired=>31,
        }
    }
}
impl Display for ErrorType {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        use ErrorType::*;
        match self {
            ExpectedToken(token)=>write!(f,"Expected the token `{:?}`", token),
            ExpectedIdent=>write!(f,"Expected an identifier"),
            UnclosedParen=>write!(f,"Unclosed parenthesis"),
            UnclosedCurly=>write!(f,"Unclosed curly bracket"),
            UnclosedSquare=>write!(f,"Unclosed square bracket"),
            UnexpectedToken=>write!(f,"Unexpected token"),
            UnexpectedEOF=>write!(f,"Unexpected end of file"),
            LineEnding=>write!(f,"Expected a semicolon or newline"),
            VarExistsInScope=>write!(f,"Variable redefined here in this scope"),
            VarDoesNotExist=>write!(f,"Variable does not exist"),
            VarUndefined=>write!(f,"Variable is undefined"),
            CannotReassign=>write!(f,"Cannot reassign to this"),
            CannotMutate=>write!(f,"Cannot mutate this"),
            BinaryOperationNotSupported(op)=>write!(f,"Binary operation ({}) is not supported on these types", op),
            UnaryOperationNotSupported(op)=>write!(f,"Unary operation ({}) is not supported on this type", op),
            NoField(sym)=>write!(f,"There is no field named <{:?}> on this object", sym),
            CannotCall=>write!(f,"Cannot call this data type"),
            CannotIndex=>write!(f,"Cannot index this data type"),
            ArrayOutOfBounds=>write!(f,"Array index out of bounds"),
            InvalidIndexType=>write!(f,"Cannot index a value with this type"),
            InvalidType=>write!(f,"Invalid type"),
            InvalidFunctionArgs(expect, got)=>write!(f,"Invalid number of function arguments. Expected {}, but got {}", expect, got),
            FunctionRedefined=>write!(f,"Function redefined here"),
            TooManyParams=>write!(f,"Too many parameters for a function. The maximum is 255."),
            TooManyArgs=>write!(f,"Too many arguments for a function. The maximum is 255."),
            TypeHasNoFields=>write!(f,"Type has no fields"),
            FieldExists=>write!(f,"Field already exists"),
            MethodRedefined=>write!(f,"Class method redefined here"),
            AssociatedMethodRedefined=>write!(f,"Associated class function redefined here"),
            UndefinedClass=>write!(f,"Class is not defined"),
            ClassHasNoAssociated=>write!(f,"The class has no associated function"),
            ConstructorRedefined=>write!(f,"Class constructor redefined here"),
            ConstructorRequired=>write!(f,"A constructor is required for classes with fields"),
        }
    }
}

/// A simple error type that should handle my needs for the foreseeable future
#[derive(Debug, Clone)]
pub enum Error {
    Standard {
        err_type: ErrorType,
        span: Span,
    },
    TwoLocation {
        err_type: ErrorType,
        first_msg: &'static str,
        first: Span,
        second: Span,
    },
}
impl Error {
    #[inline]
    /// Create a new error
    pub fn new(span: Span, err_type: ErrorType)->Self {
        Error::Standard {
            err_type,
            span,
        }
    }

    pub fn two_location(first: Span, second: Span, first_msg: &'static str, err_type: ErrorType)->Self {
        Error::TwoLocation {
            err_type,
            first_msg,
            first,
            second,
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

    /// Get a reference to the error type
    pub fn err_type(&self)->&ErrorType {
        match self {
            Self::Standard{err_type,..}|
                Self::TwoLocation{err_type,..}=>err_type,
        }
    }

    fn print_source(source: &str, metrics: SourceMetrics, line_num_width: Option<usize>, err_msg: impl Display) {
        let line_delta = metrics.end.num - metrics.start.num;
        let start_offset = metrics.start.offset;
        let end_offset = metrics.end.offset;

        if line_delta == 0 {    // single line error
            // get the source code for the line
            let line = &source[metrics.start.range];

            // convert the line number to a string so we can measure its length
            let line_num = (metrics.start.num + 1).to_string();
            let number_width = line_num_width.unwrap_or(line_num.len()).max(3);

            // print a newline if the line doesn't have one
            if line.ends_with('\n') {
                eprint!("{:>number_width$} │ {}", line_num, line);
            } else {
                eprintln!("{:>number_width$} │ {}", line_num, line);
            }

            // find the difference between the start and end points. subtract one because it
            // otherwise looks weird
            let start_end_delta = (end_offset - start_offset).saturating_sub(1);

            if start_end_delta > 1 {
                // if the difference is more than 1 character, then line characters showing the start
                // and end
                eprintln!("{:>number_width$}   {:start_offset$}╰{:─>start_end_delta$}", " ", "", "╯");
            } else {
                // otherwise, just print a carat to show the error location
                eprintln!("{:>number_width$}   {:start_offset$}^", " ", "");
            }

            // print the error message on another line
            eprintln!("{:number_width$}   {:start_offset$} {}", " ", "", err_msg);
        } else {    // multi line error
            // get the length of the longest line number (the ending line number)
            let line_num = (metrics.end.num + 1).to_string();
            let line_num_max = line_num_width.unwrap_or(line_num.len()).max(3);

            // slice the source code lines
            let line0 = &source[metrics.start.range];
            let line1 = &source[metrics.end.range];

            // print the start line and line number
            eprint!("{:>line_num_max$} │ {}", metrics.start.num + 1,line0);

            // print where the error happens and the error message
            eprintln!("{:>line_num_max$} ├─{0:─>start_offset$}╯ {}", "", err_msg);

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
                eprint!("{:>line_num_max$} │ {}", metrics.end.num + 1, line1);
            } else {
                eprintln!("{:>line_num_max$} │ {}", metrics.end.num + 1, line1);
            }

            // print the line characters pointing to where the error ends
            eprintln!("{:>line_num_max$} ╰─{:─>end_offset$}", "", "╯");
        }
    }

    /// Print the error to STDERR
    pub fn print(&self, source: &str) {
        match self {
            Self::Standard{err_type,span}=>{
                // check to make sure this error fits within the source string (sanity check)
                if span.end > source.len() {
                    println!("Invalid source");
                    return;
                }

                let metrics = SourceMetrics::new(source, span.clone());

                println!("Error[E{}]:", err_type.err_num());
                Self::print_source(source, metrics, None, err_type);
            },
            Self::TwoLocation{err_type,first_msg,first,second}=>{
                if first.end > source.len() || second.end > source.len() {
                    println!("Invalid source");
                    return;
                }

                let first_metrics = SourceMetrics::new(source, first.clone());
                let second_metrics = SourceMetrics::new(source, second.clone());

                let first_width = (first_metrics.end.num + 1).to_string().len();
                let second_width = (second_metrics.end.num + 1).to_string().len();

                let width = first_width.max(second_width).max(3);

                println!("Error[E{}]:", err_type.err_num());
                Self::print_source(source, first_metrics, Some(width), first_msg);
                println!();
                Self::print_source(source, second_metrics, Some(width), err_type);
            },
        }
    }
}


#[derive(Default)]
struct SourceMetrics {
    pub start: Line,
    pub end: Line,
}
impl SourceMetrics {
    pub fn new(source: &str, span: Span)->Self {
        let start = span.start;
        let end = span.end;

        let mut metrics = SourceMetrics::default();

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
        for (i, line) in lines.iter().enumerate() {
            if line.contains(&start) {
                metrics.start = Line {
                    range: line.clone(),
                    num: i,
                    offset: start - line.start(),
                };
            }
            if line.contains(&(end - 1)) {
                metrics.end = Line {
                    range: line.clone(),
                    num: i,
                    offset: end - line.start(),
                };
                break;
            }
        }

        return metrics;
    }
}

struct Line {
    pub range: RangeInclusive<usize>,
    pub num: usize,
    pub offset: usize,
}
impl Default for Line {
    fn default()->Self {
        Line {
            range: 0..=0,
            num: 0,
            offset: 0,
        }
    }
}
