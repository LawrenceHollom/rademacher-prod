use std::{fs::{self, File}, path::PathBuf, io::{BufWriter, Write}};

use crate::prawitz::*;
use crate::util::*;
use crate::restriction::*;
use crate::case::*;

/**
 * This file deals with all of the reading from, and writing to files.
 */

fn get_root() -> PathBuf {
    let mut pathbuf = std::env::current_exe().unwrap();
    pathbuf.pop();
    while !pathbuf.ends_with("rademacher-prod/") {
        pathbuf.pop();
    }
    pathbuf
}

/**
 * Constructs a Case structure from a file containing all of the relevant information
 */
pub fn get_case(filename: &String) -> Option<Case> {
    let mut pathbuf = get_root();
    pathbuf.push(format!("cases/{}.txt", filename));
    match fs::read_to_string(pathbuf) {
        Ok(contents) => {
            let mut lines = contents.trim().lines();
            let pars = lines.nth(0).unwrap().split(',').collect::<Vec<&str>>();
            let threshold = pars[0].trim().parse().unwrap();
	    let prob_cutoff = pars[1].trim().parse().unwrap();
	    let max_depth = pars[2].trim().parse().unwrap();
	    let denominator = pars[3].trim().parse().unwrap();

	    let mut bounds_list: Vec<(usize, Interval)> = vec![];
	    let mut restrictions = vec![];
	    let mut subcases = vec![];
	    let mut num_bounds = 0;
	    let mut hypothesis = Hypothesis::None;
	    
            for line in lines {
		let (func, args) = parse_function_like(line);
		match func.trim().to_lowercase().as_str() {
		    "bounds" => {
			let index: usize = args[0].trim().parse().unwrap();
			let interval = Interval {
			    lb: args[1].trim().parse().unwrap(),
			    ub: args[2].trim().parse().unwrap(),
			};
			bounds_list.push((index, interval));
			if index >= num_bounds {
			    num_bounds = index + 1;
			}
		    }
		    "subcase" => {
			let restrictions = args.iter()
			    .map(|x| Restriction::of_string(*x))
			    .collect::<Vec<Restriction>>();
			subcases.push(restrictions);
		    }
		    "provesbound" => {
			use Hypothesis::*;
			let target = args[0].trim().parse().unwrap();
			let delta = args[1].trim().parse().unwrap();
			match hypothesis {
			    DeltaBound(_, _) | Contradiction => {
				panic!("Only one hypothesis may be proved at a time!")
			    }
			    None => {
				hypothesis = DeltaBound(target, delta);
			    }
			}
		    }
		    "contradiction" => {
			use Hypothesis::*;
			match hypothesis {
			    DeltaBound(_, _) | Contradiction => {
				panic!("Only one hypothesis may be proved at a time!")
			    }
			    None => {
				hypothesis = Contradiction;
			    }
			}
		    }
		    &_ => {
			let restriction = Restriction::of_string(line);
			if let Restriction::Bounds(index, interval) = restriction {
			    bounds_list.push((index, interval));
			    if index >= num_bounds {
				num_bounds = index + 1;
			    }
			} else {
			    restrictions.push(restriction);
			}
		    }
			
		}
            }

	    let mut bounds = vec![Interval::UNIT; num_bounds];

	    for (index, interval) in bounds_list.iter() {
		bounds[*index].intersect_inplace(interval);
	    }
	    
            Some(Case { threshold, prob_cutoff, max_depth, denominator, bounds,
	       restrictions, subcases, hypothesis })
        }
        Err(_e) => None
    }
}

pub fn bounder_to_file(bounder: &Bounder) {
    println!("  WRITING BOUNDER! ");
    let mut pathbuf = get_root();
    pathbuf.push(format!("bounder.csv"));
    let mut writer = BufWriter::new(File::create(pathbuf).unwrap());
    let _ = writer.write(bounder.header_line().as_bytes());
    let _ = writer.write("\n".as_bytes());
    for row in bounder.bounds().iter() {
        let _ = writer.write(row.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",").as_bytes());
        let _ = writer.write("\n".as_bytes());
    }
}

pub fn bounder_from_file() -> Bounder {
    let mut pathbuf = get_root();
    pathbuf.push(format!("bounder.csv"));
    let contents = fs::read_to_string(pathbuf).unwrap();
    let mut lines = contents.trim().lines();
    let first_pars = lines.nth(0).unwrap().split(',').collect::<Vec<&str>>();
    let mut bounds = vec![];

    for line in lines {
        bounds.push(line.split(',').map(|x| x.parse().unwrap()).collect::<Vec<f64>>());
    }

    Bounder::new_manual(
        bounds,
        first_pars[0].parse().unwrap(),
        first_pars[1].parse().unwrap(),
        first_pars[2].parse().unwrap(),
    )
}
