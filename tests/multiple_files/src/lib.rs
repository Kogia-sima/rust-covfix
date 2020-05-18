mod foo;

fn hello() {
    println!("Hello!");
    foo::hello();
}

#[test]
fn test() {
    hello();
}
