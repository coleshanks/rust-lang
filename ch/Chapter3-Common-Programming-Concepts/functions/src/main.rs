fn main() {
    println!("Hello, world!");

    another_function();
    
    let x = five();
    println!("The value of x is: {x}");

}

fn another_function() {
    println!("Another function.");
}

fn five() -> i32 {
    5
}
