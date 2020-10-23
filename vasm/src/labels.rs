use crate::*;
use pest::iterators::Pair;
use std::collections::HashMap;

pub type LabelMap<'i> = HashMap<&'i str, u32>;

pub fn process_labeled_element<'i, F>(
    pair: Pair<'i, Rule>,
    labels: &mut LabelMap<'i>,
    rule: Rule,
    len: u32,
    op: F,
) -> Result<()>
where
    F: FnOnce(Pair<'i, Rule>) -> Result<()>,
{
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    let r = first.as_rule();
    if r == Rule::label {
        let label_str = first.into_inner().next().unwrap().as_span().as_str();
        labels.insert(label_str, len);
        op(pairs.next().unwrap())?;
    } else if r == rule {
        op(first)?;
    } else {
        unreachable!();
    }

    Ok(())
}
