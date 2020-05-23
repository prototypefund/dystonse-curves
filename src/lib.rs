use std::ops::{Add, Sub, Mul, Div};
use std::cmp::{PartialOrd};
use fixed::types::{U1F7, U1F15};
use fixed::traits::{LossyFrom};
use half::prelude::*;

/** 
 * This is an implementation of https://xkcd.com/927/ for 
 * converting number-like values to/from f32.
 * 
 * Many different types behave like floating point numbers,
 * but they don't support all the operators that f32 has,
 * and their conversion methods are different. Due to 
 * ophanage rules, we can't just implement one of the 
 * exisitng conversion traits for all those types - we
 * have to invent our own.
*/
pub trait ConvertF32 {
    fn make_into_f32(self) -> f32;
    fn make_from_f32(value: f32) -> Self;
}

/** Trivial "conversion" from f32 to f32. */
impl ConvertF32 for f32 {
    fn make_into_f32(self) -> f32 {
        return self;
    }

    fn make_from_f32(value: f32) -> Self {
        return value;
    }
}

impl ConvertF32 for U1F7 {
    fn make_into_f32(self) -> f32 {
        return f32::lossy_from(self);
    }

    fn make_from_f32(value: f32) -> Self {
        return U1F7::from_num(value);
    }
}

impl ConvertF32 for U1F15 {
    fn make_into_f32(self) -> f32 {
        return f32::lossy_from(self);
    }

    fn make_from_f32(value: f32) -> Self {
        return U1F15::from_num(value);
    }
}

impl ConvertF32 for f16 {
    fn make_into_f32(self) -> f32 {
        return self.to_f32();
    }

    fn make_from_f32(value: f32) -> Self {
        return f16::from_f32(value);
    }
}

impl ConvertF32 for i8 {
    fn make_into_f32(self) -> f32 {
        return self.into();
    }

    fn make_from_f32(value: f32) -> Self {
        return value as i8;
    }
}

/**
 * Trait to access the curve's values using f32 as type for X 
 * and Y, irrespective of the types used internally.
 */
pub trait Curve
{
    fn min_x(&self) -> f32;
    fn max_x(&self) -> f32;
    fn y_at_x(&self, x: f32) -> f32;
    fn x_at_y(&self, y: f32) -> f32;
    fn new(n: usize, s: f32, x0: f32, y: Vec<f32>) -> Self;
}

/**
 * Trait to acces the curve's values using the types which are
 * used internally to store the data.
 */
pub trait TypedCurve<X, Y>
{
    fn typed_min_x(&self) -> X;
    fn typed_max_x(&self) -> X;
    fn typed_y_at_x(&self, x: X) -> Y;
    fn typed_x_at_y(&self, y: Y) -> X;
    fn typed_new(n: usize, s: X, x0: X, y: Vec<Y>) -> Self;
}

/**
 * A curve that has a dynamic length and data points at regular distances.
 */
pub struct RegularDynamicCurve<X, Y> 
    where X: ConvertF32,
          Y: ConvertF32 {
    n: usize,
    s: X,
    x0: X,
    y: Vec<Y>
}

impl<X, Y> Curve for RegularDynamicCurve<X, Y>
where X: ConvertF32 + Copy + Sub<X, Output = X> + Add<X, Output = X> + Div<X, Output = X> + Mul<X, Output = X> + PartialOrd,
Y: ConvertF32 + Copy
{
    fn new(n: usize, s: f32, x0: f32, y: Vec<f32>) -> Self {
        return Self{
            n,
            s: X::make_from_f32(s),
            x0: X::make_from_f32(x0),
            y: y.iter().map(|yp| Y::make_from_f32(*yp)).collect()
        };
    }

    fn min_x(&self) -> f32 {
        return self.x0.make_into_f32();
    }

    fn max_x(&self) -> f32
    {
        let len = self.s.make_into_f32() * ((self.n - 1) as f32);
        return self.x0.make_into_f32() + len;
    }

    fn y_at_x(&self, x: f32) -> f32 {
        if x <= self.min_x() {
            return self.y[0].make_into_f32();
        }
        if x >= self.max_x() {
            return self.y[self.n - 1].make_into_f32();
        }

        let i = (x - self.min_x()) / self.s.make_into_f32();
       
        let i_min = i.floor() as usize;
        let i_max = i.ceil() as usize;

        if i_max == i_min {
            return self.y[i_min].make_into_f32();
        }

        let a = i.fract();
        return self.y[i_min].make_into_f32() * (1.0 - a) + 
               self.y[i_max].make_into_f32() * a;
    }

    /**
     * TODO when multiple consecutive points have the given Y value, the first X value will be returned. 
     * We could as well return the last one, or the center, or whatever.
     */
    fn x_at_y(&self, y: f32) -> f32 {
        assert!(y >= 0.0);
        assert!(y <= 1.0);

        if y == 0.0 {
            return self.min_x();
        }

        if y == 1.0 {
            return self.max_x();
        }

        for i in 0..self.n {
            let v_r = self.y[i].make_into_f32();
            if v_r == y {
                return self.min_x() + i as f32 * self.s.make_into_f32();
            }
            if v_r > y {
                assert!(i > 0);
                let v_l = self.y[i - 1].make_into_f32();
                let a = (y - v_l) / (v_r - v_l);
                return self.min_x() + ((i - 1) as f32 + a) * self.s.make_into_f32();
            }
        }

        panic!("Did not find y = {}", y);
    }
}

