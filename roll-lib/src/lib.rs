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

    fn grammar() -> bnf::Grammar {
        include_str!("../../grammar.bnf").parse().unwrap()
    }

    fn generate_sentence() -> String {
        loop {
            let res = grammar().generate();
            match res {
                Ok(i) => break i,
                Err(bnf::Error::RecursionLimit(_)) => continue,
                _ => panic!("aaaaa")
            }
        }
    }

    #[test]
    fn fuzz() {
        for _ in 0..500 {
            let sentence = generate_sentence();
            if let Err(e) =  Parser::new(&sentence).advanced().parse() {
                println!("failed with sentence \"{}\" and error: {:?}", sentence, e);
                break;
            }
        }
    }
}
