mod filtermodifier;
mod interpreter;
mod options;
mod parser;
mod roll;

use crate::parser::Parser;
use std::{env, process};

fn main() {
    let arg: String = env::args().skip(1).collect();

    match arg.as_str() {
        "stats" => {}
        "dir" => {}
        "-h" | "--help" | "" => {
            print_usage(0);
        }
        _ => {
            dice_roller(&arg);
        }
    }
}

fn print_usage(code: i32) -> ! {
    println!("Syntax is: roll <dice_code>\nExample: roll 2d8 + 6 + d8");
    println!("Instead of a dice code you can also put \"stats\" or \"dir\" for a stats roll or direction roll respectively");
    process::exit(code)
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
                rows[index].push(' ')
            }

            rows[index].push_str(&format!("{}", val))
        }
    }

    for row in rows {
        println!("{}", row);
    }
}
