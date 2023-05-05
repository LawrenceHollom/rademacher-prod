use rand::{thread_rng, rngs::ThreadRng, Rng};

use crate::case::*;
use crate::restriction::*;
use crate::prover::Seq;

const PRINT_PROB: f64 = 0.0001;

/**
 * Stores a record of the maximal/minimal values encountered in the simulation.
 */
pub struct Extrema {
    min_as: Seq,
    max_as: Seq,
    denominator: u128,
}

/**
 * Wraps up Extrema for a number of subcases.
 */
pub struct Results {
    rng: ThreadRng,
    subcases: Vec<(Vec<Restriction>, Extrema)>,
    default_subcase: Extrema,
}

impl Extrema {
    pub fn new(denominator: u128, max_depth: usize) -> Extrema {
        Extrema {
            min_as: Seq::new(denominator, denominator, max_depth),
            max_as: Seq::new(0, denominator, max_depth),
            denominator
        }
    }

    pub fn include_seq(&mut self, seq: &Seq) {
        for (i, numerator) in seq.iter_numerators().enumerate() {
            let old_min_val = self.min_as.get_min_numerator(i);
            let old_max_val = self.max_as.get_min_numerator(i);
            self.min_as.set(i, old_min_val.min(*numerator));
            self.max_as.set(i, old_max_val.max(*numerator));
        }
    }

    pub fn print(&self, bounds: &Vec<Interval>) {
        println!("Bounds:");
        for (index, (lower, upper)) in self.min_as.iter_numerators()
	    .zip(self.max_as.iter_numerators()).enumerate() {
            let lb = (*lower as f64) / (self.denominator as f64).max(bounds[index].lb);
            let ub = ((*upper + 1) as f64) / (self.denominator as f64)
		.min(bounds[index].ub);
            println!("{}, {}", lb, ub);
        }
    }
}

impl Results {
    pub fn new(case: &Case) -> Results {
        let mut subcases = vec![];
        for subcase in case.subcases.iter() {
            subcases.push((subcase.to_owned(),
			   Extrema::new(case.denominator, case.max_depth)));
        }
        Results {
            rng: thread_rng(),
            subcases,
            default_subcase: Extrema::new(case.denominator, case.max_depth)
        }
    }

    pub fn include_seq(&mut self, seq: &Seq, depth: usize) {
        let mut is_in_any_subcase = false;
        for (subcase, extrema) in self.subcases.iter_mut() {
            if seq.satisfies_restrictions(subcase, depth) {
                extrema.include_seq(seq);
                is_in_any_subcase = true;
            }
        }
        if !is_in_any_subcase {
            if self.rng.gen::<f64>() < PRINT_PROB {
                print!("default subcase. Rand success; prob {}: ", PRINT_PROB);
                seq._print_compact();
            }
            self.default_subcase.include_seq(seq);
        }
    }

    pub fn print(&self, bounds: &Vec<Interval>) {
	fn as_label(index: usize) -> char {
	    char::from_u32(index as u32 + ('A' as u32)).unwrap()
	}
        for (index, (subcase, extrema)) in self.subcases.iter().enumerate() {
            println!();
            println!("Subcase {}: {:?}:", as_label(index), subcase);
            extrema.print(bounds);
        }
        println!();
        println!("Default subcase (subcase {}):", as_label(self.subcases.len()));
        self.default_subcase.print(bounds);
    }
}
