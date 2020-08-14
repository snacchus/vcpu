use super::*;

#[test]
fn works() {
    instruction_runs! {
        instr_flop!(FDIV, T0, T1, T2),
        [
            T1 = 262.562_f32,
            T2 = -82.35_f32
        ] => [
            T0 = -3.188_367_f32
        ]
    }
}
