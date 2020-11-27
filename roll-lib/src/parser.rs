use crate::filtermodifier::FilterModifier;
use crate::interpreter::Ast;
use crate::options::Options;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct Parser<'a> {
    expr: Peekable<Chars<'a>>,
    pos: u64,
    source: String,

    pub advanced: bool,
}

impl<'a> Parser<'a> {
    pub fn new(expr: &'a str) -> Self {
        Self {
            source: expr.to_string(),
            expr: expr.chars().peekable(),
            pos: 0,
            advanced: false,
        }
    }

    pub fn advanced(mut self) -> Self {
        self.advanced = true;
        self
    }

    pub fn backup(&self) -> Self {
        Self {
            expr: self.expr.clone(),
            source: self.source.clone(),
            pos: self.pos,
            advanced: self.advanced,
        }
    }

    pub fn restore(&mut self, other: Self) {
        self.expr = other.expr;
        self.pos = other.pos;
        self.source = other.source;
        self.advanced = other.advanced;
    }

    pub fn accept(&mut self, c: char, options: Options) -> Result<(), Options> {
        self.expect(c, options)?;

        self.pos += 1;
        self.expr.next();
        Ok(())
    }

    pub fn accept_string(&mut self, text: &str, options: Options) -> Result<(), Options> {
        let backup = self.backup();
        for c in text.chars() {
            if let Err(e) = self.accept(c, options.clone()) {
                self.restore(backup);
                return Err(e);
            }
        }

        Ok(())
    }

    pub fn expect(&mut self, c: char, options: Options) -> Result<(), Options> {
        while let Some(i) = self.expr.peek() {
            if !i.is_whitespace() {
                break;
            }
            self.pos += 1;
            self.expr.next();
        }

        let pk = self.expr.peek();
        if pk == Some(&c) {
            Ok(())
        } else {
            Err(options.add(c).pos(self.pos))
        }
    }

    pub fn accept_any(
        &mut self,
        c: &[char],
        mut options: Options,
        name: Option<Options>,
    ) -> Result<char, Options> {
        for i in c {
            match self.accept(*i, options.clone()) {
                Ok(_) => return Ok(*i),
                Err(o) => {
                    if name.is_none() {
                        options = options.merge(o)
                    }
                }
            }
        }

        if let Some(n) = name {
            options = options.merge(n)
        }

        Err(options)
    }

    pub fn parse(&mut self) -> Result<Ast, Options> {
        let result = self.parse_expr(Options::new(self.source.clone()))?;

        if self.expr.next().is_some() {
            return Err(Options::new(self.source.clone())
                .pos(self.pos)
                .message("unexpected trailing character(s)"));
        }

        Ok(result)
    }

    pub fn parse_expr(&mut self, options: Options) -> Result<Ast, Options> {
        self.parse_sum(options)
    }

    pub fn parse_sum(&mut self, options: Options) -> Result<Ast, Options> {
        let mut res = self.parse_term(options.clone())?;

        while let Ok(op) = self.accept_any(&['+', '-'], options.clone(), None) {
            let right = self.parse_term(options.clone())?;

            res = match op {
                '+' => Ast::Add(Box::new(res), Box::new(right)),
                '-' => Ast::Sub(Box::new(res), Box::new(right)),
                _ => unreachable!(),
            }
        }

        Ok(res)
    }

    pub fn parse_term(&mut self, mut options: Options) -> Result<Ast, Options> {
        let mut res = self.parse_factor(options.clone())?;

        loop {
            let opres = self.accept_any(&['*', '/'], options.clone(), None);
            let mut op = if let Ok(i) = opres {
                i
            } else if self.accept_string("mod", options.clone()).is_ok() {
                '%'
            } else {
                //FIXME options assigned but never read
                options = options.add_str("mod");
                options = options.add_str("//");
                break;
            };

            if op == '/' && self.accept('/', options.clone()).is_ok() {
                op = 'i'
            } else {
                options = options.add('/');
            }

            let right = self.parse_factor(options.clone())?;

            res = match op {
                '*' => Ast::Mul(Box::new(res), Box::new(right)),
                '/' => Ast::Div(Box::new(res), Box::new(right)),
                'i' => Ast::IDiv(Box::new(res), Box::new(right)),
                '%' => Ast::Mod(Box::new(res), Box::new(right)),
                _ => unreachable!(),
            }
        }

        Ok(res)
    }

