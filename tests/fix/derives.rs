#[derive(Clone)]
pub struct Point {
    x: f64,
    y: f64
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point {
            x,
            y
        }
    }
}

#[derive(PartialEq)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn r() -> u8 {
        self.0
    }

    pub fn g() -> u8 {
        self.1
    }

    pub fn b() -> u8 {
        self.2
    }
}

#[derive(Serialize)]
#[serde(rename = "e")]
enum E {
    #[serde(rename = "a")]
    A(String)
}

impl E {
    pub fn new(s: String) -> Self {
        Self::A(s)
    }
}

#[derive(Copy, Clone)]
union MyUnion {
    f1: u32,
    f2: f32,
}

#[derive(Clone, Copy)]
struct UnitType;

enum EmptyEnum {}
