use crate::case::*;
use crate::restriction::*;
use crate::prover::Seq;

/**
 * Stores a record of the maximal/minimal values encountered in the simulation.
 * This corresponds to a single subcase.
 */
pub struct Extrema {
    min_as: Seq,
    max_as: Seq,
    sum_lower_bounds: Vec<Option<i128>>,
    denominator: u128,
}

/**
 * Wraps up Extrema for a number of subcases.
 * There is one Results structure for each subcase the program runs on.
 */
pub struct Results {
    subcases: Vec<(Vec<Restriction>, Extrema)>,
    sum_lower_bound_coefs: Vec<Vec<i32>>,
    default_subcase: Extrema,
}

impl Extrema {
    pub fn new(denominator: u128, max_depth: usize, num_sums: usize) -> Extrema {
        Extrema {
            min_as: Seq::new(denominator, denominator, max_depth),
            max_as: Seq::new(0, denominator, max_depth),
	    sum_lower_bounds: vec![None; num_sums],
            denominator
        }
    }

    /**
     * Include this sequence into the min and max bounds we store.
     */
    pub fn include_seq(&mut self, seq: &Seq, sum_lower_bounds_coefs: &Vec<Vec<i32>>,
		       depth: usize) {
        for (i, numerator) in seq.iter_numerators().enumerate() {
            let old_min_val = self.min_as.get_min_numerator(i);
            let old_max_val = self.max_as.get_min_numerator(i);
            self.min_as.set(i, old_min_val.min(*numerator));
            self.max_as.set(i, old_max_val.max(*numerator));
        }
	for (index, coefs) in sum_lower_bounds_coefs.iter().enumerate() {
	    if depth >= coefs.len() {
		let mut lower_bound = 0;
		for (numerator, coef) in seq.iter_numerators().take(coefs.len()).zip(coefs.iter()) {
		    if *coef >= 0 {
			lower_bound += (*numerator as i128) * (*coef as i128);
		    } else {
			lower_bound += ((*numerator + 1) as i128) * (*coef as i128); 
		    }
		}
		if let Some(old_lower_bound) = self.sum_lower_bounds[index] {
		    if lower_bound < old_lower_bound {
			self.sum_lower_bounds[index] = Some(lower_bound);
		    }
		} else {
		    self.sum_lower_bounds[index] = Some(lower_bound);
		}
	    }
	}
    }

    /**
     * Returns the maximum distance of any of the intervals from one of the
     * possible difficult cases (i.e. 0, 1/4, 1/3, 1/2, 2/3, 1)
     */
    pub fn get_max_delta(&self, target: f64, depth: usize) -> f64 {
	let mut max_delta: f64 = 0.0;
	for i in 0..depth {
	    let min = self.min_as.get_min(i);
	    let max = self.max_as.get_max(i);
	    let mut min_delta = 1.0;
	    for target in [0.0, target, 2.0 * target] {
		let delta = (target - min).abs().max((max - target).abs());
		if delta < min_delta {
		    min_delta = delta;
		}
	    }
	    if min_delta > max_delta {
		max_delta = min_delta;
	    }
	}
	max_delta
    }

    /**
     * Returns the minimum possible sum. This is computed during include_seq, and here
     * the result is merely retrieved and returned.
     */
    pub fn get_sum_lower_bound(&self, index: usize) -> Option<i128> {
	self.sum_lower_bounds[index]
    }

    /**
     * Return whether there was any sequence which could not be ruled out.
     */
    pub fn is_contradiction(&self) -> bool {
	self.min_as.get_min(0) > self.max_as.get_max(0)
    }

    /**
     * This prints this structure in a human-readable format.
     */
    pub fn print(&self, bounds: &Vec<Interval>) {
	if self.is_contradiction() {
	    println!("Case resolved: no sequence can satisfy given conditions!");
	} else {
            for (index, (lower, upper)) in self.min_as.iter_numerators().zip(self.max_as.iter_numerators()).enumerate() {
		let interval = bounds.get(index).unwrap_or(&Interval::UNIT);
		let lb = ((*lower as f64) / (self.denominator as f64)).max(interval.lb);
		let ub = (((*upper + 1) as f64) / (self.denominator as f64))
		    .min(interval.ub);
		println!("{} <= a_{} <= {}", lb, index, ub);
            }
	}
    }

    /**
     * This prints the Extrema in a format which can be immediately recycled to
     * run again.
     */
    pub fn print_machine(&self, case: &Case, subcase: &Vec<Restriction>) {
	if self.min_as.get_min(0) > self.max_as.get_max(0) {
	    println!("Case resolved: no sequence can satisfy given conditions!");
	} else {
	    println!("{}, {}, {}, {}", case.threshold, case.prob_cutoff,
		     case.max_depth, case.denominator);
	    for restriction in case.restrictions.iter() {
		println!("{:?}", restriction);
	    }
	    for restriction in subcase.iter() {
		println!("{:?}", restriction);
	    }
            for (index, (lower, upper)) in self.min_as.iter_numerators().zip(self.max_as.iter_numerators()).enumerate() {
		let interval = case.bounds.get(index).unwrap_or(&Interval::UNIT);
		let lb = ((*lower as f64) / (self.denominator as f64)).max(interval.lb);
		let ub = (((*upper + 1) as f64) / (self.denominator as f64))
		    .min(interval.ub);
		println!("Bounds({}, {}, {})", index, lb, ub);
            }
	}
    }
}

