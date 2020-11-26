mod filtermodifier;
mod interpreter;
mod options;
mod parser;
mod roll;

pub use crate::parser::*;
pub use crate::roll::*;
pub use rand_core;

#[cfg(test)]
mod test {
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
}
