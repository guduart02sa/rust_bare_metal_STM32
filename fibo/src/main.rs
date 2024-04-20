use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

/* Using the clap documentation, modify your application to work according to the following schema:


Compute Fibonacci suite values

Usage: fibo [OPTIONS] <VALUE>

Arguments:
  <VALUE>  The maximal number to print the fibo value of

Options:
  -v, --verbose       Print intermediate values
  -m, --min <NUMBER>  The minimum number to compute
  -h, --help          Print help */

struct Args {
    /// The maximal number to print the fibo value of
    #[arg(required = true)]
    val: u32,

    /// Print intermediate values
    #[arg(short, long)]
    verbose: bool,

    /// The minimum number to compute
    #[arg(short, long)]
    min: Option<u32>,
/* 
    /// Print help
    #[arg(short, long)]
    help: bool,*/
}
/*
fn fibo_recursive(n: u32) -> u32 {
    if n == 1 {
        return 1;
    }
    if n == 0 {
        return 0;
    }
    fibo_recursive(n - 1) + fibo_recursive(n - 2)
} */

fn fibo_iterative(n: u32) -> Option<u32> {
    if n <= 1 {
        Some(n)
    } else if n == 2 {
        Some(1)
    } else {
        let mut current: u32 = 2;
        let mut previous: u32 = 1;
        let mut next;
        for _i in 3..n {
            if let Some(result) = previous.checked_add(current) {
                next = result;
            } else {
                return None;
            }
            previous = current;
            current = next;
        }
        return Some(current);
    }
}

/*
fn main() {
    for a in 0..50 {
        if let Some(result) = fibo_iterative(a) {
            println!("Fibonacci_it({}) = {:?}", a, result);
        } else {
            println!("Fibonacci_it({}) = None", a);
        }
        println!("Fibonacci_rec({}) = {}", a, fibo_recursive(a));
        println!("------------")
    }
}
*/

fn main() {
    let args = Args::parse();
    /*    if args.help {
        println!("{}", Args::clap().get_matches().usage());
        return;
    } */
    let min = args.min.unwrap_or(0);
    for a in min..args.val + 1 {
        if let Some(result) = fibo_iterative(a) {
            if args.verbose {
                println!("Fibonacci_it({}) = {}", a, result);
            } else {
                if a == args.val {
                    println!("Fibonacci_it({}) = {}", a, result);
                }
            }
        } else {
            println!("Fibonacci_it({}) = None", a);
        }
    }
}
