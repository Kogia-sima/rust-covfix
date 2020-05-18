trait Abs {
    fn abs(&self) -> Self;
}

impl Abs for i32 {
    fn abs(&self) -> i32 {
        if self < 0 {
            -self
        } else {
            self
        }
    }
}

impl Abs for u32 {
    fn abs(&self) -> u32 {
        if self < 0 {
            unreachable!();
        } else {
            self
        }
    }
}
