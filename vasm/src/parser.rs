use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "vasm.pest"]
pub struct VASMParser;
