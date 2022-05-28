use std::env;
use std::io::{self, Write, Read};
use std::net::*;
use std::thread;
use std::time;
use std::collections::HashMap;

enum ConnectionMode {
    Hosting,
    Joining
}

enum PlayerChoice {
    Rock,
    Paper,
    Scissors
}

struct Arguments {
    mode: ConnectionMode,
    ip: String,
    choice: PlayerChoice
}

impl Arguments {
    fn parse(args: &[String]) -> Result<Self, ()> {
        if args.len() < 3 { return Err(()); }
        
        enum ParseState {
            Mode,
            IP,
            Choice,
            Done
        }

        let mut state = ParseState::Mode;
        let mut output = Arguments {
            mode: ConnectionMode::Hosting,
            ip: "".to_string(),
            choice: PlayerChoice::Rock
        };

        for arg in args.iter().skip(1) {
            match state {
                ParseState::Mode => {
                    match arg.as_str() {
                        "host" => { output.mode = ConnectionMode::Hosting; state = ParseState::Choice; },
                        "join" => { output.mode = ConnectionMode::Joining; state = ParseState::IP; }
                        _ => { return Err(()); }
                    }
                },
                ParseState::IP => {
                    output.ip = (*arg).to_string();
                    state = ParseState::Choice;
                },
                ParseState::Choice => {
                    match arg.as_str() {
                        "rock" | "paper" | "scissors" => {
                            output.choice = match arg.as_str() {
                                "rock" => { PlayerChoice::Rock },
                                "paper" => { PlayerChoice::Paper },
                                "scissors" => { PlayerChoice::Scissors },
                                _ => unreachable!()
                            };
                            state = ParseState::Done;
                        },
                        _ => { return Err(()); }
                    }
                }
                ParseState::Done => { return Err(()); } // It means there are more arguments than needed
            }
        }
        
        // Checks if the parsing is done
        match state {
            ParseState::Done => {},
            _ => return Err(())
        }
        
        Ok(output)
    }
}

fn host_game() -> io::Result<TcpStream> {
    println!("Waiting for incoming connection...");
    let listener = TcpListener::bind("0.0.0.0:3334")?;
    Ok(listener.accept().unwrap().0)
}

fn join_game(addr: &str) -> io::Result<TcpStream> {
    TcpStream::connect(addr)
}

fn win() {
    println!("You win!");
}

fn lose() {
    println!("You lose...");
}

fn draw() {
    println!("It's a draw !!");
}

fn print_usage() {
    println!(
"Usage:
    rps host [rock|paper|scissors]
    rps join [IP (Ex: 127.0.0.1)] [rock|paper|scissors]");
}

fn play_animation() {
    for i in (1..=3).rev() {
        println!("{}...", i);
        thread::sleep(time::Duration::new(1, 0));
    }
}

fn main() -> () {
    let args_list: Vec<String> = env::args().collect();
    let args = Arguments::parse(&args_list);
    if args.is_err() { print_usage(); return; }
    let args = args.unwrap();

    let mut ascii_art: HashMap<&str, &str> = HashMap::new();
    ascii_art.insert("rock",
"    _______
---'   ____)
      (_____)
      (_____)
      (____)
---.__(___)");
    ascii_art.insert("paper",
"     _______
---'    ____)____
           ______)
          _______)
         _______)
---.__________)");
    ascii_art.insert("scissors",
"    _______
---'   ____)____
          ______)
       __________)
      (____)
---.__(___)");

    let conn = match args.mode {
        ConnectionMode::Hosting => { host_game() },
        ConnectionMode::Joining => { join_game(&args.ip) },
    };

    if conn.is_err() {
        eprintln!("Failed to initiate the game!");
        return;
    }

    let choice = match args.choice {
        PlayerChoice::Rock => { "rock" },
        PlayerChoice::Paper => { "paper" },
        PlayerChoice::Scissors => { "scissors" },
    };

    let mut conn = conn.unwrap();

    conn.write(choice.as_bytes()).unwrap();

    let mut buffer = [0; 16];
    conn.read(&mut buffer).unwrap();
    let other: String = std::str::from_utf8(&buffer).unwrap().bytes()
        .filter(|b| b != &0)
        .map(|b| b as char)
        .collect();

    play_animation();
    match other.as_str() {
        "rock" => { println!("{}", ascii_art.get(other.as_str()).unwrap()); },
        "paper" => { println!("{}", ascii_art.get(other.as_str()).unwrap()); },
        "scissors" => { println!("{}", ascii_art.get(other.as_str()).unwrap()); },
        _ => {}
    }

    match (choice, other.as_str()) {
        ("rock", "rock") => { draw(); },
        ("rock", "paper") => { lose(); },
        ("rock", "scissors") => { win(); },
        ("paper", "rock") => { win(); },
        ("paper", "paper") => { draw(); },
        ("paper", "scissors") => { lose(); },
        ("scissors", "rock") => { lose(); },
        ("scissors", "paper") => { win(); },
        ("scissors", "scissors") => { draw(); },
        (_, _) => { println!("Other player cheated! ({})", other); }
    }
}
