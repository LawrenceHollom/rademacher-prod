use std::io::{self, Write};

use cached::proc_macro::cached;

/**
 * This code is a direct translation of the code from the paper of Dvorak and Klein.
 * Paper accessible at: https://epubs.siam.org/doi/abs/10.1137/21M1428212
 * Code at: https://github.com/IamPoosha/oleszkiewicz-problem/blob/main/verification.py
 * There was one bug we fixed during the translation. Both are marked with comments.
 * The other major change is the introduction of Bernstein's inequality to give better
 * bounds in some extreme cases.
 */

const DEFAULT_EPSILON: f64 = 0.001; // 0.002
const PI: f64 = std::f64::consts::PI;
const D_ITERATIONS: usize = 1000; // 200
const N: usize = 2000; // 1000

// The solution of exp(-x^2/2)+cos(x) = 0 with x in [0, pi]
const THETA: f64 = 1.7780882886686339603;

// Characteristic function of a standard normal variable
fn normal_char(x: f64) -> f64 {
    (- x * x / 2.0).exp()
}

// An upper bound on |f_X(v)|, given an upper bound on a1
// This is h(v, a) from page 12 of the paper. Note there are more cases there not used here.
fn fx_bound(v: f64, a1: f64) -> f64 {
    // The bound is correct only in "a1 * v < pi" range, which we assert.
    assert!(a1*v < PI);
    if a1 * v < THETA {
        normal_char(v)
    } else { // if a1*v < pi
        (-(a1 * v).cos()).powf(1.0 / a1.powi(2))
    }
}

// An upper bound on |f_X(v)-normal_char(v)|, given an upper bound on a1
// This is g(v, a) on page 12 of the paper.
fn difference_bound(v: f64, a1: f64) -> f64 {
    // the bound is correct only in "a1 * v < pi / 2" range, which we assert.
    assert!(a1 * v <= PI / 2.0);
    normal_char(v) - (a1 * v).cos().powf(1.0 / a1.powi(2))
}

// k(u, x, T) from the paper.
fn k(u: f64, x: f64, t: f64) -> f64 {
    let txu = t * x * u;
    if u == 0.0 {
        1.0 + t * x / PI
    } else if u == 1.0 {
        0.0
    } else {
        (1.0 - u) * (PI * u + txu).sin() / (PI * u).sin() + txu.sin() / PI
    }
}

fn lipschitz_integrate(f: &dyn Fn(f64) -> f64, start: f64, end: f64, epsilon: f64, derivative_bound: f64, max_f_error: f64) -> f64 {
    let width = end - start;
    let num_steps = (2.0 + derivative_bound * width.powi(2) / (4.0 * (epsilon - max_f_error * width))) as usize;
    // ensures the implied error is smaller than epsilon
    let error = derivative_bound * width.powi(2) / (4.0 * num_steps as f64) + width * max_f_error;
    assert!(error < epsilon);
    let mut sum = 0.0;
    for k in 0..num_steps {
        sum += f(start + (2 * k + 1) as f64 * width / (2.0 * num_steps as f64));
    }
    (end - start) * sum / num_steps as f64
}

fn compute_f(a1: f64, x: f64, t: f64, q: f64, epsilon: f64) -> f64 {
    let tx = (t * x).abs();
    // The three integrands are Lipschitz with the following constants.
    // The Bounds are derived in Appendix titled "Numeric integration in our proofs"
    // of:   https://arxiv.org/pdf/2006.16834.pdf
    let bound1 = t * (1.0 + 2.0 * tx / PI) + 1.1 * (tx.powi(2) / (2.0 * PI) + PI);
    let bound2 = t * (1.0 + 2.0 * tx / PI) + tx.powi(2) / (2.0 * PI) + PI;
    let bound3 = 2.0 * (t / 3.0) *(1.0 + 2.0 * tx / PI) + tx.powi(2) / (2.0 * PI) + PI;
    // Computing the integrated functions has absolute error < abs_error
    let abs_error = 2.0_f64.powi(-40) * (2.0 + tx);

    // the maximal additive errors sum to < eps
    let sum1 = lipschitz_integrate(&|u| k(u, x, t).abs() * difference_bound(u*t, a1),
        0.0, q, epsilon / 4.0, bound1, abs_error);
    let sum2 = lipschitz_integrate(&|u| k(u, x, t).abs() * fx_bound(u*t, a1),
        q, 1.0, epsilon / 4.0, bound2, abs_error);
    let sum3 = lipschitz_integrate(&|u| k(u, x, t) * normal_char(u*t),
        0.0, q, epsilon / 4.0, bound3, abs_error);

    // the value of F, minus the additive error allowed in the integration.
    0.5 - epsilon - (sum1 + sum2 + sum3)
}

