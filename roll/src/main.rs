use rand_core::OsRng;
use roll_rs::{roll_direction, roll_inline, roll_stats, Parser};
use std::{env, process};

fn main() {
    if env::args().len() <= 1 {
        print_usage()
    }

    let mut argv: Vec<String> = env::args().skip(1).collect();
    match argv[0].as_str() {
        "stats" => print_roll_stats(),
        "dir" => print_roll_dir(),
        "-h" | "--help" | "" => print_usage(),
        _ => {
            let mut advanced = false;
            let mut short = false;

            argv.retain(|x| match x.as_str() {
                "-a" | "--advanced" => {
                    advanced = true;
                    false
                }
                "-s" | "--short" => {
                    short = true;
                    false
                }
                _ => true,
            });

            if short {
                roll_short(&argv.join(" "), advanced);
            } else {
                roll_long(&argv.join(" "), advanced);
            }
        }
    }
}

fn print_usage() -> ! {
    println!("Syntax is: roll <dice_code>\nExample: roll 2d8 + 6 + d8");
    println!("Instead of a dice code you can also put \"stats\" or \"dir\" for a stats roll or direction roll respectively");
    println!("\nArgs: ");
    println!("  -a: advanced mode (composite dice notation)");
    println!("  -s: smaller output");
    process::exit(0)
}

fn print_roll_dir() {
    let dir = roll_direction(OsRng);
    println!("{}", dir);
}

fn roll_short(s: &str, advanced: bool) {
    match roll_inline(s, advanced) {
        Ok(s) => println!("{}", s),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(2);
        }
    }
}

fn roll_long(s: &str, advanced: bool) {
    let mut p = Parser::new(s);
    p.advanced = advanced;

    let ast = match p.parse() {
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

    rolls.sort_by_key(|i| i.0);

    let mut header = String::new();
    for (x, roll) in &rolls {
        for _ in header.len()..(*x as usize) {
            header.push(' ');
        }
        header.push_str(&format!("d{}", roll.sides));
    }

    println!("{}", header);
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

            rows[index].push_str(&format!("{}", val));
        }
    }

    for row in rows {
        println!("{}", row);
    }
}

fn print_roll_stats() {
    print!("{}", roll_stats());
}
