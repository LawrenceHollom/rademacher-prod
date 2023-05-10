use std::{env, io::{self, Write}, time::SystemTime};

mod prawitz;
mod prover;
mod file_io;
mod util;
mod case;
mod extrema;
mod restriction;

use prawitz::*;
use util::*;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let mut bounder = None;
    
    fn prep(bounder: &mut Option<Bounder>) {
        if bounder.is_none() {
            println!("Running first time setup of Bounder object!");
            let start_time = SystemTime::now();
            *bounder = Some(file_io::bounder_from_file());
            println!("Finsihed reading file. Duration (secs): {}",
		     start_time.elapsed().unwrap().as_secs());
        }
    }

    loop {
        print!("Enter instruction: ");
        let _ = io::stdout().flush();
        let mut text = String::new();
        io::stdin().read_line(&mut text).expect("Failed to read line");
        let (func, args) = parse_function_like(&text);
	match func.trim().trim_end_matches(')').to_lowercase().as_str() {
	    "run" => {
		if let Some(case) = file_io::get_case(&args[0].to_owned()) {
                    prep(&mut bounder);
                    let start_time = SystemTime::now();
                    prover::simulate(bounder.as_ref().unwrap(), case);
                    println!("Simulation complete! Duration: {}s.",
			     start_time.elapsed().unwrap().as_secs());
                } else {
                    println!("Unknown case!");
                }
	    }
	    "d" => {
		prep(&mut bounder);
                if let (Ok(a), Ok(cutoff)) = (args[0].parse(), args[1].parse()) {
                    bounder.as_ref().unwrap().print(a, cutoff)
                } else {
                    println!("Failed to parse arguments! Expected format: D(a,x)");
                }
	    }
	    "generate" => {
		println!("Running first time computation of Bounder object!");
		let start_time = SystemTime::now();
		let new_bounder = Bounder::new();
		file_io::bounder_to_file(&new_bounder);
		bounder = Some(new_bounder);
		println!("Precomputation complete. Duration (secs): {}",
			 start_time.elapsed().unwrap().as_secs());
	    }
	    &_ => println!("Unknown command! Valid commands: run, d, generate."),
	}
    }
}
