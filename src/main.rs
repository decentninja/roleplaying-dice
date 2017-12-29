#![feature(try_trait)]
use std::io;
use std::io::prelude::*;
extern crate rand;
use rand::distributions::{IndependentSample, Range};
use std::convert::From;

struct Roll {
    value: i32,
    ncrit: u32
}

enum FumbelableRoll {
    Fumble(Roll),
    Roll(Roll),
}

fn d100_open(rng: &mut rand::ThreadRng) -> Roll {
    let dice = Range::new(1, 101);
    let mut roll = dice.ind_sample(rng);
    let mut total = roll;
    let mut ncrit = 0;
    while roll >= 95 {
        roll = dice.ind_sample(rng);
        total += roll;
        ncrit += 1;
    }
    Roll {
        value: total,
        ncrit: ncrit
    }
}

fn d100_open_fumbelable(rng: &mut rand::ThreadRng) -> FumbelableRoll {
    let roll = d100_open(rng);
    if roll.value <= 5 {
        return FumbelableRoll::Fumble(d100_open(rng))
    }
    FumbelableRoll::Roll(roll)
}

fn d20_fumbelable(rng: &mut rand::ThreadRng) -> FumbelableRoll {
    let dice = Range::new(1, 20);
    let roll = dice.ind_sample(rng);
    if roll == 1 {
        return FumbelableRoll::Fumble(Roll {
            value: 1,
            ncrit: 0
        })
    }
    return FumbelableRoll::Roll(Roll {
        value: roll,
        ncrit: if roll == 20 { 1 } else { 0 }
    })
}

enum CommandError {
    ParseError,
    NotEnoughArguments,
    NotRecognizedCommand,
}

impl From<std::option::NoneError> for CommandError {
    fn from(_: std::option::NoneError) -> Self {
        CommandError::NotEnoughArguments
    }
}

impl From<std::num::ParseIntError> for CommandError {
    fn from(_: std::num::ParseIntError) -> Self {
        CommandError::ParseError
    }
}

fn command(line: String, last: &str, rng: &mut rand::ThreadRng) -> Result<String, CommandError> {
    let mut this = line.to_string();
    let mut parts = line.split_whitespace();
    let command = match parts.next() {
        Some(s) => s,
        None => {
            this = last.to_string();
            parts = last.split_whitespace();
            parts.next()?
        }
    };
    match command {
        // TODO: Add delay and suspence in printout
        "o100" => {
            let your_bonus = parts.next()?.parse::<i32>()?;
            let enemy_bonus = parts.next()?.parse::<i32>()?;
            let distance = parts.next().unwrap_or("0").parse::<i32>()?;
            let excla = |n| "!".repeat(n as usize);
            let dodge = d100_open(rng);
            match d100_open_fumbelable(rng) {
                FumbelableRoll::Fumble(roll) => {
                    println!("[Fumble {}{}]", roll.value, excla(roll.ncrit));
                },
                FumbelableRoll::Roll(roll) => {
                    let modifier = your_bonus - enemy_bonus - distance;
                    let result = roll.value + modifier - dodge.value;
                    println!("Modifier:    {}{}", if modifier > 0 {"+"} else {""}  , modifier);
                    println!("Attack d100: +{}{}", roll.value, excla(roll.ncrit));
                    println!("Dodge d100 : -{}{}", dodge.value, excla(dodge.ncrit));
                    println!("Result     : [{}]", result);
                }
            }
        }
        "d20" => {
            let dc = parts.next()?.parse::<i32>()?;
            let your_bonus = parts.next().unwrap_or("0").parse::<i32>()?;
            match d20_fumbelable(rng) {
                FumbelableRoll::Fumble(_) => {
                    println!("[1d20 = [1 Fumble] MEGA FAIL");
                },
                FumbelableRoll::Roll(roll) => {
                    let result = roll.value + your_bonus;
                    if roll.ncrit == 1 {
                        println!("1d20@20 = [20!] SUPER SUCCESS");
                    } else if result >= dc {
                        println!("{} + 1d20@{} = [{}] >= {} SUCCESS", your_bonus, roll.value, result, dc);
                    } else {
                        println!("{} + 1d20@{} = [{}] >!= {} FAIL", your_bonus, roll.value, result, dc);
                    }
                }

            }
        }
        _ => return Err(CommandError::NotRecognizedCommand)
    }
    Ok(this)
}

fn app() -> Result<(), std::io::Error> {
    let stdin = io::stdin();
    let mut rng = rand::thread_rng();
    let mut last = "o100 0 0".to_string();
    for line in stdin.lock().lines() {
        println!("");
        let instructions = "
Instruction:
o100 <your bonus> <enemy bonus> <?range>o
d20 <dc> <?your bonus>
        ";
        match command(line?, &last, &mut rng) {
            Ok(l) => last = l,
            Err(e) => {
                match e {
                    CommandError::ParseError => println!("Could not parse arguments"),
                    CommandError::NotEnoughArguments => println!("Not enought arguments"),
                    CommandError::NotRecognizedCommand => println!("Not recognized command")
                }
                println!("{}", instructions);
            }
        }
    }
    Ok(())
}

fn main() {
    app().unwrap();
}
