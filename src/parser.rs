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
}

impl<'a> Parser<'a> {
    pub fn new(expr: &'a str) -> Self {
        Self {
            source: expr.clone().to_string(),
            expr: expr.chars().peekable(),
            pos: 0,
        }
    }

    pub fn backup(&self) -> Self {
        Self {
            expr: self.expr.clone(),
            pos: self.pos,
            source: self.source.clone(),
        }
    }

    pub fn restore(&mut self, other: Self) {
        self.expr = other.expr;
        self.pos = other.pos;
        self.source = other.source;
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
        loop {
            if let Some(i) = self.expr.peek() {
                if !i.is_whitespace() {
                    break;
                } else {
                    self.pos += 1;
                    self.expr.next();
                }
            } else {
                break;
            }
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
        c: &Vec<char>,
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
            return Err(Options::new(self.source.clone()).pos(self.pos).message("unexpected trailing character(s)"))
        }

        Ok(result)
    }

    pub fn parse_expr(&mut self, options: Options) -> Result<Ast, Options> {
        self.parse_sum(options)
    }

    pub fn parse_sum(&mut self, options: Options) -> Result<Ast, Options> {
        let left = self.parse_term(options.clone())?;

        let mut res = left;

        loop {
            let op = if let Ok(i) = self.accept_any(&vec!['+', '-'], options.clone(), None) {
                i
            } else {
                break;
            };
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
        let left = self.parse_factor(options.clone())?;
        let mut res = left;

        loop {
            let mut op = if let Ok(i) = self.accept_any(&vec!['*', '/', '%'], options.clone(), None)
            {
                i
            } else {
                break;
            };

            if self.accept('/', options.clone()).is_ok() {
                op = 'i';
            }
            options = options.add('/');

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
            Ok(_) => Ast::Minus(Box::new(self.parse_atom(options.clone())?)),
            Err(o) => {
                self.restore(backup);

                return self.parse_atom(o);
            }
        })
    }

    pub fn parse_atom(&mut self, mut options: Options) -> Result<Ast, Options> {
        let backup = self.backup();
        if self.accept('(', options.clone()).is_ok() {
            let sm = self.parse_sum(options.clone())?;
            self.accept(')', options.clone())
                .map_err(|e| e.message("missing closing parenthesis"))?;

            return Ok(sm);
        } else {
            options = options
                .add('(')
                .message("tried to parse expression between parenthesis");
            self.restore(backup);
        }

        let backup = self.backup();
        Ok(match self.parse_dice(options.clone()) {
            Err(o) => {
                self.restore(backup);
                self.parse_number(o.message("tried to parse dice roll"))?
            }
            Ok(i) => i,
        })
    }

    pub fn parse_dice(&mut self, options: Options) -> Result<Ast, Options> {
        let rolls = self.parse_number(options.clone()).map(Box::new).ok();

        let dpos = self.pos;
        self.accept('d', options.clone())?;

        let sides = self
            .parse_number_or_percent(options.clone())
            .map(Box::new)
            .ok();

        let fm = if self.accept_string("kh", options.clone()).is_ok()
            || self.accept('h', options.clone()).is_ok()
        {
            FilterModifier::KeepHighest(Box::new(
                self.parse_number(options)
                    .unwrap_or(Ast::Const("1".to_string())),
            ))
        } else if self.accept_string("dl", options.clone()).is_ok()
            || self.accept('l', options.clone()).is_ok()
        {
            FilterModifier::DropLowest(Box::new(
                self.parse_number(options)
                    .unwrap_or(Ast::Const("1".to_string())),
            ))
        } else if self.accept_string("dh", options.clone()).is_ok() {
            FilterModifier::DropHighest(Box::new(
                self.parse_number(options)
                    .unwrap_or(Ast::Const("1".to_string())),
            ))
        } else if self.accept_string("kl", options.clone()).is_ok() {
            FilterModifier::KeepLowest(Box::new(
                self.parse_number(options)
                    .unwrap_or(Ast::Const("1".to_string())),
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
        let digits = vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.'];
        let digits_name = Options::new("".to_string()).add_str("0-9");

        let mut number = vec![self
            .accept_any(&digits, options.clone(), Some(digits_name.clone()))
            .map_err(|e| {
                options
                    .clone()
                    .merge(e)
                    .add('(')
                    .message("tried to parse a number")
            })?];

        loop {
            let backup = self.backup();

            let digit = match self.accept_any(&digits, options.clone(), Some(digits_name.clone())) {
                Ok(i) => i,
                Err(_) => {
                    self.restore(backup);
                    break;
                }
            };

            number.push(digit)
        }

        let string: String = number.iter().collect();

        return Ok(Ast::Const(string));
    }
}

#[cfg(test)]
mod tests {
    use crate::filtermodifier::FilterModifier;
    use crate::interpreter::{Ast, Value, DEFAULT_SIDES};
    use crate::parser::Parser;

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

        let roll0 = &rolls[0].1;
        let roll1 = &rolls[0].1;
        let roll2 = &rolls[0].1;

        assert_eq!(
            res,
            Value::Int((roll0.total + roll1.total + roll2.total) as i64)
        );
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

        assert_eq!(rolls.len(), 0);

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

        assert_eq!(roll.vals.len(), 100);
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
}
