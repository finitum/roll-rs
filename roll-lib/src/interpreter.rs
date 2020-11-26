use crate::filtermodifier::FilterModifier;
use crate::roll::{roll_die, Roll};
use core::fmt;
use core::option::Option::Some;
use core::result::Result::{Err, Ok};
use rand_core::OsRng;
use std::num::NonZeroU64;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

pub const DEFAULT_SIDES: &str = "20";

#[derive(Debug, PartialEq)]
pub enum Value {
    Float(f64),
    Int(i64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float(v) => f.write_str(&v.to_string()),
            Self::Int(v) => f.write_str(&v.to_string()),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => Value::Float(i as f64 + j as f64),
            (Value::Int(i), Value::Float(j)) => Value::Float(i as f64 + j as f64),
            (Value::Float(i), Value::Int(j)) => Value::Float(i as f64 + j as f64),
            (Value::Int(i), Value::Int(j)) => Value::Int(i + j),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => Value::Float(i as f64 - j),
            (Value::Int(i), Value::Float(j)) => Value::Float(i as f64 - j as f64),
            (Value::Float(i), Value::Int(j)) => Value::Float(i as f64 - j as f64),
            (Value::Int(i), Value::Int(j)) => Value::Int(i - j),
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => Value::Float(i as f64 * j as f64),
            (Value::Int(i), Value::Float(j)) => Value::Float(i as f64 * j as f64),
            (Value::Float(i), Value::Int(j)) => Value::Float(i as f64 * j as f64),
            (Value::Int(i), Value::Int(j)) => Value::Int(i * j),
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => Value::Float(i as f64 / j as f64),
            (Value::Int(i), Value::Float(j)) => Value::Float(i as f64 / j as f64),
            (Value::Float(i), Value::Int(j)) => Value::Float(i as f64 / j as f64),
            (Value::Int(i), Value::Int(j)) => Value::Float(i as f64 / j as f64),
        }
    }
}

impl Rem for Value {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => Value::Float(i as f64 % j as f64),
            (Value::Int(i), Value::Float(j)) => Value::Float(i as f64 % j as f64),
            (Value::Float(i), Value::Int(j)) => Value::Float(i as f64 % j as f64),
            (Value::Int(i), Value::Int(j)) => Value::Int(i % j),
        }
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Value::Float(i) => Value::Float(-i),
            Value::Int(i) => Value::Int(-i),
        }
    }
}

impl Value {
    pub fn floor(self) -> Self {
        match self {
            Value::Float(i) => Value::Int(i.floor() as i64),
            i => i,
        }
    }

    pub fn pow(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => Value::Float((i as f64).powf(j as f64)),
            (Value::Int(i), Value::Float(j)) => Value::Float((i as f64).powf(j as f64)),
            (Value::Float(i), Value::Int(j)) => Value::Float((i as f64).powf(j as f64)),
            (Value::Int(i), Value::Int(j)) => Value::Int((i as i64).pow(j as u32)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Ast {
    Add(Box<Ast>, Box<Ast>),
    Sub(Box<Ast>, Box<Ast>),
    Mul(Box<Ast>, Box<Ast>),
    Div(Box<Ast>, Box<Ast>),
    Mod(Box<Ast>, Box<Ast>),
    IDiv(Box<Ast>, Box<Ast>),
    Power(Box<Ast>, Box<Ast>),
    Minus(Box<Ast>),
    Dice(
        Option<Box<Ast>>,
        Option<Box<Ast>>,
        FilterModifier<Box<Ast>>,
        u64,
    ),

    Const(String),
}

impl Ast {
    pub fn interp(self, rolls: &mut Vec<(u64, Roll)>) -> Result<Value, String> {
        Ok(match self {
            Ast::Add(l, r) => l.interp(rolls)? + r.interp(rolls)?,
            Ast::Sub(l, r) => l.interp(rolls)? - r.interp(rolls)?,
            Ast::Div(l, r) => l.interp(rolls)? / r.interp(rolls)?,
            Ast::Mul(l, r) => l.interp(rolls)? * r.interp(rolls)?,
            Ast::Mod(l, r) => l.interp(rolls)? % r.interp(rolls)?,
            Ast::IDiv(l, r) => (l.interp(rolls)? / r.interp(rolls)?).floor(),
            Ast::Power(l, r) => l.interp(rolls)?.pow(r.interp(rolls)?),

            Ast::Minus(l) => -l.interp(rolls)?,

            Ast::Const(val) => {
                let dots = val.matches('.').count();
                if dots == 0 {
                    Value::Int(val.parse::<i64>().map_err(|e| e.to_string())?)
                } else if dots == 1 {
                    Value::Float(val.parse::<f64>().map_err(|e| e.to_string())?)
                } else {
                    return Err(format!(
                        "{} couldn't be parsed as number (too many dots)",
                        val
                    ));
                }
            }

            Ast::Dice(None, r, fm, dp) => {
                Ast::Dice(Some(Box::new(Ast::Const("1".to_string()))), r, fm, dp).interp(rolls)?
            }
            Ast::Dice(l, None, fm, dp) => Ast::Dice(
                l,
                Some(Box::new(Ast::Const(DEFAULT_SIDES.to_string()))),
                fm,
                dp,
            )
            .interp(rolls)?,

            Ast::Dice(Some(l), Some(r), fm, dp) => {
                if let (Value::Int(lv), Value::Int(rv)) = (l.interp(rolls)?, r.interp(rolls)?) {
                    let fm_value: FilterModifier<Value> = fm.map(|i| i.interp(rolls)).swap()?;

                    let fm_int = fm_value
                        .map(|i| {
                            if let Value::Int(v) = i {
                                Ok(v as u64)
                            } else {
                                Err(format!("{:?}: couldn't be parsed as int", i))
                            }
                        })
                        .swap()?;

                    let roll = roll_die(
                        lv as u64,
                        NonZeroU64::new(rv as u64).ok_or("Can't roll zero sided die")?,
                        fm_int,
                        OsRng,
                    );
                    let total = roll.total;

                    rolls.push((dp, roll));
                    Value::Int(total)
                } else {
                    return Err("couldn't be parsed as dice roll (no ints)".to_string());
                }
            }
        })
    }
}