impl Results {
    pub fn new(case: &Case) -> Results {
        let mut subcases = vec![];
	let mut sum_lower_bound_coefs = vec![];
	for hypothesis in case.hypotheses.iter() {
	    if let Hypothesis::SumLowerBound(coefs, _bound) = hypothesis {
		sum_lower_bound_coefs.push(coefs.to_owned());
	    }
	}
	let num_sums = sum_lower_bound_coefs.len();
        for subcase in case.subcases.iter() {
            subcases.push((subcase.to_owned(),
			   Extrema::new(case.denominator, case.max_depth, num_sums)));
        }
        Results {
            subcases,
	    sum_lower_bound_coefs,
            default_subcase: Extrema::new(case.denominator, case.max_depth, num_sums)
        }
    }

    /**
     * Pass the Seq to every applicable subcase to be stored.
     */
    pub fn include_seq(&mut self, seq: &Seq, depth: usize) {
        let mut is_in_any_subcase = false;
        for (subcase, extrema) in self.subcases.iter_mut() {
            if seq.could_satisfy_restrictions(subcase, depth) {
                extrema.include_seq(seq, &self.sum_lower_bound_coefs, depth);
                is_in_any_subcase = true;
            }
        }
        if !is_in_any_subcase {
            self.default_subcase.include_seq(seq, &self.sum_lower_bound_coefs, depth);
        }
    }

    fn as_label(index: usize) -> char {
	char::from_u32(index as u32 + ('A' as u32)).unwrap()
    }
        
    pub fn print(&self, bounds: &Vec<Interval>) {
	for (index, (subcase, extrema)) in self.subcases.iter().enumerate() {
            println!();
            println!("Subcase {}: {:?}:", Self::as_label(index), subcase);
            extrema.print(bounds);
        }
        println!();
        println!("Default subcase (subcase {}):", Self::as_label(self.subcases.len()));
        self.default_subcase.print(bounds);
    }

    pub fn print_machine(&self, case: &Case) {
	for (index, (subcase, extrema)) in self.subcases.iter().enumerate() {
            println!();
            println!("Subcase {}: {:?}:", Self::as_label(index), subcase);
            extrema.print_machine(case, &subcase);
        }
        println!();
        println!("Default subcase (subcase {}):", Self::as_label(self.subcases.len()));
        self.default_subcase.print_machine(case, &vec![]);
    }

    pub fn get_max_delta(&self, target: f64, depth: usize) -> f64 {
	let mut max_delta: f64 = self.default_subcase.get_max_delta(target, depth);
	for (_subcase, extrema) in self.subcases.iter() {
	    max_delta = max_delta.max(extrema.get_max_delta(target, depth));
	}
	max_delta
    }

    /**
     * Returns the smallest value found for a given sum of a_i. This is computed
     * during include_seq, so here the result is retrieved. We need to match the
     * coefs against the list of stored coef lists, which is unweildly.
     */
    pub fn get_sum_lower_bound(&self, coefs: &Vec<i32>) -> Option<f64> {
	let mut index = None;
	'find_coefs: for (i, these_coefs) in self.sum_lower_bound_coefs.iter().enumerate() {
	    if these_coefs.len() == coefs.len() {
		let mut are_equal = true;
		'test_coefs: for (x, y) in coefs.iter().zip(these_coefs.iter()) {
		    if *x != *y {
			are_equal = false;
			break 'test_coefs;
		    }
		}
		if are_equal {
		    index = Some(i);
		    break 'find_coefs;
		}
	    }
	}
	let index = index.unwrap();
	let mut min_sum = self.default_subcase.get_sum_lower_bound(index);
	for (_subcase, extrema) in self.subcases.iter() {
	    min_sum = match (min_sum, extrema.get_sum_lower_bound(index)) {
		(Some(x), Some(y)) => Some(x.min(y)),
		(Some(x), None) | (None, Some(x)) => Some(x),
		(None, None) => None,
	    }
	}
	match min_sum {
	    Some(numerator) => Some((numerator as f64) / (self.default_subcase.denominator as f64)),
	    None => None,
	}
    }

    pub fn is_contradiction(&self) -> bool {
	let mut is_contradiction = self.default_subcase.is_contradiction();
	for (_subcase, extrema) in self.subcases.iter() {
	    if !extrema.is_contradiction() {
		is_contradiction = false
	    }
	}
	is_contradiction
    }
}