    pub fn parse_factor(&mut self, options: Options) -> Result<Ast, Options> {
        let backup = self.backup();

        Ok(match self.accept('-', options.clone()) {
            Ok(_) => Ast::Minus(Box::new(self.parse_power(options)?)),
            Err(o) => {
                self.restore(backup);

                return self.parse_power(o);
            }
        })
    }

    pub fn parse_power(&mut self, options: Options) -> Result<Ast, Options> {
        let mut res = self.parse_atom(options.clone())?;
        if self.accept_string("**", options.clone()).is_ok() {
            let right = self.parse_factor(options)?;
            res = Ast::Power(Box::new(res), Box::new(right));
        }

        Ok(res)
    }

    pub fn parse_atom(&mut self, options: Options) -> Result<Ast, Options> {
        let backup = self.backup();
        Ok(match self.parse_dice(options) {
            Err(mut o) => {
                self.restore(backup);

                let backup = self.backup();
                if self.accept('(', o.clone()).is_ok() {
                    let sm = self.parse_sum(o.clone())?;
                    self.accept(')', o)
                        .map_err(|e| e.message("missing closing parenthesis"))?;

                    return Ok(sm);
                } else {
                    o = o
                        .add('(')
                        .message("tried to parse expression between parenthesis");
                    self.restore(backup);
                }

                self.parse_number(o.message("tried to parse dice roll"))?
            }
            Ok(i) => i,
        })
    }

    pub fn parse_dice(&mut self, mut options: Options) -> Result<Ast, Options> {
        let backup = self.backup();

        let rolls = if self.advanced && self.accept('(', options.clone()).is_ok() {
            let sm = self.parse_sum(options.clone())?;
            self.accept(')', options.clone())
                .map_err(|e| e.message("missing closing parenthesis"))?;

            Some(Box::new(sm))
        } else {
            if self.advanced {
                options = options
                    .add('(')
                    .message("tried to parse expression between parenthesis");
            }
            self.restore(backup);
            self.parse_number(options.clone()).map(Box::new).ok()
        };

        self.accept('d', options.clone())?;
        let dpos = self.pos - 1;

        let backup = self.backup();
        let sides = if self.advanced && self.accept('(', options.clone()).is_ok() {
            let sm = self.parse_sum(options.clone())?;
            self.accept(')', options.clone())
                .map_err(|e| e.message("missing closing parenthesis"))?;

            Some(Box::new(sm))
        } else {
            if self.advanced {
                options = options
                    .add('(')
                    .message("tried to parse expression between parenthesis");
            }
            self.restore(backup);

            self.parse_number_or_percent(options.clone())
                .map(Box::new)
                .ok()
        };

        let fm = if self.accept_string("kh", options.clone()).is_ok()
            || self.accept('h', options.clone()).is_ok()
        {
            FilterModifier::KeepHighest(Box::new(
                self.parse_number(options)
                    .unwrap_or_else(|_| Ast::Const("1".to_string())),
            ))
        } else if self.accept_string("dl", options.clone()).is_ok()
            || self.accept('l', options.clone()).is_ok()
        {
            FilterModifier::DropLowest(Box::new(
                self.parse_number(options)
                    .unwrap_or_else(|_| Ast::Const("1".to_string())),
            ))
        } else if self.accept_string("dh", options.clone()).is_ok() {
            FilterModifier::DropHighest(Box::new(
                self.parse_number(options)
                    .unwrap_or_else(|_| Ast::Const("1".to_string())),
            ))
        } else if self.accept_string("kl", options.clone()).is_ok() {
            FilterModifier::KeepLowest(Box::new(
                self.parse_number(options)
                    .unwrap_or_else(|_| Ast::Const("1".to_string())),
            ))
        } else {
            FilterModifier::None
        };

        Ok(Ast::Dice(rolls, sides, fm, dpos))
    }

