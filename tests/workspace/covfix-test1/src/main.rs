fn main() {
    for i in 1..=100 {
        match i % 15 {
            3 | 6 | 9 | 12 => println!("Fizz"),
            5 | 10 => println!("Buzz"),
            0 => println!("FizzBuzz"),
            i => println!("{}", i),
        }
    }
}
