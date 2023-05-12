use std::io::{self, Write};

use crate::prawitz::Bounder;
use crate::restriction::*;
use crate::case::*;
use crate::extrema::*;

// To mitigate risk of floating-point errors.
const EPSILON: f64 = 0.0000000001;
const DELTA_ERROR: f64 = 0.000001;

/**
 * Represents a sequence of intervals. Interval i is
 * [numerators[i] / denominator, (numerators[i] + 1) / denominator]
 */
pub struct Seq {
    pub numerators: Vec<u128>,
    pub denominator: u128,
}

impl Seq {
    pub fn new(numerator: u128, denominator: u128, max_depth: usize) -> Seq {
        Seq {
            numerators: vec![numerator; max_depth],
            denominator
        }
    }

    pub fn set(&mut self, index: usize, numerator: u128) {
        self.numerators[index] = numerator;
    }

    pub fn get_min_numerator(&self, index: usize) -> u128 {
        self.numerators[index]
    }

    pub fn get_min(&self, index: usize) -> f64 {
        self.numerators[index] as f64 / self.denominator as f64
    }

    pub fn get_max(&self, index: usize) -> f64 {
        (self.numerators[index] + 1) as f64 / self.denominator as f64
    }

    pub fn iter_numerators(&self) -> impl Iterator<Item = &u128> {
        self.numerators.iter()
    }

    /**
     * Computes the minimum possible variance (i.e. each a_i at the bottom
     * of its interval)
     */
    pub fn min_variance(&self) -> f64 {
        let num_sum: u128 = self.numerators.iter().map(|x| *x * *x).sum();
        num_sum as f64 / ((self.denominator * self.denominator) as f64)
    }

    /**
     * Computes the maximum possible variance (i.e. each a_i at the top
     * of its interval)
     */
    pub fn max_variance(&self) -> f64 {
        let num_sum: u128 = self.numerators.iter().map(|x| (*x + 1) * (*x + 1)).sum();
        num_sum as f64 / ((self.denominator * self.denominator) as f64)
    }

    /**
     * Tests if this Seq satisfies all of the given list of Restriction s
     */
    pub fn satisfies_restrictions(&self, hints: &Vec<Restriction>,
				  depth: usize) -> bool {
        use Restriction::*;
        let mut satisfies_all = true;
        'test_hints: for hint in hints.iter() {
            match *hint {
                InitialSumUpperBound(sum_depth, bound) => {
                    let sum: u128 = self.numerators.iter()
			.take(depth.min(sum_depth)).sum();
                    if (sum as f64) / (self.denominator as f64) > bound {
                        satisfies_all = false;
                        break 'test_hints;
                    }
                }
                InitialSumLowerBound(sum_depth, bound) => {
                    if depth >= sum_depth {
                        let sum: u128 = self.numerators.iter()
			    .take(sum_depth).sum();
                        if (sum as f64 + sum_depth as f64) / (self.denominator as f64)
			    < bound {
                            satisfies_all = false;
                            break 'test_hints;
                        }
                    }
                }
                MidSumUpperBound(start, end, bound) => {
                    let sum: u128 = self.numerators.iter()
			.take(depth.min(end)).skip(start).sum();
                    if (sum as f64) / (self.denominator as f64) > bound {
                        satisfies_all = false;
                        break 'test_hints;
                    }
                }
                Bounds(index, interval) => {
                    if index < depth &&	(self.get_max(index) < interval.lb
			 || self.get_min(index) > interval.ub) {
                        satisfies_all = false;
                        break 'test_hints;
                    }
                }
            }
        }
        satisfies_all
    }

    /**
     * Returns whether this Seq could be a counterexample to
     *     P[ X >= bound * sqrt(Var(X)) ] >= prob_cutoff
     * including checking whether the variance is within acceptable limits.
     *
     * Note that even if min_remaining_var < 0, this still works as expected due to
     * some case analysis in bounder.get_with_var(...).
     */
    pub fn could_be_counterexample(&self, bounder: &Bounder, case: &Case,
				   depth: usize) -> bool {
        let min_remaining_var = 1.0 - self.max_variance();
        let max_remaining_var = 1.0 - self.min_variance();
        if self.min_variance() > 1.0 {
            // The variance is too large and so we can ignore this case.
            false
        } else {
            let mut total = 0.0;
            for signs_code in 0..(1 << depth) {
                let mut threshold_adjustment_numerator: i128 = 0;
                let mut sta = signs_code;
                for numerator in self.numerators.iter().take(depth) {
                    if sta % 2 == 1 {
                        threshold_adjustment_numerator += (*numerator + 1) as i128;
                    } else {
                        threshold_adjustment_numerator -= (*numerator) as i128;
                    }
                    sta /= 2;
                }
                let new_threshold = case.threshold +
		    (threshold_adjustment_numerator as f64 / self.denominator as f64);
                // In this case we care about P[ X >= new_bound ]
                total += bounder.get_with_var(self.get_max(depth - 1), new_threshold,
					      min_remaining_var, max_remaining_var);
            }
            let prob_lower_bound = total / (1 << depth) as f64;

            // Then this could be a counterexample if our computed lower bound
	    // isn't large enough.
            prob_lower_bound < case.prob_cutoff + EPSILON
        }
    }

    pub fn _print_compact(&self) {
        for numer in self.numerators.iter() {
            print!("{:.3} ", *numer as f64 / self.denominator as f64);
        }
        println!();
    }
}