    pub fn parse_number_or_percent(&mut self, options: Options) -> Result<Ast, Options> {
        if self.accept('%', options.clone()).is_ok() {
            Ok(Ast::Const("100".to_ascii_lowercase()))
        } else {
            self.parse_number(options.add('%'))
        }
    }

    pub fn parse_number(&mut self, options: Options) -> Result<Ast, Options> {
        const DIGITS: &[char] = &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.'];
        let digits_name = Options::new("".to_string()).add_str("0-9");

        let mut number = vec![self
            .accept_any(&DIGITS, options.clone(), Some(digits_name.clone()))
            .map_err(|e| {
                options
                    .clone()
                    .merge(e)
                    .add('(')
                    .message("tried to parse a number")
            })?];

        loop {
            let backup = self.backup();

            let digit = match self.accept_any(&DIGITS, options.clone(), Some(digits_name.clone())) {
                Ok(i) => i,
                Err(_) => {
                    self.restore(backup);
                    break;
                }
            };

            number.push(digit)
        }

        let string: String = number.iter().collect();

        Ok(Ast::Const(string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filtermodifier::FilterModifier;
    use crate::interpreter::{Ast, Value, DEFAULT_SIDES};

    #[test]
    pub fn add() {
        let mut p = Parser::new("3 + 5");
        assert_eq!(
            p.parse().unwrap().interp(&mut Vec::new()).unwrap(),
            Value::Int(8)
        );
    }

    #[test]
    pub fn sub() {
        let mut p = Parser::new("3 - 5");
        assert_eq!(
            p.parse().unwrap().interp(&mut Vec::new()).unwrap(),
            Value::Int(-2)
        );
    }

    #[test]
    pub fn mul() {
        let mut p = Parser::new("3 * 5");
        assert_eq!(
            p.parse().unwrap().interp(&mut Vec::new()).unwrap(),
            Value::Int(15)
        );
    }

    #[test]
    pub fn div() {
        let mut p = Parser::new("15/3");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Float(5.0));
    }

    #[test]
    pub fn pemdas() {
        let mut p = Parser::new("3 * 5 + -5");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Int(10));
    }

    #[test]
    pub fn parens() {
        let mut p = Parser::new("3 * (5 + -5)");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Int(0))
    }

