use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Mul, Sub},
};

/// The simulation time unit. One jiffy is the smallest discrete time step.
#[derive(PartialEq, PartialOrd, Ord, Eq, Copy, Clone, Default)]
pub struct Jiffies(pub usize);

impl Add for Jiffies {
    type Output = Jiffies;

    fn add(self, rhs: Self) -> Self::Output {
        Jiffies(self.0 + rhs.0)
    }
}

impl Sub for Jiffies {
    type Output = Jiffies;

    fn sub(self, rhs: Self) -> Self::Output {
        Jiffies(self.0 - rhs.0)
    }
}

impl AddAssign<Jiffies> for Jiffies {
    fn add_assign(&mut self, rhs: Jiffies) {
        self.0 += rhs.0
    }
}

impl AddAssign<usize> for Jiffies {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

impl Mul<Jiffies> for usize {
    type Output = Jiffies;

    fn mul(self, rhs: Jiffies) -> Self::Output {
        Jiffies(self * rhs.0)
    }
}

impl Display for Jiffies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str((self.0.to_string() + " jfs").as_str())
    }
}

impl Debug for Jiffies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}