impl<X, Y> TypedCurve<X, Y> for RegularDynamicCurve<X, Y>
    where X: ConvertF32 + Copy + Sub<X, Output = X> + Add<X, Output = X> + Div<X, Output = X> + Mul<X, Output = X> + PartialOrd,
          Y: ConvertF32 + Copy
{
    fn typed_new(n: usize, s: X, x0: X, y: Vec<Y>) -> Self {
        return Self{
            n,s,x0,y
        };
    }

    fn typed_min_x(&self) -> X {
        return self.x0;
    }

    fn typed_max_x(&self) -> X
    {
        let len : X = self.s * X::make_from_f32((self.n - 1) as f32);
        return self.x0 + len;
    }

    fn typed_y_at_x(&self, x: X) -> Y {
        if x <= self.x0 {
            return self.y[0];
        }
        if x >= self.typed_max_x() {
            return self.y[self.n - 1];
        }

        let i = X::make_into_f32(x - self.x0) / X::make_into_f32(self.s);
       
        let i_min = i.floor() as usize;
        let i_max = i.ceil() as usize;

        if i_max == i_min {
            return self.y[i_min];
        }

        let a = i.fract();
        return Y::make_from_f32(self.y[i_min].make_into_f32() * (1.0 - a) + 
                                self.y[i_max].make_into_f32() * a);
    }

    /**
     * TODO when multiple consecutive points have the given Y value, the first X value will be returned. 
     * We could as well return the last one, or the center, or whatever.
     */
    fn typed_x_at_y(&self, y: Y) -> X {
        let yf = y.make_into_f32();
        assert!(yf >= 0.0);
        assert!(yf <= 1.0);

        if yf == 0.0 {
            return self.typed_min_x();
        }

        if yf == 1.0 {
            return self.typed_max_x();
        }

        for i in 0..self.n {
            let v_r = self.y[i].make_into_f32();
            if v_r == yf {
                return X::make_from_f32(self.x0.make_into_f32() + i as f32 * self.s.make_into_f32());
            }
            if v_r > yf {
                assert!(i > 0);
                let v_l = self.y[i - 1].make_into_f32();
                let a = (yf - v_l) / (v_r - v_l);
                return X::make_from_f32(self.x0.make_into_f32() + ((i - 1) as f32 + a) * self.s.make_into_f32());
            }
        }

        panic!("Did not find y = {}", yf);
    }
}

// TODO Move tests into own file?
// TODO Test multiple consecutive points with the same value
// TODO split test functions
#[cfg(test)]
mod tests {
    use crate::{Curve, TypedCurve, RegularDynamicCurve, ConvertF32};
    use assert_approx_eq::assert_approx_eq;
    use fixed::types::{U1F7, U1F15};
    use half::prelude::*;

    #[test]
    fn test_all() {
        test_curve::<RegularDynamicCurve<f32,   f32>, f32,   f32>(true , 0.000001);
        test_curve::<RegularDynamicCurve< i8,   f32>,  i8,   f32>(false, 0.000001);
        test_curve::<RegularDynamicCurve<f32,  U1F7>, f32,  U1F7>(true , 0.05);
        test_curve::<RegularDynamicCurve<f32, U1F15>, f32, U1F15>(true , 0.0005);
        test_curve::<RegularDynamicCurve<f32,   f16>, f32,   f16>(true , 0.005);
    }

    fn test_curve<T, X, Y>(test_float_x: bool, epsilon: f32) 
    where X: ConvertF32,
          Y: ConvertF32,
          T: Curve + TypedCurve<X, Y>
        {
            test_curve_typed::<T, X, Y>(test_float_x, epsilon);
            test_curve_untyped::<T, X, Y>(test_float_x, epsilon);
        }

