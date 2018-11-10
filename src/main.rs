use std::env;

mod ast;
mod cek;
mod val;

fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let filename = &args[1];

  let entry_point = ast::Expr::entry_point();
  let module: ast::Module = ast::load_module(filename)?;
  eprintln!("Loaded\n==========");

  let mut state = cek::State::from_expr(&entry_point);
  let count = state.run(&module);
  eprintln!("==========\nSteps: {}", count);

  Ok(())
}
