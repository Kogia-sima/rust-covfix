fn main() {
    for i in 0..10 {
        println!("{}", i);
    }

    // do not have to fix this loop
    loop {
        break;
    }

    let mut i = 0;

    // do not have to fix this loop
    while i < 10 {
        i += 1;
    }
}

struct A;

impl Default for A {
    fn default() -> Self {
        Self
    }
}
