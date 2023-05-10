use crate::restriction::*;

pub enum Hypothesis {
    DeltaBound(f64, f64),
    Contradiction,
    None,
}

/**
 * This stores all the information about an instance of the problem, and how it
 * is to be run.
 * This structure is produced in file_io.rs
 */
pub struct Case {
    pub threshold: f64,
    pub prob_cutoff: f64,
    pub max_depth: usize,
    pub denominator: u128,
    pub bounds: Vec<Interval>,
    pub restrictions: Vec<Restriction>,
    pub subcases: Vec<Vec<Restriction>>,
    pub hypothesis: Hypothesis,
}

impl Case {
    /**
     * This returns the lower bound we have on the numerator of a_depth in this case
     */
    pub fn get_lower_bound(&self, depth: usize) -> u128 {
	if let Some(interval) = self.bounds.get(depth) {
	    (interval.lb * (self.denominator as f64)) as u128
	} else {
	    0
	}
    }

    pub fn get_upper_bound(&self, depth: usize) -> u128 {
	if let Some(interval) = self.bounds.get(depth) {
	    (interval.ub * (self.denominator as f64)) as u128
	} else {
	    self.denominator - 1
	}
    }
}