/**
 * Recieves a newly-generated seq and then:
 *  - tests that seq satisfies the given hints; if it doesn't, then do nothing.
 *  - if we can immediately prove that seq cannot be a counterexample:
 *  - - then do nothing and return
 *  - else if not at max_depth:
 *  - - go one level deeper into seq
 *  - else:
 *  - - We fail to prove this case. Update lower_bounds and sumsq_bounds.
 *
 * lower_bounds[i] stores the smallest value of a_i we cannot deal with
 * sumsq_bounds[i] stores the minimal a_1^2+...+a_i^2 for a seq we cannot deal with.
 */
fn simulate_rec(bounder: &Bounder, seq: &mut Seq, results: &mut Results,
        case: &Case, depth: usize) {
    if seq.satisfies_restrictions(&case.restrictions, depth)
	&& seq.could_be_counterexample(bounder, case, depth) {
        if depth < case.max_depth {
            let min = case.get_lower_bound(depth);
            let max = seq.get_min_numerator(depth - 1).min(case.get_upper_bound(depth));
            for numerator in min..=max {
                seq.set(depth, numerator);
                simulate_rec(bounder, seq, results, case, depth + 1);
            }
            seq.set(depth, 0);
        } else {
            results.include_seq(seq, depth);
        }
    }
}

/**
 * Runs a simulation to produce a sequence of lower-bounds on the a_i for the problem
 *     P[ X >= bound * sqrt(Var(X)) ] >= prob_cutoff
 * i.e. if any a_i is below the returned lower bound, then the simulation here
 * has automatically proven that the above inequality must hold.
 */
pub fn simulate(bounder: &Bounder, case: Case) {
    // We run with a fixed denominator.
    let mut seq = Seq::new(0, case.denominator, case.max_depth);
    let mut results = Results::new(&case);
    let min = case.get_lower_bound(0);
    let max = case.get_upper_bound(0);
    for numerator in min..=max {
        print!("{:.1}% ", (100.0 * (numerator - min) as f64) / ((1 + max - min) as f64));
        let _ = io::stdout().flush();
        seq.set(0, numerator);
        simulate_rec(bounder, &mut seq, &mut results, &case, 1);
    }
    println!("100.0%");
    println!();
    println!("MACHINE-READABLE RESULTS:");
    results.print_machine(&case);
    println!();
    println!("HUMAN-READABLE RESULTS:");
    results.print(&case.bounds);
    use Hypothesis::*;
    let mut all_hypotheses_proved = true;
    println!();
    for hypothesis in case.hypotheses.iter() {
	match hypothesis {
	    DeltaBound(target, delta_bound) => {
		let max_delta = results.get_max_delta(*target, case.max_depth);
		if max_delta + DELTA_ERROR <= *delta_bound {
		    println!("We prove that delta <= {}. Actual max delta: {}",
			     delta_bound, max_delta);
		} else {
		    println!("delta not below bound: actual max delta = {} > {}",
			     max_delta, delta_bound);
		    all_hypotheses_proved = false;
		}
	    }
	    SumLowerBound(coefs, bound) => {
		let sum_bound = results.get_sum_lower_bound(&coefs);
		let mut proved = false;
		if let Some(sum_bound) = sum_bound {
		    if sum_bound >= *bound {
			println!("We prove for coefs {:?}, sum >= {}. Min sum = {}",
				 coefs, bound, sum_bound);
			proved = true;
		    }
		}
		if !proved {
		    println!("sum {:?} not above bound: actual min sum = {:?} < {}",
			     coefs, sum_bound, bound);
		    all_hypotheses_proved = false;
		}
	    }
	    Contradiction => {
		if results.is_contradiction() {
		    println!("There is a contradiction, as required.")
		} else {
		    println!("There is no contradiction.");
		    all_hypotheses_proved = false;
		}
	    }
	}
    }
    if case.hypotheses.len() >= 1 {
	println!();
	if all_hypotheses_proved {
	    println!("All hypotheses proved!");
	} else {
	    println!("FAILED to prove all hypotheses!");
	}
    }
    println!();
}
