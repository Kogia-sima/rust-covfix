pub fn add_two(a: i32) -> i32 {
    a + 2
}

#[cfg(test)]
#[allow(dead_code)]

mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, add_two(2));
    }
}

// should not fix the coverage
mod tests {
    use super::*;

    fn it_works() {
        assert_eq!(4, add_two(2));
    }
}

fn function_for_test() -> i32 {
    let mut a = 1;

    return a;
}

#[test]
fn foo() {
    println!("foo");
}

#[cfg(not(test))]
mod imp {
    fn hello() {
        println!("Hello, world!");
    }
}
