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

fn open_ended_roll(rng: &mut rand::ThreadRng) -> Roll {
    let dice = Range::new(1, 100);
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

fn fumbelable_roll(rng: &mut rand::ThreadRng) -> FumbelableRoll {
    let roll = open_ended_roll(rng);
    if roll.value <= 5 {
        return FumbelableRoll::Fumble(open_ended_roll(rng))
    }
    FumbelableRoll::Roll(roll)
}

enum CommandError {
    ParseError,
    NotEnoughArguments,
    NotRecognizedArgument(String)
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
        "o" => {
            let your_bonus = parts.next()?.parse::<i32>()?;
            let enemy_bonus = parts.next()?.parse::<i32>()?;
            let distance = parts.next().unwrap_or("0").parse::<i32>()?;
            let excla = |n| "!".repeat(n as usize);
            match fumbelable_roll(rng) {
                FumbelableRoll::Fumble(roll) => {
                    println!("[Fumble {}{}]", roll.value, excla(roll.ncrit));
                },
                FumbelableRoll::Roll(roll) => {
                    let modifier = your_bonus - enemy_bonus - distance;
                    let result = roll.value + modifier;
                    println!("{} + 1d100@{} = [{}{}]", modifier, roll.value, result, excla(roll.ncrit));
                }
            }
        }
        _ => return Err(CommandError::NotRecognizedArgument(command.to_string()))
    }
    Ok(this)
}

fn app() -> Result<(), std::io::Error> {
    let stdin = io::stdin();
    let mut rng = rand::thread_rng();
    let mut last = "o 0 0".to_string();
    for line in stdin.lock().lines() {
        match command(line?, &last, &mut rng) {
            Ok(l) => last = l,
            Err(_) => println!("o <your bonus> <enemy bonus> <range?>"),
        }
    }
    Ok(())
}

fn main() {
    app().unwrap();
}
