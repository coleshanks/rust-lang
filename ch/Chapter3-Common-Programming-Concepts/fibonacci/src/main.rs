use std::io;

fn main() {
    println!("This program will generate the nth Fibonacci number");
    // initialize a Vec to hold the fibonacci sequence
    let mut fibonacci: Vec<u128> = vec![0, 1];

    println!("Please enter a number");

    let n: usize = loop {
        // user inputs the number so we start with an empty string
        let mut input = String::new();

        // read user input from stdin
        io::stdin()
            .read_line(&mut input) // save user input
            .expect("Failed to read line");

        // Err handling for invalid inputs
        match input.trim().parse() {
            Ok(num) if num > 0 => break num,
            Ok(_) => println!("Please enter a number greater than 0"),
            Err(_) => println!("Invalid input, please enter a valid number"),
        }
    };

    //if n == 1 we dont run the loop and return early
    if n == 1 {
        println!("-----");
        println!("0");
        println!("\nThe nth fibonacci number is: 0!");
        return;
    }

    // initialize count to 2 because [0, 1] are already there
    let mut count: usize = 2;

    println!("-----");
    println!("0");
    println!("1");

    // Add new fibonacci numbers to the Vec
    while count < n {
        fibonacci.push(fibonacci[count - 2] + fibonacci[count - 1]);
        count = count + 1;

        println!("{}", fibonacci[count - 1]);
    }

    let nth_number = fibonacci[n - 1];

    println!("\nThe nth fibonacci number is: {nth_number}!");
}
