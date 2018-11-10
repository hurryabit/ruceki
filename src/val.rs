use std::io::Read;
use std::rc::Rc;

use crate::ast::{External, Lambda, Name};

#[derive(Debug, Copy, Clone)]
pub enum Prim<'a> {
  Global(&'a Name, &'a Lambda),
  External(External),
  Pack(usize, usize),
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
  Num(i64),
  Pack(usize, Vec<Rc<Value<'a>>>),
  PAP(Prim<'a>, Vec<Rc<Value<'a>>>, usize),
}

impl<'a> Value<'a> {
  pub fn rc_unit() -> Rc<Self> {
    Rc::new(Value::Pack(0, Vec::new()))
  }

  pub fn rc_from_bool(b: bool) -> Rc<Self> {
    Rc::new(Value::Pack(b.into(), Vec::new()))
  }

  pub fn rc_from_i64(n: i64) -> Rc<Self> {
    Rc::new(Value::Num(n))
  }

  pub fn as_i64(&self) -> i64 {
    match self {
      Value::Num(n) => *n,
      _ => panic!("Expected Int, found {:?}", self),
    }
  }

  pub fn eval_external(name: External, args: &Vec<Rc<Self>>) -> Rc<Self> {
    use self::External::*;
    if args.len() != name.arity() {
      panic!(
        "Found {} arguments for {:?}/{}",
        args.len(),
        name,
        name.arity()
      );
    }
    match name {
      add => Value::rc_from_i64(args[0].as_i64() + args[1].as_i64()),
      sub => Value::rc_from_i64(args[0].as_i64() - args[1].as_i64()),
      mul => Value::rc_from_i64(args[0].as_i64() * args[1].as_i64()),
      neg => Value::rc_from_i64(-args[0].as_i64()),
      eq => Value::rc_from_bool(args[0].as_i64() == args[1].as_i64()),
      le => Value::rc_from_bool(args[0].as_i64() <= args[1].as_i64()),
      lt => Value::rc_from_bool(args[0].as_i64() < args[1].as_i64()),
      gt => Value::rc_from_bool(args[0].as_i64() > args[1].as_i64()),
      ge => Value::rc_from_bool(args[0].as_i64() >= args[1].as_i64()),
      chr => Value::rc_from_i64(args[0].as_i64() & 0xFF),
      ord => Value::rc_from_i64(args[0].as_i64()),
      puti => {
        println!("{}", args[0].as_i64());
        Value::rc_unit()
      }
      putc => {
        print!("{}", args[0].as_i64() as u8 as char);
        Value::rc_unit()
      }
      geti => {
        let mut input = String::new();
        std::io::stdin()
          .read_line(&mut input)
          .expect("Failed to read line");
        Value::rc_from_i64(input.trim().parse().expect("Input not a number"))
      }
      getc => {
        let mut input = [0];
        let n = match std::io::stdin().read_exact(&mut input) {
          Ok(()) => input[0] as i64,
          Err(_) => -1,
        };
        Value::rc_from_i64(n)
      }
      seq => Rc::clone(&args[1]),
    }
  }
}
