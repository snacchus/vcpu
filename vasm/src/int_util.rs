use crate::*;
use num::{Num, Signed, Unsigned};
use pest::iterators::Pair;
use std::num::ParseIntError;

pub trait GetUnsigned: Signed {
    type Unsigned;
}

impl GetUnsigned for i8 {
    type Unsigned = u8;
}

impl GetUnsigned for i16 {
    type Unsigned = u16;
}

impl GetUnsigned for i32 {
    type Unsigned = u32;
}

pub trait ToPrimitiveTrunc: Sized {
    fn to_i8(&self) -> i8;
    fn to_i16(&self) -> i16;
    fn to_i32(&self) -> i32;
}

macro_rules! impl_to_prim_trunc {
    ($T:ty) => {
        impl ToPrimitiveTrunc for $T {
            #[inline]
            fn to_i8(&self) -> i8 {
                *self as i8
            }
            #[inline]
            fn to_i16(&self) -> i16 {
                *self as i16
            }
            #[inline]
            fn to_i32(&self) -> i32 {
                *self as i32
            }
        }
    };
}

impl_to_prim_trunc!(u8);
impl_to_prim_trunc!(u16);
impl_to_prim_trunc!(u32);

pub trait NumCastTrunc: Sized {
    fn from<T: ToPrimitiveTrunc>(n: T) -> Self;
}

macro_rules! impl_num_cast_trunc {
    ($T:ty, $conv:ident) => {
        impl NumCastTrunc for $T {
            #[inline]
            fn from<N: ToPrimitiveTrunc>(n: N) -> $T {
                n.$conv()
            }
        }
    };
}

impl_num_cast_trunc!(i8, to_i8);
impl_num_cast_trunc!(i16, to_i16);
impl_num_cast_trunc!(i32, to_i32);

fn process_num_lit<T>(pair: Pair<Rule>, base: u32) -> Result<T>
where
    T: Num<FromStrRadixErr = ParseIntError>,
{
    let span = pair.as_span();
    T::from_str_radix(span.as_str(), base)
        .map_err(|err| new_parser_error(span, format!("Parsing integer failed: {}", err)))
}

fn process_unsigned_lit<T>(pair: Pair<Rule>, base: u32) -> Result<T>
where
    T: GetUnsigned + NumCastTrunc,
    <T as GetUnsigned>::Unsigned: Num<FromStrRadixErr = ParseIntError> + ToPrimitiveTrunc,
{
    Ok(NumCastTrunc::from(process_num_lit::<T::Unsigned>(
        pair, base,
    )?))
}

pub fn process_uint<T>(pair: Pair<Rule>) -> Result<T>
where
    T: Unsigned + Num<FromStrRadixErr = ParseIntError>,
{
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::bin_uint => process_num_lit(inner.into_inner().next().unwrap(), 2),
        Rule::oct_uint => process_num_lit(inner.into_inner().next().unwrap(), 8),
        Rule::hex_uint => process_num_lit(inner.into_inner().next().unwrap(), 16),
        Rule::dec_uint => process_num_lit(inner, 10),
        _ => unreachable!(),
    }
}

pub fn process_int<T>(pair: Pair<Rule>) -> Result<T>
where
    T: GetUnsigned + Num<FromStrRadixErr = ParseIntError> + NumCastTrunc,
    <T as GetUnsigned>::Unsigned: Num<FromStrRadixErr = ParseIntError> + ToPrimitiveTrunc,
{
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::bin_uint => process_unsigned_lit(inner.into_inner().next().unwrap(), 2),
        Rule::oct_uint => process_unsigned_lit(inner.into_inner().next().unwrap(), 8),
        Rule::hex_uint => process_unsigned_lit(inner.into_inner().next().unwrap(), 16),
        Rule::dec_int => process_num_lit(inner, 10),
        _ => unreachable!(),
    }
}
