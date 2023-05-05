use std::fmt;

use crate::util::*;

/**
 * A utility structure for storing intervals of floats.
 */
#[derive(Clone, Copy)]
pub struct Interval {
    pub lb: f64,
    pub ub: f64,
}

/**
 * Represents a restriction on the values of a_i that we may wish to enforce.
 * For example, a_0 + a_1 + a_2 < 1
 */
#[derive(Debug, Clone, Copy)]
pub enum Restriction {
    InitialSumUpperBound(usize, f64),
    InitialSumLowerBound(usize, f64),
    MidSumUpperBound(usize, usize, f64),
    Bounds(usize, Interval),
}

impl Interval {
    pub const UNIT: Interval = Interval { lb: 0.0, ub: 1.0 };

    /**
     * Mutates this interval to be the intersection of itself and another interval.
     */
    pub fn intersect_inplace(&mut self, other: &Interval) {
	self.lb = self.lb.max(other.lb);
	self.ub = self.ub.min(other.ub);
    }
}

impl Restriction {
    pub fn of_string(text: &str) -> Restriction {
	fn parse_index(text: &str) -> usize {
	    text.trim().parse().unwrap()
	}
	fn parse_float(text: &str) -> f64 {
	    text.trim().parse().unwrap()
	}
	let (func, args) = parse_function_like(text);
	use Restriction::*;
	match func.trim().to_lowercase().as_str() {
	    "initialsumupperbound" => {
		InitialSumUpperBound(parse_index(args[0]), parse_float(args[1]))
	    }
	    "initialsumlowerbound" => {
		InitialSumLowerBound(parse_index(args[0]), parse_float(args[1]))
	    }
	    "midsumupperbound" => {
		MidSumUpperBound(parse_index(args[0]),
				 parse_index(args[1]),
				 parse_float(args[2]))
	    }
	    "bounds" => {
		let interval = Interval {
		    lb: parse_float(args[1]),
		    ub: parse_float(args[2]),
		};
		Bounds(parse_index(args[0]), interval)
	    }
	    &_ => panic!("Unknown restriction!")
	}
    }
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.lb, self.ub)
    }
}
