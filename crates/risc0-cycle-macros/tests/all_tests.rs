#[test]
fn cycle_macro_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cycle_macro.rs");
    t.pass("tests/cycle_macro_with_docs.rs");
}
