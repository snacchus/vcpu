use super::*;

#[test]
fn write_read() {
    let path = "test.vex";

    let data = vec![34, 1, 231, 60, 4, 2, 255];
    let instructions = vec![
        30, 27, 34, 50, 91, 83, 49, 5, 6, 134, 61, 2, 6, 239, 34, 8, 15,
    ];

    let executable_orig = Executable::from(0, data, instructions);

    write_file(path, &executable_orig).unwrap();

    let executable_read = read_file(path).unwrap();

    assert_eq!(executable_orig, executable_read);

    std::fs::remove_file(path).unwrap();
}