    #[test]
    pub fn meme() {
        let mut p = Parser::new("6 / 2 * (1 + 2)");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Float(9.0))
    }

    #[test]
    pub fn long_sum() {
        let mut p = Parser::new("3 + 3 + 3");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Int(9));
    }

    #[test]
    pub fn long_mul() {
        let mut p = Parser::new("3 * 3 * 3");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Int(27));
    }

    #[test]
    pub fn idiv() {
        let mut p = Parser::new("3 // 5");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Int(0));
    }

    #[test]
    pub fn dice_none() {
        let mut p = Parser::new("d");
        let ast = p.parse().unwrap();
        assert_eq!(ast, Ast::Dice(None, None, FilterModifier::None, 0));

        let mut rolls = Vec::new();
        let res = ast.interp(&mut rolls).unwrap();

        assert_eq!(rolls.len(), 1);

        let roll = &rolls[0].1;
        assert_eq!(DEFAULT_SIDES, roll.sides.to_string());

        assert_eq!(res, Value::Int(roll.total));
    }

    #[test]
    pub fn dice_6() {
        let mut p = Parser::new("d6");
        let ast = p.parse().unwrap();
        assert_eq!(
            ast,
            Ast::Dice(
                None,
                Some(Box::new(Ast::Const("6".to_string()))),
                FilterModifier::None,
                0
            )
        );

        let mut rolls = Vec::new();
        let res = ast.interp(&mut rolls).unwrap();

        assert_eq!(rolls.len(), 1);

        let roll = &rolls[0].1;

        assert_eq!(res, Value::Int(roll.total as i64));
    }

    #[test]
    pub fn dice_3_6() {
        let mut p = Parser::new("3d6");
        let ast = p.parse().unwrap();
        assert_eq!(
            ast,
            Ast::Dice(
                Some(Box::new(Ast::Const("3".to_string()))),
                Some(Box::new(Ast::Const("6".to_string()))),
                FilterModifier::None,
                1
            )
        );

        let mut rolls = Vec::new();
        let res = ast.interp(&mut rolls).unwrap();

        assert_eq!(rolls.len(), 1);

        let roll0 = &rolls[0].1.vals[0];
        let roll1 = &rolls[0].1.vals[1];
        let roll2 = &rolls[0].1.vals[2];

        assert_eq!(res, Value::Int((roll0 + roll1 + roll2) as i64));
    }

    #[test]
    pub fn dice_0() {
        let mut p = Parser::new("0d6");
        let ast = p.parse().unwrap();
        assert_eq!(
            ast,
            Ast::Dice(
                Some(Box::new(Ast::Const("0".to_string()))),
                Some(Box::new(Ast::Const("6".to_string()))),
                FilterModifier::None,
                1
            )
        );

        let mut rolls = Vec::new();
        let res = ast.interp(&mut rolls).unwrap();

        assert_eq!(rolls.len(), 1);

        assert_eq!(res, Value::Int(0));
    }

    #[test]
    pub fn dice_float() {
        let mut p = Parser::new("3.5d6");
        let ast = p.parse().unwrap();
        assert_eq!(
            ast,
            Ast::Dice(
                Some(Box::new(Ast::Const("3.5".to_string()))),
                Some(Box::new(Ast::Const("6".to_string()))),
                FilterModifier::None,
                3
            )
        );

        ast.interp(&mut Vec::new()).expect_err("result was okay");
    }

    #[test]
    pub fn dice_dfloat() {
        let mut p = Parser::new("3d3.5");
        let ast = p.parse().unwrap();
        assert_eq!(
            ast,
            Ast::Dice(
                Some(Box::new(Ast::Const("3".to_string()))),
                Some(Box::new(Ast::Const("3.5".to_string()))),
                FilterModifier::None,
                1
            )
        );

        ast.interp(&mut Vec::new()).expect_err("result was okay");
    }

    #[test]
    pub fn dice_d0() {
        let mut p = Parser::new("3d0");
        let ast = p.parse().unwrap();
        assert_eq!(
            ast,
            Ast::Dice(
                Some(Box::new(Ast::Const("3".to_string()))),
                Some(Box::new(Ast::Const("0".to_string()))),
                FilterModifier::None,
                1
            )
        );

        ast.interp(&mut Vec::new()).expect_err("result was okay");
    }

    #[test]
    pub fn dice_dpercent() {
        let mut p = Parser::new("d%");
        let ast = p.parse().unwrap();
        assert_eq!(
            ast,
            Ast::Dice(
                None,
                Some(Box::new(Ast::Const("100".to_string()))),
                FilterModifier::None,
                0
            )
        );

        let mut rolls = Vec::new();
        let res = ast.interp(&mut rolls).unwrap();

        assert_eq!(rolls.len(), 1);

        let roll = &rolls[0].1;

        assert_eq!(roll.sides.get(), 100);
        assert_eq!(res, Value::Int(roll.total));
    }

    #[test]
    pub fn dice_kl() {
        let mut p = Parser::new("5d%kl");
        let ast = p.parse().unwrap();

        let mut rolls = Vec::new();
        let res = ast.interp(&mut rolls).unwrap();

        assert_eq!(rolls.len(), 1);

        let roll = &rolls[0].1;

        assert_eq!(roll.vals.len(), 1);
        assert_eq!(res, Value::Int(roll.total));
    }

    #[test]
    pub fn pow() {
        let mut p = Parser::new("5 ** 2");
        let ast = p.parse().unwrap();
        assert_eq!(ast.interp(&mut Vec::new()).unwrap(), Value::Int(25));
    }

    #[test]
    pub fn compound() {
        let mut p = Parser::new("(3d5)d(5d3)");
        p.advanced = true;
        let _ = p.parse().unwrap();
    }
}
