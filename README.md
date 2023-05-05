# Rademacher Sums

This project is the companion code to the paper <i>Tight lower bounds for anti-concentration of Rademacher sums and Tomaszewski’s counterpart problem</i> by L. Hollom and J. Portier, and produces many of the bounds used in that paper. The paper itself serves as a complete description of the mathematical footing of the project, and the content here concern only the practical details of the code.

The core of the code is based on the project https://github.com/ohadkel/oleszkiewicz-problem. Here it is translated into Rust and extended to perform significantly more general simulations.

## Using the code
The code should be run as a rust cargo project. There are three functions which can be run:
1. `generate` - this should be run after first setting up the code. This generates and saves down the table of bounds used in the simulation. With default parameters, this takes (very) roughly 30 minutes to run.
2. `D(a,x)` - this prints out a single value of the bounding function. Here, a and x are numbers, where a should be between 0 and 1 inclusive.
3. `run(file)` - this runs the simulation on the parameters stored in the given file, for example `run(2)` or `run(0CBAA)`

## Enforcing manual bounds
In several places, bounds are manually added to speed up computation, with proofs in the appendix of the paper linked above. These are:
- `0FAA`: we enforce `InitialSumUpperBound(3, 1.0)` and `MidSumUpperBound(1, 5, 1.0)`
- `0GA`: we enforce `MidSumUpperBound(2, 6, 1.0)` and `MidSumUpperBound(1, 4, 1.0)`
