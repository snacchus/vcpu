use super::*;

#[test]
fn works() {
    instruction_runs! {
        instr_flop!(FSUB, T0, T1, T2),
        [
            T1 = 262.562_f32,
            T2 = -82.35_f32
        ] => [
            T0 = 344.912_02_f32
        ]
    }
}
