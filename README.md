# Rademacher Sums

This project is the companion code to the paper <i>Tight lower bounds for anti-concentration of Rademacher sums and Tomaszewski’s counterpart problem</i> by L. Hollom and J. Portier, and produces many of the bounds used in that paper. The paper itself serves as a complete description of the mathematical footing of the project, and the content here concern only the practical details of the code.

The core of the code is based on the project https://github.com/ohadkel/oleszkiewicz-problem by Dvorak and Klein. Here it is translated into Rust and extended to perform significantly more general and efficient simulations.

## Using the code
The code should be run as a rust cargo project. There are three functions which can be run:
1. `generate` - this should be run after first setting up the code. This generates and saves down the table of bounds used in the simulation. With default parameters, this takes (very) roughly 30 minutes to run.
2. `D(a,x)` - this prints out a single value of the bounding function. Here, a and x are numbers, where a should be between 0 and 1 inclusive.
3. `run(file)` - this runs the simulation on the parameters stored in the given file, for example `run(2)` or `run(0CAA)`

## Syntax of the files in `cases/`
This directory contains fourty-two files, each containing a particular case which the program can run. This format of these files consists of a first line of four comma-separated numbers; `s, p, k, d`. 
This means that we are attempting to prove $`\mathbb{P}[X \leq s\sqrt{\text{Var}(X)}] \geq p`$, where $`X = a_0 \varepsilon_0+\cdots+ a_{k-1} \varepsilon_{k-1} `$ is a weighted sum of Rademacher random variables, and we will divide the interval $[0,1]$ into $d$-many intervals, each of width $1/d$.

Each further line has one of several forms, as listed below. They may occur in any order.
- `Bounds(i, x, y)`: this enforces that $x\leq a_i \leq y$.
- `InitialSumLowerBound(l, x)`: this enforces that $a_0+\dotsc+a_{l-1}\geq x$.
- `InitialSumUpperBound(l, x)`: this enforces that $a_0+\dotsc+a_{l-1}\leq x$.
- `MidSumUpperBound(l, m, x)`: this enforces that $a_l+\dotsc+a_{m-1} \leq x$. In particular, this must have $l < m$.
- `ProvesBound(d, x)`: this instructs the program to check if it can prove that all of $a_0,\dotsc,a_{k-1}$ are within $d$ of one of $x$ and $2x$.
- `ProvesSumLowerBound(c, x)`: this instructs the program to check if it can prove that the sum $c_0 a_0 +\dots + c_k a_k\geq x$. `c` should be formatted as a comma-separated list of integers, positive or negative, of any length.
- `Contradiction()`: this instructs the program to check if it can derive a contradiction, i.e. there are no values of $a_0,\dotsc,a_{k-1}$ which satisfy all the given conditions. This is something that the program does check anyway, so this line is used simply as a note that this is expected. There can only be one line of either this or the previous form.
- `Subcase(...)`: this instructs the program to split its output into subcases. It accepts as arguments a list of any of the first four instructions in this list.

## Enforcing manual bounds
In several places, bounds are manually added to speed up computation, with proofs in the paper referenced above. These are:
- `0DAA`: we enforce `InitialSumUpperBound(4, 1.0)`
- `0FAA`: we enforce `InitialSumUpperBound(3, 1.0)` and `MidSumUpperBound(1, 5, 1.0)`
- `0GA`: we enforce `MidSumUpperBound(2, 6, 1.0)` and `MidSumUpperBound(1, 4, 1.0)`
- `0CA` and `0CB`: we enforce `InitialSumUpperBound(3, 1.0)`
