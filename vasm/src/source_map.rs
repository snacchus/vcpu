#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceMapItem {
    pub start_line: u32,
    pub line_count: u32,
}

pub type SourceMap = Vec<SourceMapItem>;
