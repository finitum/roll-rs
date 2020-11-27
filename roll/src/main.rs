use roll_lib::rand_core::OsRng;
use roll_lib::{inplace_interp, roll_direction, Parser, Roll};
use std::{env, process};

fn main() {
    let mut argv: Vec<String> = env::args().skip(1).collect();
    let args = argv.join(" ");

    match args.as_str() {
        "stats" => roll_stats(),
        "dir" => roll_dir(),
        "-h" | "--help" | "" => {
            print_usage();
        }
        _ => {
            let mut advanced = false;
            let mut short = false;

            argv.retain(|x| match x.as_str() {
                "-a" => {
                    advanced = true;
                    false
                }
                "-s" => {
                    short = true;
                    false
                }
                _ => true,
            });

            if short {
                println!("{}", inplace_interp(&argv.join(" "), advanced));
            } else {
                dice_roller(&argv.join(" "), advanced);
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
    process::exit(0);
}

fn roll_dir() {
    let dir = roll_direction(OsRng);
    println!("{}", dir);
}

fn dice_roller(s: &str, advanced: bool) {
    let mut p = Parser::new(s);
    if advanced {
        p = p.advanced()
    }

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
        header.push_str(&format!("d{}", roll.sides))
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

            rows[index].push_str(&format!("{}", val))
        }
    }

    for row in rows {
        println!("{}", row);
    }
}

const STAT_ROLL: &str = "4d6l";
fn roll_stats() {
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
        println!("{:2}: {:?}", roll.total, roll.vals)
    }
}
