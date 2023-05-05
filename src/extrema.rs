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
    denominator: u128,
}

/**
 * Wraps up Extrema for a number of subcases.
 * There is one Results structure for each Case the program runs on.
 */
pub struct Results {
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

    /**
     * Include this sequence into the min and max bounds we store.
     */
    pub fn include_seq(&mut self, seq: &Seq) {
        for (i, numerator) in seq.iter_numerators().enumerate() {
            let old_min_val = self.min_as.get_min_numerator(i);
            let old_max_val = self.max_as.get_min_numerator(i);
            self.min_as.set(i, old_min_val.min(*numerator));
            self.max_as.set(i, old_max_val.max(*numerator));
        }
    }

    pub fn print(&self, bounds: &Vec<Interval>) {
	if self.min_as.get_min(0) > self.max_as.get_max(0) {
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
        for subcase in case.subcases.iter() {
            subcases.push((subcase.to_owned(),
			   Extrema::new(case.denominator, case.max_depth)));
        }
        Results {
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
            self.default_subcase.include_seq(seq);
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
}
