use logos::Span;
use crate::lexer::Token;


#[derive(Debug)]
pub enum ErrorType {
    ExpectedToken(Token),
    ExpectedIdent,
    UnclosedParen,
    UnexpectedToken,
    UnexpectedEOF,
}


#[derive(Debug)]
pub struct Error {
    pub err_type: ErrorType,
    start: usize,
    end: usize,
}
impl Error {
    #[inline]
    pub fn new(span: Span, err_type: ErrorType)->Self {
        Error {
            err_type,
            start: span.start,
            end: span.end,
        }
    }

    #[inline]
    pub fn eof(span: Span)->Self {
        Self::new(span, ErrorType::UnexpectedEOF)
    }

    #[inline]
    pub fn token(span: Span)->Self {
        Self::new(span, ErrorType::UnexpectedToken)
    }

    #[inline]
    pub fn ident(span: Span)->Self {
        Self::new(span, ErrorType::ExpectedIdent)
    }

    pub fn print(&self, source: &str) {
        if self.end > source.len() {
            return;
        }
        let mut lines = Vec::new();
        let mut line_start = 0;
        for (i, c) in source.char_indices() {
            if c=='\n' {
                lines.push(line_start..=i);
                line_start = i + 1;
            }
        }
        lines.push(line_start..=source.len());

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

        debug_assert_ne!(start_line, usize::MAX);
        debug_assert_ne!(end_line, usize::MAX);

        let line_delta = end_line - start_line;
        let start_offset = self.start - lines[start_line].start();
        let end_offset = self.end - lines[end_line].start();
        // dbg!(
        //     start_offset,
        //     end_offset,
        //     start_line,
        //     end_line,
        //     self.start,
        //     self.end,
        //     source.len(),
        //     line_delta,
        //     &lines,
        //     &source[lines[0].clone()],
        // );

        match line_delta {
            0=>{    // error is on one line
                let line = &source[lines[start_line].clone()];
                let line_num = (start_line + 1).to_string();
                let number_width = line_num.len();

                if line.ends_with('\n') {
                    print!("{} │ {}", line_num, line);
                } else {
                    println!("{} │ {}", line_num, line);
                }

                let start_end_delta = end_offset - start_offset;
                if start_end_delta > 1 {
                    println!("{:number_width$}   {:start_offset$}^{:->start_end_delta$}", " ", "", "^");
                } else {
                    println!("{:number_width$}   {:start_offset$}^", " ", "");
                }
                println!("{:?}", self.err_type);
            },
            // 1=>{    // error is across two lines
            //     let start_line_extra_width = (lines[start_line].end() - self.start).saturating_sub(1);
            //     let line_num_max = end_line.to_string().len();
            //     println!("{:line_num_max$} ╭─{0:─>start_offset$}╮{0:>start_line_extra_width$}", "");
            //     let line0 = &source[lines[start_line].clone()];
            //     let line1 = &source[lines[end_line].clone()];
            //     print!("{:<line_num_max$} │ {}", start_line,line0);
            //     if line1.ends_with('\n') {
            //         print!("{} │ {}", end_line, line1);
            //     } else {
            //         println!("{} │ {}", end_line, line1);
            //     }
            //     println!("{:line_num_max$} ╰─{:─>end_offset$}", "", "╯");
            //     println!("{:?}", self.err_type);
            // },
            _=>{    // error is across more than two lines
                let line_num_max = end_line.to_string().len().max(3);
                let line0 = &source[lines[start_line].clone()];
                let line1 = &source[lines[end_line].clone()];
                print!("{:>line_num_max$} │ {}", start_line,line0);
                println!("{:>line_num_max$} ├─{0:─>start_offset$}╯ {:?}", "", self.err_type);
                if line_delta > 1 {
                    println!("...");
                } else {
                    println!("{:>line_num_max$} │", "");
                }
                if line1.ends_with('\n') {
                    print!("{:>line_num_max$} │ {}", end_line, line1);
                } else {
                    println!("{:>line_num_max$} │ {}", end_line, line1);
                }
                println!("{:>line_num_max$} ╰─{:─>end_offset$}", "", "╯");
            },
        }
    }
}
