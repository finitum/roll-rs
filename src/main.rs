mod filtermodifier;
mod interpreter;
mod options;
mod parser;
mod roll;

use crate::parser::Parser;
use std::{env, process};
use rand_core::OsRng;
use crate::roll::roll_die;
use crate::filtermodifier::FilterModifier;
use std::num::NonZeroU64;

fn main() {
    let argv: Vec<String> = env::args().skip(1).into_iter().collect();
    let args = argv.join(" ");

    match args.as_str() {
        "stats" => roll_stats(),
        "dir" => roll_dir(),
        "-h" | "--help" | "" => {
            print_usage(0);
        }
        _ => {
            dice_roller(&args);
        }
    }
}

fn print_usage(code: i32) -> ! {
    println!("Syntax is: roll <dice_code>\nExample: roll 2d8 + 6 + d8");
    println!("Instead of a dice code you can also put \"stats\" or \"dir\" for a stats roll or direction roll respectively");
    process::exit(code)
}

const STAT_ROLL: &str = "4d6l";

fn roll_stats() {
    fn roll_stat() -> roll::Roll {
        let mut rolls = Vec::new();
        Parser::new(STAT_ROLL).parse().unwrap().interp(&mut rolls).unwrap();
        rolls.remove(0).1
    }

    for _ in 0..6 {
        let roll = roll_stat();
        println!("{:2}: {:?}", roll.total, roll.vals)
    }
}

const DIR: &[&str] = &["North","North East","East","South East","South","South West","West","North West","Stay"];

fn roll_dir() {
    let value = roll_die(1, NonZeroU64::new(DIR.len() as u64).unwrap(), FilterModifier::None, OsRng);
    println!("{}", DIR[value.total as usize - 1])
}

fn dice_roller(s: &str) {
    let mut p = Parser::new(s);

    let ast = match p.parse()  {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1)
        }
    };

    let mut rolls = Vec::new();
    let total = match ast.interp(&mut rolls) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(2)
        }
    };

    println!("{} = {}", s, total);

    let mut rows = Vec::new();

    for (x, roll) in rolls {
        while roll.vals.len() > rows.len() {
            rows.push(String::new());
        }

        for (index, val) in roll.vals.iter().enumerate() {
            for _ in rows[index].len()..(x as usize) {
                rows[index].push(' ');
            }

            rows[index].push_str(&format!("{}", val))
        }
    }

    for row in rows {
        println!("{}", row);
    }
}
