use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};
use tree_sitter_rust;
use std::collections::HashMap;

pub struct HighlightSpan {
    pub highlight_name: &'static str,
    pub start_line: usize,
    pub start_col: usize
}

struct ByteToLineCol {
    line_starts: Vec<usize>,
}

impl ByteToLineCol {
    pub fn new(source: &[u8]) -> Self {
        let mut line_starts = vec![0];
        
        for (i, &byte) in source.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(i + 1);
            }
        }

        Self { line_starts }
    }

    pub fn byte_to_line_col(&self, byte_pos: usize, source: &[u8]) -> Result<(usize, usize), std::str::Utf8Error> {
        let line = match self.line_starts.binary_search(&byte_pos) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1)
        };

        let line_start = self.line_starts[line];
        let line_slice = &source[line_start..byte_pos];
        let line_str = std::str::from_utf8(line_slice)?;
        let col = line_str.chars().count();

        Ok((line, col))
    }
}

pub enum HighlightType {
    Start(HighlightSpan),
    End
}

pub fn create_syntax_highlighting(source: &[u8]) -> Result<HashMap<(usize, usize), HighlightType>, Box<dyn std::error::Error>> {
    let highlight_names = [
        "attribute",
        "comment",
        "constant",
        "constant.builtin",
        "constructor",
        "embedded",
        "function",
        "function.builtin",
        "keyword",
        "module",
        "number",
        "operator",
        "property",
        "property.builtin",
        "punctuation",
        "punctuation.bracket",
        "punctuation.delimiter",
        "punctuation.special",
        "string",
        "string.special",
        "tag",
        "type",
        "type.builtin",
        "variable",
        "variable.builtin",
        "variable.parameter",
    ];

    let rust_language = tree_sitter_rust::LANGUAGE;
    let mut highlighter = Highlighter::new();
    let byte_to_line_conv = ByteToLineCol::new(source);

    let mut rust_config = HighlightConfiguration::new(
        rust_language.into(),
        "rust",
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        tree_sitter_rust::INJECTIONS_QUERY,
        "",
    )?;

    rust_config.configure(&highlight_names);

    
    let highlights = highlighter.highlight(
        &rust_config,
        source,
        None,
        |_| None
    )?;

    let mut result: HashMap<(usize, usize), HighlightType> = HashMap::new();
    // syntax highlight can be nested
    let mut highlight_stack = Vec::new();
    let mut current_pos = 0;

    for event in highlights {
        match event? {
            HighlightEvent::Source { start: _, end } => {
                current_pos = end;
            },
            HighlightEvent::HighlightStart(highlight) => {
                highlight_stack.push((highlight.0, current_pos));
            },
            HighlightEvent::HighlightEnd => {
                if let Some((highlight_index, start_pos)) = highlight_stack.pop() {
                    let highlight_name = highlight_names[highlight_index];
                    let (start_line, start_col) = byte_to_line_conv.byte_to_line_col(start_pos, source)?;
                    let (end_line, end_col) = byte_to_line_conv.byte_to_line_col(current_pos, source)?;

                    result.insert(
                        (start_line, start_col),
                        HighlightType::Start(HighlightSpan { highlight_name, start_line, start_col })
                    );

                    result.insert((end_line, end_col), HighlightType::End);
                }
            },
        }
    }

    Ok(result)
}