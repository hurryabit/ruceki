use std::borrow::Borrow;
use std::env;
use std::io::Read;
use std::rc::Rc;

mod ast;
use crate::ast::{Altn, Defn, External, Expr, Lambda, Module, Name};

#[derive(Debug, Copy, Clone)]
enum Prim<'a> {
  Global(&'a Name, &'a Lambda),
  External(External),
  Pack(usize, usize),
}

#[derive(Debug, Clone)]
enum Value<'a> {
  Num(i64),
  Pack(usize, Vec<Rc<Value<'a>>>),
  PAP(Prim<'a>, Vec<Rc<Value<'a>>>, usize),
}

#[derive(Debug)]
enum Ctrl<'a> {
  Evaluating,
  Expr(&'a Expr),
  Value(Rc<Value<'a>>),
}

#[derive(Debug)]
struct Env<'a> {
  stack: Vec<Rc<Value<'a>>>,
}

impl<'a> Env<'a> {
  fn new() -> Self {
    Env { stack: Vec::new() }
  }

  fn get(&self, idx: usize) -> &Rc<Value<'a>> {
    self
      .stack
      .get(self.stack.len() - idx)
      .expect("Bad de Bruijn index")
  }

  fn push(&mut self, value: Rc<Value<'a>>) {
    self.stack.push(value);
  }

  fn push_many(&mut self, args: &Vec<Rc<Value<'a>>>) {
    self.stack.extend_from_slice(args);
  }

  fn pop(&mut self, count: usize) {
    let new_len = self.stack.len() - count;
    self.stack.truncate(new_len);
  }
}

