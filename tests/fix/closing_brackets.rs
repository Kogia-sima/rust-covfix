fn main() {
    if 1 > 0 {
        if 2 > 1 {
            println!("reachable");
        } else if 3 > 2 {
            println!("unreachable");
        } else {
            println!("unreachable");
        }
    };
}
