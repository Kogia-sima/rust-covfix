fn main() {
    let a = 1 + 2; // cov:ignore-line

    // cov:begin-ignore-branch
    println!("Hello");
    println!("world!");
    // cov:end-ignore-branch

    // cov: begin-ignore-line
    if a > 2 {
        println!("a is large!");
    } else if a == 0 {
        println!("a is small!");
    }
    // cov: end-ignore-line

    // cov:begin-ignore
    println!("a = {}", a);
    // cov:end-ignore

    println!("finish."); // cov:ignore-branch

    return (); // cov:ignore
}