// lower bound on Pr[X > x] for a Rademacher sum X
// with largest coefficient <= a, and Variance = 1.
// Just an application of F with T = pi/a, q = 0.5.
// We pass things in as fractions so that we play nicely with #cached.
#[cached]
pub fn prawitz_bound(a_num: i32, a_denom: usize, x_num: i32, x_denom: usize) -> f64 {
    let a = a_num as f64 / a_denom as f64;
    let x = x_num as f64 / x_denom as f64;
    // If a1 is small, we increase it for efficiency reasons.
    // This is allowed --
    //   F2 lower bounds the supremum of Pr[X > x] where X is a
    //   normalized Rademacher sums with largest coefficient <= a1.
    if a < 0.1 {
        prawitz_bound(1, 10, x_num, x_denom)
    } else {
        let out = compute_f(a, x, PI/a, 0.5, DEFAULT_EPSILON).max(0.0);
        out
    }
}

// round v to the next multiple of g.
fn round_up(v: i32, denom: usize) -> i32 {
    let d = denom as i32;
    if v >= 0 {
        ((v + d - 1) / d) * d
    } else {
        (v / d) * d
    }
}

pub fn prawitz_bound_raw(a: usize, y: usize, coef_gran: usize, thresh_gran: usize, max_bound: usize) -> f64 {
    prawitz_bound(round_up(a as i32, 16) + 1, coef_gran,
        round_up(y as i32 - max_bound as i32, 8) + 1, thresh_gran)
}

////// DYNAMIC PROGRAMMING //////

pub struct Bounder {
    bounds: Vec<Vec<f64>>,
    coef_gran: usize,
    thresh_gran: usize,
    max_bound: usize,
}

impl Bounder {
    /**
     * This is the function which actually pulls out our values for the function D.
     * A bug was fixed here during the translation from python, wherein negative
     * values of cutoff were rounded the wrong way.
     */
    fn get_internal(bounds: &Vec<Vec<f64>>, coef_gran: usize, thresh_gran: usize, max_bound: usize, a: f64, cutoff: f64) -> f64 {
        // A[M-1] represents a_1 = 1 case.
        let a_scaled = ((a * coef_gran as f64).ceil() as usize).min(bounds.len() - 1);
        let cutoff_scaled = (((cutoff * (thresh_gran as f64)) + max_bound as f64).ceil() as usize).max(0);
        // A clear lower bound
        if cutoff_scaled >= bounds[a_scaled].len() {
            0.0
        } else {
            bounds[a_scaled][cutoff_scaled]
        }
    }

    pub fn new_manual(bounds: Vec<Vec<f64>>, coef_gran: usize, thresh_gran: usize, max_bound: usize) -> Bounder {
        Bounder { bounds, coef_gran, thresh_gran, max_bound }
    }

    pub fn header_line(&self) -> String {
        format!("{},{},{}", self.coef_gran, self.thresh_gran, self.max_bound)
    }

    pub fn bounds(&self) -> &Vec<Vec<f64>> {
        &self.bounds
    }

