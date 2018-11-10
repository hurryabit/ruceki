use crate::ast;
use crate::cek;

fn test_text(name: &str, input: &str, expected_output: &str) {
  let file = "test/".to_string() + name + ".pub";
  let module: ast::Module = ast::load_module(file).unwrap();
  let entry_point = ast::Expr::entry_point();
  let mut input = input.as_bytes();
  let mut actual_output = Vec::new();
  let mut state = cek::State::from_expr(&entry_point, &mut input, &mut actual_output);
  let _count = state.run(&module);
  let actual_output = std::str::from_utf8(&actual_output).unwrap();
  assert!(actual_output == expected_output, "\n  expected output: {:?}\n    actual output: {:?}\n", expected_output, actual_output);
}

fn test_numeric(name: &str, input: &[i64], expected_output: &[i64]) {
  let input: String = input.iter().map(|x| x.to_string() + "\n").collect();
  let expected_output: String = expected_output.iter().map(|x| x.to_string() + "\n").collect();
  test_text(name, &input, &expected_output);
}

fn test_sort(name: &str) {
  use proptest::test_runner::{Config, TestRunner};
  let config = Config {
    cases: 20,
    .. Config::default()
  };
  let mut runner = TestRunner::new(config);
  let strategy = proptest::collection::vec(-100..100_i64, 0..100);
  let result = runner.run(&strategy, |v| {
    let mut input = vec![v.len() as i64];
    input.extend(v.iter());
    let mut expected_output = v.clone();
    expected_output.sort();
    test_numeric(name, &input, &expected_output);
    Ok(())
  });
  result.expect("test_sort failed");
}

#[test]
fn hello() {
  test_text("hello", "", "Hello World!\n");
}

#[test]
fn rev() {
  test_text("rev", "abc", "cba");
}

#[test]
fn monad_io() {
  test_numeric("monad_io", &[3, 2],  &[2, 1, 0, 2, 1, 0, 2, 1, 0]);
}

#[test]
fn wildcard() {
  test_numeric("wildcard", &[7, 13], &[7, 13]);
}

#[test]
fn queens() {
  test_numeric("queens", &[8], &[92]);
}

#[test]
fn qsort() {
  test_sort("qsort");
}

#[test]
fn isort() {
  test_sort("isort");
}