#[derive(Debug)]
enum Kont<'a> {
  Dump(Env<'a>),
  Pop(usize),
  Args(&'a [Expr]),
  Fun(Prim<'a>, Vec<Rc<Value<'a>>>, usize),
  Match(&'a Vec<Altn>),
  Let(&'a Name, &'a Expr),
}

#[derive(Debug)]
struct State<'a> {
  ctrl: Ctrl<'a>,
  env: Env<'a>,
  kont: Vec<Kont<'a>>,
}

impl<'a> Value<'a> {
  fn rc_unit() -> Rc<Self> {
    Rc::new(Value::Pack(0, Vec::new()))
  }

  fn rc_from_bool(b: bool) -> Rc<Self> {
    Rc::new(Value::Pack(b.into(), Vec::new()))
  }

  fn rc_from_i64(n: i64) -> Rc<Self> {
    Rc::new(Value::Num(n))
  }

  fn as_i64(&self) -> &i64 {
    match self {
      Value::Num(n) => n,
      _ => panic!("Expected Int, found {:?}", self),
    }
  }
}

impl<'a> Ctrl<'a> {
  fn from_prim(prim: Prim<'a>, arity: usize) -> Self {
    Ctrl::Value(Rc::new(Value::PAP(prim, Vec::new(), arity)))
  }
}

impl<'a> State<'a> {
  fn from_expr(expr: &'a Expr) -> Self {
    State {
      ctrl: Ctrl::Expr(expr),
      env: Env::new(),
      kont: Vec::new(),
    }
  }

  fn step(&mut self, module: &'a Module) {
    let old_ctrl = std::mem::replace(&mut self.ctrl, Ctrl::Evaluating);

    let new_ctrl = match old_ctrl {
      Ctrl::Evaluating => panic!("Control was not update after last step"),

      Ctrl::Expr(Expr::Local { idx, .. }) => {
        let v = self.env.get(*idx);
        Ctrl::Value(Rc::clone(&v))
      }
      Ctrl::Expr(Expr::Global { name }) => {
        let lam = module
          .get(name)
          .expect(&format!("Unknown global: {}", name));
        Ctrl::from_prim(Prim::Global(name, lam), lam.binds.len())
      }
      Ctrl::Expr(Expr::External { name }) => {
        Ctrl::from_prim(Prim::External(*name), name.arity())
      }
      Ctrl::Expr(&Expr::Pack { tag, arity }) => Ctrl::from_prim(Prim::Pack(tag, arity), arity),
      Ctrl::Expr(&Expr::Num { int }) => Ctrl::Value(Value::rc_from_i64(int)),
      Ctrl::Expr(Expr::Ap { fun, args }) => {
        self.kont.push(Kont::Args(args));
        Ctrl::Expr(fun)
      }
      Ctrl::Expr(Expr::Let { defn, body }) => {
        let Defn { lhs, rhs } = defn.borrow();
        self.kont.push(Kont::Let(lhs, body));
        Ctrl::Expr(rhs)
      }
      Ctrl::Expr(Expr::Match { expr, altns }) => {
        self.kont.push(Kont::Match(altns));
        Ctrl::Expr(expr)
      }

      Ctrl::Value(v) => match v.borrow() {
        Value::PAP(prim, args, 0) => match prim {
          Prim::Global(_name, lam) => {
            let Lambda { body, .. } = lam;
            let mut new_env = Env::new();
            new_env.push_many(args);
            let old_env = std::mem::replace(&mut self.env, new_env);
            self.kont.push(Kont::Dump(old_env));
            Ctrl::Expr(body)
          }
          Prim::External(name) => Ctrl::Value(external(*name, &args)),
          Prim::Pack(tag, _arity) => Ctrl::Value(Rc::new(Value::Pack(*tag, args.clone()))),
        },

        _ => match self.kont.pop().expect("Step on final state") {
          Kont::Dump(env) => {
            self.env = env;
            Ctrl::Value(Rc::clone(&v))
          }
          Kont::Pop(count) => {
            self.env.pop(count);
            Ctrl::Value(Rc::clone(&v))
          }
          Kont::Args(next_args) => match v.borrow() {
            Value::PAP(prim, args, missing) => {
              let (next_arg, next_args) = next_args.split_first().expect("Empty Args");
              if !next_args.is_empty() {
                self.kont.push(Kont::Args(next_args));
              }
              self.kont.push(Kont::Fun(*prim, args.clone(), *missing));
              Ctrl::Expr(next_arg)
            }
            _ => panic!("Applying value"),
          },
          Kont::Fun(prim2, mut args2, missing2) => {
            args2.push(Rc::clone(&v));
            Ctrl::Value(Rc::new(Value::PAP(prim2, args2, missing2 - 1)))
          }
          Kont::Match(altns) => match v.borrow() {
            Value::Pack(tag, args) => {
              let Altn { rhs, .. } = &altns[*tag];
              self.kont.push(Kont::Pop(args.len()));
              self.env.push_many(&args);
              Ctrl::Expr(rhs)
            }
            _ => panic!("Pattern match on non-data value"),
          },
          Kont::Let(_name, body) => {
            self.kont.push(Kont::Pop(1));
            self.env.push(Rc::clone(&v));
            Ctrl::Expr(body)
          }
        },
      },
    };

    self.ctrl = new_ctrl
  }

  fn is_final(&self) -> bool {
    match self.ctrl.borrow() {
      Ctrl::Value(v) => match v.borrow() {
        Value::Num(_) | Value::Pack(_, _) => self.kont.is_empty(),
        _ => false,
      },
      _ => false,
    }
  }
}

fn external<'a>(name: External, args: &Vec<Rc<Value<'a>>>) -> Rc<Value<'a>> {
  use self::External::*;
  if args.len() != name.arity() {
    panic!("Found {} arguments for {:?}/{}", args.len(), name, name.arity());
  }
  match name {
    add => Value::rc_from_i64(args[0].as_i64() + args[1].as_i64()),
    sub => Value::rc_from_i64(args[0].as_i64() - args[1].as_i64()),
    mul => Value::rc_from_i64(args[0].as_i64() * args[1].as_i64()),
    neg => Value::rc_from_i64(-args[0].as_i64()),
    eq => Value::rc_from_bool(args[0].as_i64() == args[1].as_i64()),
    le => Value::rc_from_bool(args[0].as_i64() <= args[1].as_i64()),
    lt => Value::rc_from_bool(args[0].as_i64() <  args[1].as_i64()),
    gt => Value::rc_from_bool(args[0].as_i64() >  args[1].as_i64()),
    ge => Value::rc_from_bool(args[0].as_i64() >= args[1].as_i64()),
    chr => Value::rc_from_i64(args[0].as_i64() & 0xFF),
    ord => Value::rc_from_i64(*args[0].as_i64()),
    puti => {
      println!("{}", args[0].as_i64());
      Value::rc_unit()
    },
    putc => {
      print!("{}", *args[0].as_i64() as u8 as char);
      Value::rc_unit()
    },
    geti => {
      let mut input = String::new();
      std::io::stdin().read_line(&mut input).expect("Failed to read line");
      Value::rc_from_i64(input.trim().parse().expect("Input not a number"))
    },
    getc => {
      let mut input = [0];
      let n = match std::io::stdin().read_exact(&mut input) {
        Ok(()) => input[0] as i64,
        Err(_) => -1,
      };
      Value::rc_from_i64(n)
    },
    seq => Rc::clone(&args[1]),
  }
}

fn main() -> std::io::Result<()> {
  let debug = false;
  let args: Vec<String> = env::args().collect();
  let filename = &args[1];

  let entry_point = Expr::entry_point();
  let module: Module = ast::load_module(filename)?;
  eprintln!("Loaded!");

  // let mut state = State::enter_caf(String::from("test"));
  let mut state = State::from_expr(&entry_point);
  let mut count = 0;
  if debug {
    eprintln!("State 0: {:?}", state);
  }
  while !state.is_final() {
    state.step(&module);
    count += 1;
    if debug {
      eprintln!("State {}: {:?}", count, state);
    }
  }
  eprintln!("==========\nSteps: {}", count);

  if debug {
    eprintln!("==========\nResult: {:?}", state.ctrl);
  }

  Ok(())
}
