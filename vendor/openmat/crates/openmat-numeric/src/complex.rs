use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

/// Double-precision complex scalar; no native numerical dependency.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}
impl Complex {
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn exp(self) -> Self {
        if self.im == 0. {
            return Self::new(self.re.exp(), 0.);
        }
        let r = self.re.exp();
        Self::new(r * self.im.cos(), r * self.im.sin())
    }
    pub fn log(self) -> Self {
        if self.im == 0. && (self.re >= 0. || self.re.is_nan()) {
            return Self::new(self.re.ln(), 0.);
        }
        Self::new(self.abs().ln(), self.im.atan2(self.re))
    }
    pub fn sqrt(self) -> Self {
        if self.im == 0.0 {
            return if self.re < 0.0 {
                Self::new(0., (-self.re).sqrt())
            } else {
                Self::new(self.re.sqrt(), self.im)
            };
        }
        // Scale before combining to avoid overflow for finite large components.
        let scale = self.re.abs().max(self.im.abs());
        let t = scale.sqrt()
            * (((self.re / scale).hypot(self.im / scale) + self.re.abs() / scale) / 2.).sqrt();
        if self.re >= 0. {
            Self::new(t, self.im / (2. * t))
        } else {
            Self::new(self.im.abs() / (2. * t), t.copysign(self.im))
        }
    }
    pub fn sin(self) -> Self {
        if self.im == 0. {
            return Self::new(self.re.sin(), 0.);
        }
        Self::new(
            self.re.sin() * self.im.cosh(),
            self.re.cos() * self.im.sinh(),
        )
    }
    pub fn cos(self) -> Self {
        if self.im == 0. {
            return Self::new(self.re.cos(), 0.);
        }
        Self::new(
            self.re.cos() * self.im.cosh(),
            -self.re.sin() * self.im.sinh(),
        )
    }
    pub fn pow(self, rhs: Self) -> Self {
        if self.im == 0. && rhs.im == 0. && (self.re >= 0. || rhs.re.fract() == 0.) {
            return Self::new(self.re.powf(rhs.re), 0.);
        }
        if rhs.im == 0. && rhs.re == 0. {
            return Self::new(1., 0.);
        }
        (rhs * self.log()).exp()
    }
}
impl Add for Complex {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.re + b.re, self.im + b.im)
    }
}
impl Sub for Complex {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.re - b.re, self.im - b.im)
    }
}
impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.re, -self.im)
    }
}
impl Mul for Complex {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        if self.im == 0. && b.im == 0. {
            return Self::new(self.re * b.re, 0.);
        }
        Self::new(
            self.re * b.re - self.im * b.im,
            self.re * b.im + self.im * b.re,
        )
    }
}
impl Div for Complex {
    type Output = Self;
    fn div(self, b: Self) -> Self {
        if self.im == 0. && b.im == 0. {
            return Self::new(self.re / b.re, 0.);
        }
        // Normalize the divisor first, avoiding squaring its magnitude.
        let scale = b.re.abs().max(b.im.abs());
        let r = b.re / scale;
        let i = b.im / scale;
        let d = r * r + i * i;
        Self::new(
            (self.re / d / scale) * r + (self.im / d / scale) * i,
            (self.im / d / scale) * r - (self.re / d / scale) * i,
        )
    }
}
impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im == 0. {
            write!(f, "{}", self.re)
        } else {
            write!(f, "{}{:+}i", self.re, self.im)
        }
    }
}