    pub fn new() -> Bounder {
        let coef_gran = N;
        let thresh_gran = N;
        let max_bound = 3 * thresh_gran;
        let mut bounds = vec![vec![0.0; 2 * max_bound]; coef_gran];

        print!("Precomputation #1, {} steps: ", 2 * max_bound);
        for y in 0..(2 * max_bound) {
            if y % 100 == 0 {
                print!("{}% ", (y * 100) / (2 * max_bound));
                let _ = io::stdout().flush();
            }
            for a in 0..coef_gran {
                // The round-up is a (pessimistic) speedup. To allow caching.
                bounds[a][y] = prawitz_bound_raw(a, y, coef_gran, thresh_gran, max_bound);
                // If threshold < 0, then Pr[X > threshold] >= 1/2.
                if y < max_bound {
                    bounds[a][y] = bounds[a][y].max(0.5)
                }
            }
        }

        println!();
        print!("Precomputation #2, {} steps: ", D_ITERATIONS);

        for i in 0..D_ITERATIONS {
            if i % 5 == 0 {
                print!("{}% ", (i * 100) / D_ITERATIONS);
		let _ = io::stdout().flush();
            }
            for y in 0..(2 * max_bound) {
                // The threshold we consider.
                let t = (y as f64 - max_bound as f64 + 1.0) / thresh_gran as f64;
                for a in 0..coef_gran {
                    // In bounds[a][y+max_bound] we assign a lower bound to Pr[X >= t],
                    // given a_1 <= (a+1)/coef_gran.
                    // We split into two cases:
                    //   a_1 <= a/coef_gran
                    //   a_1 in [a/coef_gran, (a+1)/coef_gran]
                    // The first case may be lower bounded using bounds[a-1, y+max_bound].
                    // The second case is lower bounded by elimination
                    // of largest coefficient, and trivial bounds.
                    // The lower bound is the minimum of the two cases.
                    //
                    // We start with the second case:
                    let min_a_1 = a as f64 / coef_gran as f64;
                    let max_a_1 = (a as f64 + 1.0) / coef_gran as f64;
                    // minimum variance of a_2 * epsilon_2 + ... + a_n * epsilon_n
                    let min_sigma = (1.0 - max_a_1.powi(2)).powf(0.5);
                    // if t <= a_1, clearly Pr[X >= t]
                    // is lower bounded by 1/4:
                    // the Rademacher sum is larger than t whenever both
                    //   sign of a_1 is positive (probability 1/2)
                    //   sign of the rest of the process is positive (probability >= 1/2)
                    let mut bound: f64 = if t <= min_a_1 { 0.25 } else { 0.0 };
                    // Note that the case a = coef_gran-1 which includes a_1 = 1,
                    // for which elimination is prohibited, is handled correctly.
                    if a+1 < coef_gran {
                        let sta1 = Self::get_internal(&bounds, coef_gran, thresh_gran, max_bound,
                            max_a_1 / min_sigma, (t - min_a_1) / min_sigma);
                        let sta2 = Self::get_internal(&bounds, coef_gran, thresh_gran, max_bound,
                            max_a_1 / min_sigma, (t + max_a_1) / min_sigma);
                        bound = bound.max((sta1 + sta2) / 2.0);
                    }
                    // We now consider the case a_1 <= a / coef_gran, and take the minimum.
                    if a > 0 {
                        bound = bound.min(bounds[a - 1][y]);
                    }
                    // If we got a better lower bound to bounds[a][y], we update it.
                    if bound > bounds[a][y] {
                        bounds[a][y] = bound;
                    }
                }
            }
        }

        println!();

        Bounder { bounds, coef_gran, thresh_gran, max_bound }
    }

    /**
     * Returns our best lower bound on the function P(X > cutoff)
     */
    pub fn get(&self, a: f64, cutoff: f64) -> f64 {
        /**
         * Bernstein's inequality; from https://en.wikipedia.org/wiki/Bernstein_inequalities_(probability_theory)
         * (first one in 'some of the inequalities' section.)
         */
        fn get_bernstein(a: f64, t: f64) -> f64 {
            1.0 - ((- (t * t)) / (2.0 * (1.0 - (a * t / 3.0)))).exp()
        }

        let d = Self::get_internal(&self.bounds, self.coef_gran,
            self.thresh_gran, self.max_bound, a, cutoff);
        if cutoff < -3.0 {
            d.max(get_bernstein(a, cutoff))
        } else {
            d
        }
    }

    pub fn print(&self, a: f64, cutoff: f64) {
        let val = self.get(a, cutoff);
        let a_scaled = ((a * self.coef_gran as f64) as usize).min(self.bounds.len() - 1);
        let cutoff_scaled = ((cutoff * self.thresh_gran as f64) as usize + self.max_bound).max(0);
        println!("D({}, {}) ~ bounds[{}][{}] = {}", a, cutoff, a_scaled, cutoff_scaled, val);
    }

    pub fn get_with_var(&self, a: f64, cutoff: f64, min_remaining_var: f64, max_remaining_var: f64) -> f64 {
        if min_remaining_var > 0.0 {
            if cutoff >= 0.0 {
                // Make cutoff as large in absolute value as possible
                self.get(a / min_remaining_var.sqrt(), cutoff / min_remaining_var.sqrt())
            } else {
                // Make cutoff as small in absolute value as possible
                self.get(a / min_remaining_var.sqrt(), cutoff / max_remaining_var.sqrt())
            }
        } else {
            if cutoff >= 0.0 {
                // Make cutoff as large as possible - in this case, infinite.
                0.0
            } else {
                // Make cutoff as small in absolute value as possible. a is infinite so rounds down to 1.
                self.get(1.0, cutoff / max_remaining_var.sqrt())
            }
        }
    }
}
