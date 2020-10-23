use crate::int_util::*;
use crate::labels::*;
use crate::*;
use byteorder::ByteOrder;
use matches::debug_assert_matches;
use num::{Num, ToPrimitive};
use pest::iterators::Pair;
use std::collections::HashMap;
use std::num::ParseIntError;
use util::Endian;

fn process_int_list<T>(pair: Pair<Rule>, data: &mut Vec<u8>) -> Result<()>
where
    T: GetUnsigned + Num<FromStrRadixErr = ParseIntError> + ToPrimitive + NumCastTrunc,
    <T as GetUnsigned>::Unsigned: Num<FromStrRadixErr = ParseIntError> + ToPrimitiveTrunc,
{
    let pairs = pair.into_inner();
    let element_size = std::mem::size_of::<T>();
    let (lower, upper) = pairs.size_hint();
    let estimated_size = if let Some(upper_bound) = upper {
        upper_bound
    } else {
        lower
    };
    data.reserve(estimated_size * element_size);

    for int in pairs {
        let span = int.as_span();
        let value = process_int::<T>(int)?
            .to_i64()
            .ok_or_else(|| new_parser_error(span, "Cannot cast integer".to_owned()))?;
        let current_size = data.len();
        let new_size = current_size + element_size;
        data.resize(new_size, 0u8);
        Endian::write_int(&mut data[current_size..new_size], value, element_size);
    }
    Ok(())
}

fn process_data_element(pair: Pair<Rule>, data: &mut Vec<u8>) -> Result<()> {
    debug_assert_matches!(pair.as_rule(), Rule::data_element);
    let inner = pair.into_inner().next().unwrap();
    let span = inner.as_span();

    match inner.as_rule() {
        Rule::data_block => {
            let element_size = process_uint::<usize>(inner.into_inner().next().unwrap())?;
            let new_size = data.len().checked_add(element_size).ok_or_else(|| {
                new_parser_error(span.clone(), "Data block is too big".to_owned())
            })?;
            data.resize(new_size, 0u8);
        }
        Rule::data_byte => process_int_list::<i8>(inner.into_inner().next().unwrap(), data)?,
        Rule::data_short => process_int_list::<i16>(inner.into_inner().next().unwrap(), data)?,
        Rule::data_word => process_int_list::<i32>(inner.into_inner().next().unwrap(), data)?,
        _ => unreachable!(),
    };

    let max_size = u32::max_value() as usize - 1;

    if data.len() > max_size {
        Err(new_parser_error(
            span,
            format!("Data exceeds maximum size of {} bytes", max_size),
        ))
    } else {
        Ok(())
    }
}

pub fn process_data(pair: Pair<Rule>) -> Result<(Vec<u8>, LabelMap)> {
    debug_assert_matches!(pair.as_rule(), Rule::data);

    let mut data = Vec::new();
    let mut labels = HashMap::new();

    for labeled_data_element in pair.into_inner() {
        process_labeled_element(
            labeled_data_element,
            &mut labels,
            Rule::data_element,
            data.len() as u32,
            |p| process_data_element(p, &mut data),
        )?;
    }

    Ok((data, labels))
}

#[cfg(test)]
mod test {
    use crate::test::parse_rule;
    use crate::Rule;

    #[test]
    fn large_hexadecimal_data_word() {
        let input = ".word 0xFFFFFFFF";
        let mut output = Vec::new();

        let pair = parse_rule(Rule::data_element, input).unwrap();
        super::process_data_element(pair, &mut output).unwrap();

        assert_eq!([0xFF, 0xFF, 0xFF, 0xFF], &output[..]);
    }

    #[test]
    fn large_hexadecimal_data_short() {
        let input = ".short 0xFFFF";
        let mut output = Vec::new();

        let pair = parse_rule(Rule::data_element, input).unwrap();
        super::process_data_element(pair, &mut output).unwrap();

        assert_eq!([0xFF, 0xFF], &output[..]);
    }

    #[test]
    fn large_hexadecimal_data_byte() {
        let input = ".byte 0xFF";
        let mut output = Vec::new();

        let pair = parse_rule(Rule::data_element, input).unwrap();
        super::process_data_element(pair, &mut output).unwrap();

        assert_eq!([0xFF], &output[..]);
    }

    #[test]
    fn negative_signed_data_word() {
        let input = ".word -1234";
        let mut output = Vec::new();

        let pair = parse_rule(Rule::data_element, input).unwrap();
        super::process_data_element(pair, &mut output).unwrap();

        assert_eq!([0x2E, 0xFB, 0xFF, 0xFF], &output[..]);
    }

    #[test]
    fn negative_signed_data_short() {
        let input = ".short -1234";
        let mut output = Vec::new();

        let pair = parse_rule(Rule::data_element, input).unwrap();
        super::process_data_element(pair, &mut output).unwrap();

        assert_eq!([0x2E, 0xFB], &output[..]);
    }

    #[test]
    fn negative_signed_data_byte() {
        let input = ".byte -123";
        let mut output = Vec::new();

        let pair = parse_rule(Rule::data_element, input).unwrap();
        super::process_data_element(pair, &mut output).unwrap();

        assert_eq!([0x85], &output[..]);
    }
}