    fn test_curve_untyped<T, X, Y>(test_float_x: bool, epsilon: f32) 
        where X: ConvertF32,
            Y: ConvertF32,
            T: Curve
        {
        let c : T = T::new(
            3,
            10.0,
            10.0,
            vec!{
                0.0, 
                0.6, 
                1.0}
        );

        // Test x bounds
        assert_eq!(c.min_x(), 10.0);
        assert_eq!(c.max_x(), 30.0);

        // Test x outside of bounds
        assert_eq!(c.y_at_x(0.0), 0.0);
        assert_eq!(c.y_at_x(100.0), 1.0);

        // Test x equal to the actual points
        assert_approx_eq!(c.y_at_x(10.0), 0.0, epsilon);
        assert_approx_eq!(c.y_at_x(20.0), 0.6, epsilon);
        assert_approx_eq!(c.y_at_x(30.0), 1.0, epsilon);

        // Test arbitrary "integer" x values
        assert_approx_eq!(c.y_at_x(15.0), 0.3, epsilon);
        assert_approx_eq!(c.y_at_x(25.0), 0.8, epsilon);

        if test_float_x {
            // Test arbitrary "float" x values
            assert_approx_eq!(c.y_at_x(12.5), 0.15, epsilon);
            assert_approx_eq!(c.y_at_x(17.5), 0.45, epsilon);
        }

        // Test y queries
        assert_approx_eq!(c.x_at_y(0.0), 10.0, epsilon);
        assert_approx_eq!(c.x_at_y(1.0), 30.0, epsilon);
        assert_approx_eq!(c.x_at_y(0.6), 20.0, epsilon);
        if test_float_x {
            assert_approx_eq!(c.x_at_y(0.15), 12.5, epsilon);
            assert_approx_eq!(c.x_at_y(0.45), 17.5, epsilon);
        }
    }

    fn test_curve_typed<T, X, Y>(test_float_x: bool, epsilon: f32) 
        where X: ConvertF32,
              Y: ConvertF32,
              T: TypedCurve<X, Y>
        {
        let c : T = T::typed_new(
            3,
            X::make_from_f32(10.0),
            X::make_from_f32(10.0),
            vec!{
                Y::make_from_f32(0.0), 
                Y::make_from_f32(0.6), 
                Y::make_from_f32(1.0)}
        );

        // Test x bounds
        assert_eq!(c.typed_min_x().make_into_f32(), 10.0);
        assert_eq!(c.typed_max_x().make_into_f32(), 30.0);

        // Test x outside of bounds
        assert_eq!(c.typed_y_at_x(X::make_from_f32(0.0)).make_into_f32(), 0.0);
        assert_eq!(c.typed_y_at_x(X::make_from_f32(100.0)).make_into_f32(), 1.0);

        // Test x equal to the actual points
        assert_approx_eq!(c.typed_y_at_x(X::make_from_f32(10.0)).make_into_f32(), 0.0, epsilon);
        assert_approx_eq!(c.typed_y_at_x(X::make_from_f32(20.0)).make_into_f32(), 0.6, epsilon);
        assert_approx_eq!(c.typed_y_at_x(X::make_from_f32(30.0)).make_into_f32(), 1.0, epsilon);

        // Test arbitrary "integer" x values
        assert_approx_eq!(c.typed_y_at_x(X::make_from_f32(15.0)).make_into_f32(), 0.3, epsilon);
        assert_approx_eq!(c.typed_y_at_x(X::make_from_f32(25.0)).make_into_f32(), 0.8, epsilon);

        if test_float_x {
            // Test arbitrary "float" x values
            assert_approx_eq!(c.typed_y_at_x(X::make_from_f32(12.5)).make_into_f32(), 0.15, epsilon);
            assert_approx_eq!(c.typed_y_at_x(X::make_from_f32(17.5)).make_into_f32(), 0.45, epsilon);
        }

        // Test y queries
        assert_approx_eq!(c.typed_x_at_y(Y::make_from_f32(0.0)).make_into_f32(), 10.0, epsilon);
        assert_approx_eq!(c.typed_x_at_y(Y::make_from_f32(1.0)).make_into_f32(), 30.0, epsilon);
        assert_approx_eq!(c.typed_x_at_y(Y::make_from_f32(0.6)).make_into_f32(), 20.0, epsilon);
        if test_float_x {
            assert_approx_eq!(c.typed_x_at_y(Y::make_from_f32(0.15)).make_into_f32(), 12.5, epsilon);
            assert_approx_eq!(c.typed_x_at_y(Y::make_from_f32(0.45)).make_into_f32(), 17.5, epsilon);
        }
    }
}
