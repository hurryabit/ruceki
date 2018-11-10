use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Name = String;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum External {
  add,
  sub,
  mul,
  neg,
  eq,
  le,
  lt,
  gt,
  ge,
  chr,
  ord,
  puti,
  putc,
  geti,
  getc,
  seq,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
  Local { name: Name, idx: usize },
  Global { name: Name },
  External { name: External },
  Pack { tag: usize, arity: usize },
  Num { int: i64 },
  Ap { fun: Box<Expr>, args: Vec<Expr> },
  Let { defn: Box<Defn>, body: Box<Expr> },
  Match { expr: Box<Expr>, altns: Vec<Altn> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Defn {
  pub lhs: Name,
  pub rhs: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Altn {
  pub binds: Vec<Option<Name>>,
  pub rhs: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TopLevel {
  Def {
    name: Name,
    binds: Vec<Option<Name>>,
    body: Expr,
  },
  Asm {
    name: Name,
  },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lambda {
  pub binds: Vec<Option<Name>>,
  pub body: Expr,
}

pub type Module = HashMap<Name, Lambda>;

impl External {
  pub fn arity(self) -> usize {
    use self::External::*;
    match self {
      add => 2,
      sub => 2,
      mul => 2,
      neg => 1,
      eq => 2,
      le => 2,
      lt => 2,
      gt => 2,
      ge => 2,
      chr => 1,
      ord => 1,
      puti => 1,
      putc => 1,
      geti => 1,
      getc => 1,
      seq => 2,
    }
  }
}

impl Expr {
  pub fn entry_point() -> Expr {
    Expr::Ap {
      fun: Box::new(Expr::Global {
        name: String::from("main"),
      }),
      args: vec![Expr::Pack { tag: 0, arity: 0 }],
    }
  }
}

impl TopLevel {
  fn lambda(self) -> Option<(Name, Lambda)> {
    match self {
      TopLevel::Def { name, binds, body } => Some((name, Lambda { binds, body })),
      TopLevel::Asm { .. } => None,
    }
  }
}

pub fn load_module<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Module> {
  use std::fs::File;
  let file: File = File::open(path)?;
  let top_levels: Vec<TopLevel> = serde_json::from_reader(file)?;
  let module: Module = top_levels
    .into_iter()
    .filter_map(TopLevel::lambda)
    .collect();
  Ok(module)
}
