#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

union ieee_float_shape_type {
    value: f32,
    word: u32,
}

macro_rules! get_float_word {
    ($i:ident, $d:expr) => {
        unsafe {
            let gf_u = ieee_float_shape_type { value: $d };
            $i = gf_u.word;
        }
    };
}

macro_rules! set_float_word {
    ($d:ident, $i:expr) => {
        unsafe {
            let sf_u = ieee_float_shape_type { word: $i };
            $d = sf_u.value;
        }
    };
}

const one: f32 = 1.0;
const halF: [f32; 2] = [0.5, -0.5];
const huge: f32 = 1.0e+30;
const twom100: f32 = 7.8886090522e-31;
const o_threshold: f32 = 8.8721679688e+01;
const u_threshold: f32 = -1.0397208405e+02;
const ln2HI: [f32; 2] = [6.9314575195e-01, -6.9314575195e-01];
const ln2LO: [f32; 2] = [1.4286067653e-06, -1.4286067653e-06];
const invln2: f32 = 1.4426950216e+00;
const P1: f32 = 1.6666667163e-01;
const P2: f32 = -2.7777778450e-03;
const P3: f32 = 6.6137559770e-05;
const P4: f32 = -1.6533901999e-06;
const P5: f32 = 4.1381369442e-08;

pub fn ieee754_expf(mut x: f32) -> f32 {
    let mut hi = 0.0f32;
    let mut lo = 0.0f32;
    let mut k: i32 = 0;
    let mut hx: u32;
    get_float_word!(hx, x);

    let xsb = (hx as usize >> 31) & 1;
    hx &= 0x7fffffff;

    if hx >= 0x42b17218 {
        if hx > 0x7f800000 {
            return x + x;
        }
        if hx == 0x7f800000 {
            return if xsb == 0 { x } else { 0.0 };
        }
        if x > o_threshold {
            return huge * huge;
        }
        if x < u_threshold {
            return twom100 * twom100;
        }
    }

    if hx > 0x3eb17218 {
        if hx < 0x3F851592 {
            hi = x - ln2HI[xsb];
            lo = ln2LO[xsb];
            k = 1 - (xsb << 1) as i32;
        } else {
            k = (invln2 * x + halF[xsb]) as i32;
            let t = k as f32;
            hi = x - t * ln2HI[0];
            lo = t * ln2LO[0];
        }
        x = hi - lo;
    } else if hx < 0x31800000 {
        if huge + x > one {
            return one + x;
        }
    } else {
        k = 0;
    }

    let t = x * x;
    let c = x - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));

    let mut y = match k {
        0 => return one - ((x * c) / (c - 2.0f32) - x),
        _ => {
            one - ((lo - (x * c) / (2.0f32 - c)) - hi)
        }
    };

    if k >= -125 {
        let hy: u32;
        get_float_word!(hy, y);
        set_float_word!(y, hy.wrapping_add((k as u32) << 23));
        return y;
    } else {
        let hy: u32;
        get_float_word!(hy, y);
        set_float_word!(y, hy.wrapping_add((k as u32 + 100) << 23));
        return y * twom100;
    }
}
