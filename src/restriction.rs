use crate::util::*;

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub lb: f64,
    pub ub: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Restriction {
    InitialSumUpperBound(usize, f64),
    InitialSumLowerBound(usize, f64),
    MidSumUpperBound(usize, usize, f64),
    Bounds(usize, Interval),
}

impl Interval {
    pub const UNIT: Interval = Interval { lb: 0.0, ub: 1.0 };
}

impl Restriction {
    pub fn of_string(text: &str) -> Restriction {
	let (func, args) = parse_function_like(text);
	use Restriction::*;
	match func.trim().to_lowercase().as_str() {
	    "initialsumupperbound" => {
		InitialSumUpperBound(args[0].parse().unwrap(), args[1].parse().unwrap())
	    }
	    "initialsumlowerbound" => {
		InitialSumLowerBound(args[0].parse().unwrap(), args[1].parse().unwrap())
	    }
	    "midsumupperbound" => {
		MidSumUpperBound(
		    args[0].parse().unwrap(),
		    args[1].parse().unwrap(),
		    args[2].parse().unwrap())
	    }
	    "bounds" => {
		let interval = Interval {
		    lb: args[1].parse().unwrap(),
		    ub: args[2].parse().unwrap(),
		};
		Bounds(args[0].parse().unwrap(), interval)
	    }
	    &_ => panic!("Unknown restriction!")
	}
    }
}
