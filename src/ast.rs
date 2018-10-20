use std;
use std::collections::HashMap;
use serde_json;

pub type Name = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expr {
  Local{name: Name},
  Global{name: Name},
  External{name: Name},
  Pack{tag: usize, arity: usize},
  Num{int: i64},
  Ap{fun: Box<Expr>, args: Vec<Expr>},
  Let{isrec: bool, defns: Vec<Defn>, body: Box<Expr>},
  Match{expr: Box<Expr>, altns: Vec<Altn>}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Defn {
  pub lhs: Name,
  pub rhs: Expr
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Altn {
  pub binds: Vec<Option<Name>>,
  pub rhs: Expr
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TopLevel {
  Def{name: Name, binds: Vec<Option<Name>>, body: Expr},
  Asm{name: Name}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lambda {
  pub binds: Vec<Option<Name>>,
  pub body: Expr
}

pub type Module = HashMap<Name, Lambda>;

impl TopLevel {
  fn lambda(self) -> Option<(Name, Lambda)> {
    match self {
      TopLevel::Def{name, binds, body} => Some((name, Lambda{binds, body})),
      TopLevel::Asm{..} => None
    }
  }
}

pub fn load_module<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Module> {
  use std::fs::File;
  let file: File = File::open(path)?;
  let top_levels: Vec<TopLevel> = serde_json::from_reader(file)?;
  let module: Module = top_levels.into_iter().filter_map(TopLevel::lambda).collect();
  Ok(module)
}