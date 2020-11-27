mod filtermodifier;
mod interpreter;
mod options;
mod parser;
mod roll;

use crate::interpreter::Ast;
pub use crate::parser::*;
pub use crate::roll::*;
pub use rand_core;
use std::collections::HashMap;

const STAT_ROLL: &str = "4d6l";
pub fn roll_stats() -> String {
    let mut res = String::new();
    fn roll_stat() -> Roll {
        let mut rolls = Vec::new();
        Parser::new(STAT_ROLL)
            .parse()
            .unwrap()
            .interp(&mut rolls)
            .unwrap();
        rolls.remove(0).1
    }

    for _ in 0..6 {
        let roll = roll_stat();
        res.push_str(&format!("{:2}: {:?}\n", roll.total, roll.vals))
    }
    res
}

pub fn roll_inline(s: &str, advanced: bool) -> Result<String, String> {
    let mut p = Parser::new(s);
    p.advanced = advanced;

    let ast = p.parse().map_err(|e| e.to_string())?;

    let copy = ast.clone();

    let mut rolls = Vec::new();
    let total = ast.interp(&mut rolls).unwrap();

    let mut map = HashMap::new();
    for (pos, roll) in rolls {
        map.insert(pos, roll);
    }

    let res = replace_rolls(copy, &map, |roll| format!("{:?}", roll.vals));
    Ok(format!("{} = {} = {}", s, res, total))
}

fn replace_rolls(ast: Ast, lookup: &HashMap<u64, Roll>, func: fn(&Roll) -> String) -> Ast {
    return match ast {
        Ast::Add(l, r) => Ast::Add(
            Box::from(replace_rolls(*l, lookup, func)),
            Box::from(replace_rolls(*r, lookup, func)),
        ),
        Ast::Sub(l, r) => Ast::Sub(
            Box::from(replace_rolls(*l, lookup, func)),
            Box::from(replace_rolls(*r, lookup, func)),
        ),
        Ast::Mul(l, r) => Ast::Mul(
            Box::from(replace_rolls(*l, lookup, func)),
            Box::from(replace_rolls(*r, lookup, func)),
        ),
        Ast::Div(l, r) => Ast::Div(
            Box::from(replace_rolls(*l, lookup, func)),
            Box::from(replace_rolls(*r, lookup, func)),
        ),
        Ast::Mod(l, r) => Ast::Mod(
            Box::from(replace_rolls(*l, lookup, func)),
            Box::from(replace_rolls(*r, lookup, func)),
        ),
        Ast::IDiv(l, r) => Ast::IDiv(
            Box::from(replace_rolls(*l, lookup, func)),
            Box::from(replace_rolls(*r, lookup, func)),
        ),
        Ast::Power(l, r) => Ast::Power(
            Box::from(replace_rolls(*l, lookup, func)),
            Box::from(replace_rolls(*r, lookup, func)),
        ),
        Ast::Minus(l) => Ast::Minus(Box::from(replace_rolls(*l, lookup, func))),
        Ast::Dice(_, _, _, pos) => {
            // Safety: we exhaustively add all positions to this hashmap so it must contain everything
            // we look up.
            let roll = lookup.get(&pos).unwrap();
            Ast::Const(func(roll))
        }
        x @ Ast::Const(_) => x,
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Parser;
    use bnf::Grammar;

    const GRAMMAR: &str = include_str!("../../grammar.bnf");

    fn generate_sentence(g: &Grammar) -> String {
        loop {
            let res = g.generate();
            match res {
                Ok(i) => break i,
                Err(bnf::Error::RecursionLimit(_)) => continue,
                _ => panic!("aaaaa"),
            }
        }
    }

    #[test]
    fn fuzz() {
        let grammar: Grammar = GRAMMAR.parse().unwrap();

        for _ in 0..500 {
            let sentence = generate_sentence(&grammar);
            if let Err(e) = Parser::new(&sentence).advanced().parse() {
                println!("failed with sentence \"{}\" and error: {:?}", sentence, e);
                break;
            }
        }
    }

    #[test]
    fn test_inplace() {
        println!("{}", roll_inline("4d8 + 2d8", false).unwrap());
    }
}
