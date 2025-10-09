use num::{Float, Signed};
use std::ops::*;
use std::iter::Sum;
use num::traits::{NumAssign, One, Zero};
pub use uuid::Uuid;

pub mod arena;
pub mod phys;

#[repr(transparent)]
pub struct Vector<T, const N: usize>(
    pub [T; N],
);

impl<T, const N: usize> Copy for Vector<T, N>
where
    T: Copy,
{}

impl<T, const N: usize> Clone for Vector<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Vector(self.0.clone())
    }
}

impl<T, const N: usize> Vector<T, N> {
    pub fn map<F, U>(self, f: F) -> Vector<U, N>
    where
        F: FnMut(T) -> U,
    {
        Vector(self.0.map(f))
    }

    pub fn mul<Rhs>(self, rhs: Rhs) -> Vector<<T as Mul<Rhs>>::Output, N>
    where
        Rhs: Copy,
        T: Mul<Rhs>,
    {
        self.map(|lhs| lhs * rhs)
    }

    pub fn div<Rhs>(self, rhs: Rhs) -> Vector<<T as Div<Rhs>>::Output, N>
    where
        Rhs: Copy,
        T: Div<Rhs>,
    {
        self.map(|lhs| lhs / rhs)
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Copy + Zero,
{
    pub fn zero() -> Self {
        Self([T::zero(); N])
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Copy + One,
{
    pub fn one() -> Self {
        Self([T::one(); N])
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Copy,
{
    pub const fn filled(value: T) -> Self {
        Self([value; N])
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Mul,
{
    pub fn dot<U>(self, rhs: Self) -> U
    where
        U: Sum<<T as Mul>::Output>,
    {
        std::iter::zip(self.0, rhs.0).map(|(a, b)| a * b).sum()
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Copy + Float + Sum,
{
    pub fn magnitude(&self) -> T {
        self.dot::<T>(*self).sqrt()
    }

    pub fn normalized(&self) -> Self {
        self.div(self.magnitude())
    }

    pub fn equals_delta(&self, other: Self, delta: Self) -> bool {
        std::iter::zip(&self.0, &other.0)
            .zip(&delta.0)
            .all(|((&a, &b), &d)| (a - b).abs() <= d)
    }

    pub fn lerp(&self, other: Self, t: T) -> Self {
        let mut result = Vector::zero();
        for index in 0..N {
            result.0[index] = self.0[index] * (T::one() - t) + other.0[index] * t;
        }
        result
    }
}

impl<T> Vector<T, 2>
where
    T: Copy,
{
    pub const fn x(&self) -> T {
        self.0[0]
    }

    pub const fn set_x(&mut self, x: T) {
        self.0[0] = x;
    }

    pub const fn y(&self) -> T {
        self.0[1]
    }

    pub const fn set_y(&mut self, y: T) {
        self.0[1] = y;
    }

    pub const fn with_z(&self, z: T) -> Vector<T, 3> {
        Vector([self.x(), self.y(), z])
    }
}

impl<T> Vector<T, 3>
where
    T: Copy,
{
    pub const fn x(&self) -> T {
        self.0[0]
    }

    pub const fn set_x(&mut self, x: T) {
        self.0[0] = x;
    }

    pub const fn y(&self) -> T {
        self.0[1]
    }

    pub const fn set_y(&mut self, y: T) {
        self.0[1] = y;
    }

    pub const fn z(&self) -> T {
        self.0[2]
    }

    pub const fn set_z(&mut self, z: T) {
        self.0[2] = z;
    }

    pub const fn xy(&self) -> Vector<T, 2> {
        Vector([self.x(), self.y()])
    }

    pub const fn with_w(&self, w: T) -> Vector<T, 4> {
        Vector([self.x(), self.y(), self.z(), w])
    }
}

impl<T> Vector<T, 3>
where
    T: Copy + Mul,
    <T as Mul>::Output: Sub,
{
    pub fn cross(&self, rhs: Self) -> Vector<<<T as Mul>::Output as Sub>::Output, 3> {
        Vector([
            self.y() * rhs.z() - self.z() * rhs.y(),
            self.z() * rhs.x() - self.x() * rhs.z(),
            self.x() * rhs.y() - self.y() * rhs.x(),
        ])
    }
}

impl<T> Vector<T, 4>
where
    T: Copy,
{
    pub const fn x(&self) -> T {
        self.0[0]
    }

    pub const fn set_x(&mut self, x: T) {
        self.0[0] = x;
    }

    pub const fn y(&self) -> T {
        self.0[1]
    }

    pub const fn set_y(&mut self, y: T) {
        self.0[1] = y;
    }

    pub const fn z(&self) -> T {
        self.0[2]
    }

    pub const fn set_z(&mut self, z: T) {
        self.0[2] = z;
    }

    pub const fn w(&self) -> T {
        self.0[3]
    }

    pub const fn set_w(&mut self, w: T) {
        self.0[3] = w;
    }

    pub const fn xyz(&self) -> Vector<T, 3> {
        Vector([self.x(), self.y(), self.z()])
    }
}

impl<T, const N: usize> PartialEq for Vector<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, const N: usize> PartialOrd for Vector<T, N>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: NumAssign + Copy, const N: usize> Eq for Vector<T, N>
where
    T: Eq,
{}

impl<T: NumAssign + Copy, const N: usize> Ord for Vector<T, N>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, const N: usize, I> Index<I> for Vector<T, N>
where
    [T; N]: Index<I>,
{
    type Output = <[T; N] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T, const N: usize, I> IndexMut<I> for Vector<T, N>
where
    [T; N]: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T, const N: usize> Neg for Vector<T, N>
where
    T: Neg,
{
    type Output = Vector<<T as Neg>::Output, N>;

    fn neg(self) -> Self::Output {
        Vector(self.0.map(|x| -x))
    }
}

impl<T, const N: usize, Rhs> Add<Vector<Rhs, N>> for Vector<T, N>
where
    T: Add<Rhs>,
{
    type Output = Vector<<T as Add<Rhs>>::Output, N>;

    fn add(self, rhs: Vector<Rhs, N>) -> Self::Output {
        let mut pairs = std::iter::zip(self.0, rhs.0);
        Vector(std::array::from_fn(|_| {
            // SAFETY: The size of both input arrays and the output array is N at compile time
            let (lhs, rhs) = unsafe { pairs.next().unwrap_unchecked() };
            lhs + rhs
        }))
    }
}

impl<T, const N: usize, Rhs> AddAssign<Vector<Rhs, N>> for Vector<T, N>
where
    T: AddAssign<Rhs>,
{
    fn add_assign(&mut self, rhs: Vector<Rhs, N>) {
        for (lhs, rhs) in std::iter::zip(&mut self.0, rhs.0) {
            *lhs += rhs;
        }
    }
}

impl<T, const N: usize, Rhs> Sub<Vector<Rhs, N>> for Vector<T, N>
where
    T: Sub<Rhs>,
{
    type Output = Vector<<T as Sub<Rhs>>::Output, N>;

    fn sub(self, rhs: Vector<Rhs, N>) -> Self::Output {
        let mut pairs = std::iter::zip(self.0, rhs.0);
        Vector(std::array::from_fn(|_| {
            // SAFETY: The size of both input arrays and the output array is N at compile time
            let (lhs, rhs) = unsafe { pairs.next().unwrap_unchecked() };
            lhs - rhs
        }))
    }
}

impl<T, const N: usize, Rhs> SubAssign<Vector<Rhs, N>> for Vector<T, N>
where
    T: SubAssign<Rhs>,
{
    fn sub_assign(&mut self, rhs: Vector<Rhs, N>) {
        for (lhs, rhs) in std::iter::zip(&mut self.0, rhs.0) {
            *lhs -= rhs;
        }
    }
}

impl<T, const N: usize, Rhs> Mul<Vector<Rhs, N>> for Vector<T, N>
where
    T: Mul<Rhs>,
{
    type Output = Vector<<T as Mul<Rhs>>::Output, N>;

    fn mul(self, rhs: Vector<Rhs, N>) -> Self::Output {
        let mut pairs = std::iter::zip(self.0, rhs.0);
        Vector(std::array::from_fn(|_| {
            // SAFETY: The size of both input arrays and the output array is N at compile time
            let (lhs, rhs) = unsafe { pairs.next().unwrap_unchecked() };
            lhs * rhs
        }))
    }
}

impl<T, const N: usize, Rhs> MulAssign<Vector<Rhs, N>> for Vector<T, N>
where
    T: MulAssign<Rhs>,
{
    fn mul_assign(&mut self, rhs: Vector<Rhs, N>) {
        for (lhs, rhs) in std::iter::zip(&mut self.0, rhs.0) {
            *lhs *= rhs;
        }
    }
}

impl<T, const N: usize, Rhs> Div<Vector<Rhs, N>> for Vector<T, N>
where
    T: Div<Rhs>,
{
    type Output = Vector<<T as Div<Rhs>>::Output, N>;

    fn div(self, rhs: Vector<Rhs, N>) -> Self::Output {
        let mut pairs = std::iter::zip(self.0, rhs.0);
        Vector(std::array::from_fn(|_| {
            // SAFETY: The size of both input arrays and the output array is N at compile time
            let (lhs, rhs) = unsafe { pairs.next().unwrap_unchecked() };
            lhs / rhs
        }))
    }
}

impl<T, const N: usize, Rhs> DivAssign<Vector<Rhs, N>> for Vector<T, N>
where
    T: DivAssign<Rhs>,
{
    fn div_assign(&mut self, rhs: Vector<Rhs, N>) {
        for (lhs, rhs) in std::iter::zip(&mut self.0, rhs.0) {
            *lhs /= rhs;
        }
    }
}

impl<T, const N: usize> std::fmt::Debug for Vector<T, N>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector{:?}", self.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Matrix<T: Float + NumAssign, const R: usize, const C: usize>(
    pub [Vector<T, R>; C],
);

#[allow(type_alias_bounds)]
pub type Transform2D<T: Float + NumAssign> = Matrix<T, 3, 3>;

#[allow(type_alias_bounds)]
pub type Transform3D<T: Float + NumAssign> = Matrix<T, 4, 4>;

impl<T: Float + NumAssign, const R: usize, const C: usize> Matrix<T, R, C> {
    pub fn zero() -> Self {
        Matrix([Vector::zero(); C])
    }

    pub fn content(&self) -> [Vector<T, R>; C] {
        self.0
    }

    pub fn at(&self, row: usize, col: usize) -> T {
        self.0[col][row]
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) {
        self.0[col][row] = value;
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr() as *const T
    }

    pub fn fill_zero(&mut self) {
        self.0.fill(Vector::zero());
    }

    pub fn swap_rows(&mut self, row1: usize, row2: usize) {
        for col in 0..C {
            self[col].0.swap(row1, row2);
        }
    }

    pub fn mul_row(&mut self, row: usize, value: T) {
        for col in 0..C {
            self[col][row] *= value;
        }
    }

    pub fn div_row(&mut self, row: usize, value: T) {
        for col in 0..C {
            self[col][row] /= value;
        }
    }

    pub fn add_row(&mut self, row: usize, from: usize, mul: T) {
        for col in 0..C {
            let to_add = self[col][from] * mul;
            self[col][row] += to_add;
        }
    }

    pub fn rref(&self) -> Self {
        let mut output = *self;
        let mut target_row: usize = 0;
        for col in 0..C {
            if target_row >= R {
                break;
            }
            for row in target_row..R {
                if !output[col][row].is_zero() {
                    output.swap_rows(row, target_row);
                    output.div_row(target_row, output[col][target_row]);
                    for cancel_row in 0..R {
                        if cancel_row != target_row && !output[col][cancel_row].is_zero() {
                            output.add_row(cancel_row, target_row, -output[col][cancel_row]);
                        }
                    }
                    target_row += 1;
                    break;
                }
            }
        }
        output
    }
}

impl<T: Float + NumAssign, const N: usize> Matrix<T, N, N> {
    pub fn identity() -> Self {
        let mut matrix = Matrix::zero();
        for n in 0..N {
            matrix[n][n] = T::one();
        }
        matrix
    }

    pub fn reset_to_identity(&mut self) {
        self.fill_zero();
        for n in 0..N {
            self[n][n] = T::one();
        }
    }
}

impl<T: Float + NumAssign + Sum> Transform3D<T> {
    pub fn affine(&self) -> Matrix<T, 3, 3> {
        Matrix([
            Vector([self[0][0], self[0][1], self[0][2]]),
            Vector([self[1][0], self[1][1], self[1][2]]),
            Vector([self[2][0], self[2][1], self[2][2]]),
        ])
    }

    pub fn rotate_x(&mut self, angle: T) {
        let zero = T::zero();
        let one = T::one();
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(Self([
            Vector([one, zero, zero, zero]),
            Vector([zero, cos, -sin, zero]),
            Vector([zero, sin, cos, zero]),
            Vector([zero, zero, zero, one]),
        ]));
    }

    pub fn rotate_y(&mut self, angle: T) {
        let zero = T::zero();
        let one = T::one();
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(Self([
            Vector([cos, zero, sin, zero]),
            Vector([zero, one, zero, zero]),
            Vector([-sin, zero, cos, zero]),
            Vector([zero, zero, zero, one]),
        ]));
    }

    pub fn rotate_z(&mut self, angle: T) {
        let zero = T::zero();
        let one = T::one();
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(Self([
            Vector([cos, sin, zero, zero]),
            Vector([-sin, cos, zero, zero]),
            Vector([zero, zero, one, zero]),
            Vector([zero, zero, zero, one]),
        ]));
    }

    pub fn translate(&mut self, amount: Vector<T, 3>) {
        let to_add = self[0].mul(amount.x()) + self[1].mul(amount.y()) + self[2].mul(amount.z());
        self[3] += to_add;
    }

    pub fn scale(&mut self, amount: Vector<T, 3>) {
        self.scale_x(amount.x());
        self.scale_y(amount.y());
        self.scale_z(amount.z());
    }

    pub fn scale_x(&mut self, amount: T) {
        self[0] = self[0].mul(amount);
    }

    pub fn scale_y(&mut self, amount: T) {
        self[1] = self[1].mul(amount);
    }

    pub fn scale_z(&mut self, amount: T) {
        self[2] = self[2].mul(amount);
    }

    pub fn inverted(&self) -> Self {
        // Borrowed from JOML (github.com/JOML-CI/JOML)
        let a = self[0][0] * self[1][1] - self[0][1] * self[1][0];
        let b = self[0][0] * self[1][2] - self[0][2] * self[1][0];
        let c = self[0][0] * self[1][3] - self[0][3] * self[1][0];
        let d = self[0][1] * self[1][2] - self[0][2] * self[1][1];
        let e = self[0][1] * self[1][3] - self[0][3] * self[1][1];
        let f = self[0][2] * self[1][3] - self[0][3] * self[1][2];
        let g = self[2][0] * self[3][1] - self[2][1] * self[3][0];
        let h = self[2][0] * self[3][2] - self[2][2] * self[3][0];
        let i = self[2][0] * self[3][3] - self[2][3] * self[3][0];
        let j = self[2][1] * self[3][2] - self[2][2] * self[3][1];
        let k = self[2][1] * self[3][3] - self[2][3] * self[3][1];
        let l = self[2][2] * self[3][3] - self[2][3] * self[3][2];

        Self([
            Vector([
                l.mul_add(self[1][1], k.mul_add(-self[1][2], j * self[1][3])),
                l.mul_add(-self[0][1], k.mul_add(self[0][2], j * -self[0][3])),
                f.mul_add(self[3][1], e.mul_add(-self[3][2], d * self[3][3])),
                f.mul_add(-self[2][1], e.mul_add(self[2][2], d * -self[2][3])),
            ]),
            Vector([
                l.mul_add(-self[1][0], i.mul_add(self[1][2], h * -self[1][3])),
                l.mul_add(self[0][0], i.mul_add(-self[0][2], h * self[0][3])),
                f.mul_add(-self[3][0], c.mul_add(self[3][2], b * -self[3][3])),
                f.mul_add(self[2][0], c.mul_add(-self[2][2], b * self[2][3])),
            ]),
            Vector([
                k.mul_add(self[1][0], i.mul_add(-self[1][1], g * self[1][3])),
                k.mul_add(-self[0][0], i.mul_add(self[0][1], g * -self[0][3])),
                e.mul_add(self[3][0], c.mul_add(-self[3][1], a * self[3][3])),
                e.mul_add(-self[2][0], c.mul_add(self[2][1], a * -self[2][3])),
            ]),
            Vector([
                j.mul_add(-self[1][0], h.mul_add(self[1][1], g * -self[1][2])),
                j.mul_add(self[0][0], h.mul_add(-self[0][1], g * self[0][2])),
                d.mul_add(-self[3][0], b.mul_add(self[3][1], a * -self[3][2])),
                d.mul_add(self[2][0], b.mul_add(-self[2][1], a * self[2][2])),
            ]),
        ])
    }

    pub fn set_look_at(&mut self, eye: Vector<T, 3>, center: Vector<T, 3>, up: Vector<T, 3>) {
        let forward = (eye - center).normalized();
        let right = up.cross(forward).normalized();
        let up = forward.cross(right).normalized();

        self[0].0 = [right.x(), up.x(), forward.x(), T::zero()];
        self[1].0 = [right.y(), up.y(), forward.y(), T::zero()];
        self[2].0 = [right.z(), up.z(), forward.z(), T::zero()];
        self[3].0 = [-right.dot::<T>(eye), -up.dot::<T>(eye), -forward.dot::<T>(eye), T::one()];
    }

    pub fn new_look_at(eye: Vector<T, 3>, center: Vector<T, 3>, up: Vector<T, 3>) -> Self {
        let mut transform = Self::zero();
        transform.set_look_at(eye, center, up);
        transform
    }

    pub fn look_at(&mut self, eye: Vector<T, 3>, center: Vector<T, 3>, up: Vector<T, 3>) {
        let transform = Self::new_look_at(eye, center, up);
        *self *= transform;
    }

    pub fn orthographic(&mut self, left: T, right: T, bottom: T, top: T, near: T, far: T) {
        self.translate(Vector([
            (left + right) / (left - right),
            (bottom + top) / (bottom - top),
            (near + far) / (near - far),
        ]));
        let two = T::one() + T::one();
        self.scale(Vector([
            two / (right - left),
            two / (top - bottom),
            two / (near - far),
        ]));
    }

    pub fn frustum(&mut self, left: T, right: T, bottom: T, top: T, near: T, far: T) {
        let two = T::one() + T::one();
        let mut transform = Self::zero();
        transform[0][0] = two * near / (right - left);
        transform[1][1] = two * near / (top - bottom);
        transform[0][2] = (right + left) / (right - left);
        transform[1][2] = (top + bottom) / (top - bottom);
        transform[2][2] = (far + near) / (near - far);
        transform[3][2] = two * near * far / (near - far);
        transform[2][3] = -T::one();
        *self *= transform;
    }

    pub fn perspective(&mut self, field_of_view: T, aspect_ratio: T, near: T, far: T) {
        let two = T::one() + T::one();
        let scale = (field_of_view / two).to_radians().tan() * near;
        self.frustum(
            -scale * aspect_ratio,
            scale * aspect_ratio,
            -scale,
            scale,
            near,
            far,
        );
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> Index<usize>
    for Matrix<T, R, C>
{
    type Output = Vector<T, R>;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> IndexMut<usize>
    for Matrix<T, R, C>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T: Signed + Float + NumAssign, const R: usize, const C: usize> Neg
    for Matrix<T, R, C>
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Matrix(self.0.map(|v| -v))
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> Mul<T> for Matrix<T, R, C> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut output = self;
        for col in 0..C {
            output[col] = output[col].mul(rhs);
        }
        output
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> Div<T> for Matrix<T, R, C> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let mut output = self;
        for col in 0..C {
            output[col] = output[col].div(rhs);
        }
        output
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> Mul<Vector<T, C>>
    for Matrix<T, R, C>
{
    type Output = Vector<T, R>;

    fn mul(self, rhs: Vector<T, C>) -> Self::Output {
        let mut output = Vector::zero();
        for col in 0..C {
            output += self[col].mul(rhs[col]);
        }
        output
    }
}

// help me.
impl<T: Float + NumAssign, const R: usize, const C: usize> Mul<Vector<T, C>>
    for &Matrix<T, R, C>
{
    type Output = <Matrix<T, R, C> as Mul<Vector<T, C>>>::Output;
    fn mul(self, rhs: Vector<T, C>) -> Self::Output {
        <Matrix<T, R, C> as Mul<Vector<T, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> Mul<Vector<T, C>>
    for &mut Matrix<T, R, C>
{
    type Output = <Matrix<T, R, C> as Mul<Vector<T, C>>>::Output;
    fn mul(self, rhs: Vector<T, C>) -> Self::Output {
        <Matrix<T, R, C> as Mul<Vector<T, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign, const R: usize, const N: usize, const C: usize>
    Mul<Matrix<T, N, C>> for Matrix<T, R, N>
{
    type Output = Matrix<T, R, C>;

    fn mul(self, rhs: Matrix<T, N, C>) -> Self::Output {
        let mut output = Matrix::zero();
        for col in 0..C {
            output[col] = self * rhs[col];
        }
        output
    }
}

impl<T: Float + NumAssign, const R: usize, const N: usize, const C: usize>
    Mul<Matrix<T, N, C>> for &Matrix<T, R, N>
{
    type Output = <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::Output;

    fn mul(self, rhs: Matrix<T, N, C>) -> Self::Output {
        <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign, const R: usize, const N: usize, const C: usize>
    Mul<Matrix<T, N, C>> for &mut Matrix<T, R, N>
{
    type Output = <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::Output;

    fn mul(self, rhs: Matrix<T, N, C>) -> Self::Output {
        <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> MulAssign<Matrix<T, C, C>>
    for Matrix<T, R, C>
{
    fn mul_assign(&mut self, rhs: Matrix<T, C, C>) {
        let original = *self;
        for col in 0..C {
            self[col] = original * rhs[col];
        }
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> MulAssign<Matrix<T, C, C>>
    for &mut Matrix<T, R, C>
{
    fn mul_assign(&mut self, rhs: Matrix<T, C, C>) {
        <Matrix<T, R, C> as MulAssign<Matrix<T, C, C>>>::mul_assign(*self, rhs)
    }
}

impl<T: Float + NumAssign> Mul<Vector<T, 3>> for Transform3D<T> {
    type Output = Vector<T, 3>;

    fn mul(self, rhs: Vector<T, 3>) -> Self::Output {
        let scale = T::one() / (
            self[0][3] * rhs.x() + self[1][3] * rhs.y() + self[2][3] * rhs.z() + self[3][3]
        );
        Vector([
            (self[0][0] * rhs.x() + self[1][0] * rhs.y() + self[2][0] * rhs.z() + self[3][0])
                * scale,
            (self[0][1] * rhs.x() + self[1][1] * rhs.y() + self[2][1] * rhs.z() + self[3][1])
                * scale,
            (self[0][2] * rhs.x() + self[1][2] * rhs.y() + self[2][2] * rhs.z() + self[3][2])
                * scale,
        ])
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> std::fmt::Debug for Matrix<T, R, C> where T: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Matrix{:?}", self.0)
    }
}

#[derive(Copy, PartialEq, Clone)]
pub struct Rectangle<T: NumAssign + Copy> {
    min: Vector<T, 2>,
    max: Vector<T, 2>,
}

impl<T: NumAssign + Copy> Rectangle<T> {
    pub const fn new(min: Vector<T, 2>, max: Vector<T, 2>) -> Self {
        Self {
            min,
            max,
        }
    }

    pub fn from_size(min: Vector<T, 2>, size: Vector<T, 2>) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    pub fn min(&self) -> Vector<T, 2> {
        self.min
    }

    pub fn min_x(&self) -> T {
        self.min.x()
    }

    pub fn min_y(&self) -> T {
        self.min.y()
    }

    pub fn max(&self) -> Vector<T, 2> {
        self.max
    }

    pub fn max_x(&self) -> T {
        self.max.x()
    }

    pub fn max_y(&self) -> T {
        self.max.y()
    }

    pub fn min_x_max_y(&self) -> Vector<T, 2> {
        Vector([self.min_x(), self.max_y()])
    }

    pub fn max_x_min_y(&self) -> Vector<T, 2> {
        Vector([self.max_x(), self.min_y()])
    }

    pub fn size(&self) -> Vector<T, 2> {
        Vector([self.width(), self.height()])
    }

    pub fn width(&self) -> T {
        self.max_x() - self.min_x()
    }

    pub fn height(&self) -> T {
        self.max_y() - self.min_y()
    }

    pub fn set_min(&mut self, min: Vector<T, 2>) {
        self.min = min;
    }

    pub fn set_min_x(&mut self, x: T) {
        self.min.set_x(x);
    }

    pub fn set_min_y(&mut self, y: T) {
        self.min.set_y(y);
    }

    pub fn set_max(&mut self, max: Vector<T, 2>) {
        self.max = max;
    }

    pub fn set_max_x(&mut self, x: T) {
        self.max.set_x(x);
    }

    pub fn set_max_y(&mut self, y: T) {
        self.max.set_y(y);
    }

    pub fn shift_by(&mut self, amount: Vector<T, 2>) {
        self.shift_x_by(amount.x());
        self.shift_y_by(amount.y());
    }

    pub fn shift_x_by(&mut self, amount: T) {
        self.min.set_x(self.min.x() + amount);
        self.max.set_x(self.max.x() + amount);
    }

    pub fn shift_y_by(&mut self, amount: T) {
        self.min.set_y(self.min.y() + amount);
        self.max.set_y(self.max.y() + amount);
    }

    pub fn shift_min_to(&mut self, min: Vector<T, 2>) {
        self.shift_min_x_to(min.x());
        self.shift_min_y_to(min.y());
    }

    pub fn shift_min_x_to(&mut self, x: T) {
        self.max.set_x(x + self.width());
        self.min.set_x(x);
    }

    pub fn shift_min_y_to(&mut self, y: T) {
        self.max.set_y(y + self.height());
        self.min.set_y(y);
    }

    pub fn shift_max_to(&mut self, max: Vector<T, 2>) {
        self.shift_max_x_to(max.x());
        self.shift_max_y_to(max.y());
    }

    pub fn shift_max_x_to(&mut self, x: T) {
        self.min.set_x(x - self.width());
        self.max.set_x(x);
    }

    pub fn shift_max_y_to(&mut self, y: T) {
        self.min.set_y(y - self.height());
        self.max.set_y(y);
    }

    pub fn center(&self) -> Vector<T, 2> {
        (self.min + self.max).div(T::one() + T::one())
    }

    pub fn flip_x(&mut self) {
        std::mem::swap(&mut self.min[0], &mut self.max[0]);
    }

    pub fn flip_y(&mut self) {
        std::mem::swap(&mut self.min[1], &mut self.max[1]);
    }
}

impl<T: NumAssign + Copy> Rectangle<T> where T: PartialOrd {
    pub fn expand_toward(&mut self, amount: Vector<T, 2>) {
        self.expand_x_toward(amount.x());
        self.expand_y_toward(amount.y());
    }

    pub fn expand_x_toward(&mut self, amount: T) {
        if amount >= T::zero() {
            self.set_max_x(self.max_x() + amount);
        }
        else {
            self.set_min_x(self.min_x() + amount);
        }
    }

    pub fn expand_y_toward(&mut self, amount: T) {
        if amount >= T::zero() {
            self.set_max_y(self.max_y() + amount);
        }
        else {
            self.set_min_y(self.min_y() + amount);
        }
    }

    pub fn contains_inclusive(&self, point: Vector<T, 2>) -> bool {
        point.x() >= self.min_x() && point.x() <= self.max_x()
            && point.y() >= self.min_y() && point.y() <= self.max_y()
    }

    pub fn contains_exclusive(&self, point: Vector<T, 2>) -> bool {
        point.x() > self.min_x() && point.x() < self.max_x()
            && point.y() > self.min_y() && point.y() < self.max_y()
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.max_x() > other.min_x() && self.min_x() < other.max_x()
            && self.max_y() > other.min_y() && self.min_y() < other.max_y()
    }
}

impl<T: NumAssign + Copy> std::fmt::Debug for Rectangle<T> where T: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rectangle")
            .field("min", &self.min)
            .field("max", &self.max)
            .finish()
    }
}

#[derive(Debug)]
pub struct Clock {
    start_time: std::time::SystemTime,
}

impl Clock {
    pub fn start() -> Clock {
        Clock {
            start_time: std::time::SystemTime::now(),
        }
    }

    pub fn reset(&mut self) {
        self.start_time = std::time::SystemTime::now();
    }

    pub fn read(&self) -> f32 {
        self.start_time.elapsed().unwrap().as_millis() as f32 * 0.001
    }
}

pub fn lerp(t: f32, v0: f32, v1: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}

#[derive(Clone, Copy, Debug)]
pub enum Easing {
    None,
    Linear,
    Sine,
    SineIn,
    SineOut,
}

impl Easing {
    pub fn value(&self, t: f32, v0: f32, v1: f32) -> f32 {
        if t <= 0.0 {
            v0
        } else if t >= 1.0 {
            v1
        } else {
            match *self {
                Self::Linear => lerp(t, v0, v1),
                Self::Sine => lerp(0.5 - 0.5 * (t * std::f32::consts::PI).cos(), v0, v1),
                Self::SineIn => lerp(1.0 - (t * std::f32::consts::FRAC_PI_2).cos(), v0, v1),
                Self::SineOut => lerp((t * std::f32::consts::FRAC_PI_2).sin(), v0, v1),
                _ => v1,
            }
        }
    }
}

#[derive(Debug)]
pub struct AnimationTimer<'a> {
    // TODO: account for duration change when interrupting animation?
    clock: &'a Clock,
    easing: Easing,
    duration: f32,
    origin_time: Option<f32>,
    origin: f32,
    target: f32,
    repeat: bool,
}

impl<'a> AnimationTimer<'a> {
    pub fn new(
        clock: &'a Clock,
        easing: Easing,
        duration: f32,
        init_value: f32,
    ) -> AnimationTimer<'a> {
        AnimationTimer {
            clock,
            easing,
            duration,
            origin_time: None,
            origin: init_value,
            target: init_value,
            repeat: false,
        }
    }

    pub fn easing(&self) -> Easing {
        self.easing
    }

    pub fn set_easing(&mut self, easing: Easing) {
        self.set_origin(self.value());
        self.easing = easing;
    }

    pub fn origin(&self) -> f32 {
        self.origin
    }

    pub fn set_origin(&mut self, value: f32) {
        self.origin = value;
        self.origin_time = Some(self.clock.read());
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn set_target(&mut self, value: f32) {
        self.set_origin(self.value());
        self.target = value;
        self.repeat = false;
    }

    pub fn at_target(&self) -> bool {
        self.origin_time.is_none() || self.clock.read() >= self.origin_time.unwrap() + self.duration
    }

    pub fn set_repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    pub fn value(&self) -> f32 {
        if self.duration <= 0.0 || self.origin_time.is_none() {
            return self.target;
        }
        let mut origin_time = self.origin_time.unwrap();
        let time = self.clock.read();
        if self.repeat && time >= origin_time + self.duration {
            origin_time = time - (time - origin_time) % self.duration;
        }
        self.easing.value(
            (time - origin_time) / self.duration,
            self.origin,
            self.target,
        )
    }

    pub fn set_value(&mut self, value: f32) {
        self.set_origin(value);
        self.target = value;
    }
}
