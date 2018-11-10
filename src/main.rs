use std::env;

mod ast;
mod cek;
mod val;
#[cfg(test)]
mod tests;

fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let filename = &args[1];

  let stdin = std::io::stdin();
  let mut stdin = stdin.lock();
  let stdout = std::io::stdout();
  let mut stdout = stdout.lock();
  let entry_point = ast::Expr::entry_point();
  let module: ast::Module = ast::load_module(filename)?;
  eprintln!("Loaded\n==========");

  let mut state = cek::State::from_expr(&entry_point, &mut stdin, &mut stdout);
  let count = state.run(&module);
  eprintln!("==========\nSteps: {}", count);

  Ok(())
}
