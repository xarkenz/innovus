use num::Float;
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

impl<T, const N: usize> Vector<T, N> {
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Self(std::array::from_fn(|_| T::zero()))
    }

    pub fn one() -> Self
    where
        T: One,
    {
        Self(std::array::from_fn(|_| T::one()))
    }

    pub const fn filled(value: T) -> Self
    where
        T: Copy,
    {
        Self([value; N])
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

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

    pub fn dot<Rhs>(self, rhs: Vector<Rhs, N>) -> <T as Mul<Rhs>>::Output
    where
        T: Mul<Rhs>,
        <T as Mul<Rhs>>::Output: Sum,
    {
        std::iter::zip(self.0, rhs.0)
            .map(|(lhs, rhs)| lhs * rhs)
            .sum()
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Copy + Float + Sum,
{
    pub fn magnitude(&self) -> T {
        self.dot(*self).sqrt()
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

impl<T, const N: usize> Eq for Vector<T, N>
where
    T: Eq,
{}

impl<T, const N: usize> Ord for Vector<T, N>
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

impl<T, const N: usize> Sum for Vector<T, N>
where
    T: Zero + Add<Output = T>,
{
    fn sum<I: Iterator<Item = Vector<T, N>>>(iter: I) -> Self {
        iter.reduce(|lhs, rhs| lhs + rhs).unwrap_or(Vector::zero())
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
pub struct Matrix<T, const R: usize, const C: usize>(
    pub [Vector<T, R>; C],
);

pub type Transform2D<T> = Matrix<T, 3, 3>;

pub type Transform3D<T> = Matrix<T, 4, 4>;

impl<T, const R: usize, const C: usize> Copy for Matrix<T, R, C>
where
    T: Copy,
{}

impl<T, const R: usize, const C: usize> Clone for Matrix<T, R, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Matrix(self.0.clone())
    }
}

impl<T, const R: usize, const C: usize> Matrix<T, R, C> {
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Matrix(std::array::from_fn(|_| Vector::zero()))
    }

    pub fn set_zero(&mut self)
    where
        T: Zero,
    {
        self.0.fill_with(Vector::zero);
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr() as *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr() as *mut T
    }

    pub fn map_columns<F, U, const N: usize>(self, f: F) -> Matrix<U, N, C>
    where
        F: FnMut(Vector<T, R>) -> Vector<U, N>,
    {
        Matrix(self.0.map(f))
    }

    pub fn mul<Rhs>(self, rhs: Rhs) -> Matrix<<T as Mul<Rhs>>::Output, R, C>
    where
        Rhs: Copy,
        T: Mul<Rhs>,
    {
        self.map_columns(|lhs| lhs.mul(rhs))
    }

    pub fn div<Rhs>(self, rhs: Rhs) -> Matrix<<T as Div<Rhs>>::Output, R, C>
    where
        Rhs: Copy,
        T: Div<Rhs>,
    {
        self.map_columns(|lhs| lhs.div(rhs))
    }
}

impl<T, const N: usize> Matrix<T, N, N> {
    pub fn identity() -> Self
    where
        T: Copy + Zero + One,
    {
        let mut matrix = Matrix::zero();
        for n in 0..N {
            matrix[n][n] = T::one();
        }
        matrix
    }

    pub fn set_identity(&mut self)
    where
        T: Copy + Zero + One,
    {
        self.set_zero();
        for n in 0..N {
            self[n][n] = T::one();
        }
    }

    pub fn transpose(&mut self) {
        if N <= 1 {
            return;
        }
        for i in 0..N - 1 {
            for j in 1..N {
                let [col_i, col_j] = self.0.get_disjoint_mut([i, j]).unwrap();
                std::mem::swap(&mut col_i[j], &mut col_j[i]);
            }
        }
    }
}

impl<T> Transform3D<T>
where
    T: Float + NumAssign + Sum,
{
    pub fn affine(&self) -> Matrix<T, 3, 3> {
        Matrix([
            self[0].xyz(),
            self[1].xyz(),
            self[2].xyz(),
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
        self[3].0 = [-right.dot(eye), -up.dot(eye), -forward.dot(eye), T::one()];
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

impl<T, const R: usize, const C: usize> PartialEq for Matrix<T, R, C>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, const R: usize, const C: usize> PartialOrd for Matrix<T, R, C>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T, const R: usize, const C: usize> Eq for Matrix<T, R, C>
where
    T: Eq,
{}

impl<T, const R: usize, const C: usize> Ord for Matrix<T, R, C>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, const R: usize, const C: usize> Index<usize> for Matrix<T, R, C> {
    type Output = Vector<T, R>;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T, const R: usize, const C: usize> IndexMut<usize> for Matrix<T, R, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T, const R: usize, const C: usize> Neg for Matrix<T, R, C>
where
    T: Neg,
{
    type Output = Matrix<<T as Neg>::Output, R, C>;

    fn neg(self) -> Self::Output {
        Matrix(self.0.map(|v| -v))
    }
}

impl<Lhs, Rhs, const R: usize, const C: usize> Mul<Vector<Rhs, C>> for Matrix<Lhs, R, C>
where
    Lhs: Mul<Rhs>,
    Rhs: Copy,
    <Lhs as Mul<Rhs>>::Output: Zero + Sum,
{
    type Output = Vector<<Lhs as Mul<Rhs>>::Output, R>;

    fn mul(self, rhs: Vector<Rhs, C>) -> Self::Output {
        std::iter::zip(self.0, rhs.0)
            .map(|(lhs, rhs)| lhs.mul(rhs))
            .sum()
    }
}

impl<Lhs, Rhs, const R: usize, const N: usize, const C: usize> Mul<Matrix<Rhs, N, C>> for Matrix<Lhs, R, N>
where
    Lhs: Copy + Mul<Rhs>,
    Rhs: Copy,
    <Lhs as Mul<Rhs>>::Output: Zero + Sum,
{
    type Output = Matrix<<Lhs as Mul<Rhs>>::Output, R, C>;

    fn mul(self, rhs: Matrix<Rhs, N, C>) -> Self::Output {
        rhs.map_columns(|rhs| self * rhs)
    }
}

impl<Lhs, Rhs, const R: usize, const C: usize> MulAssign<Matrix<Rhs, C, C>> for Matrix<Lhs, R, C>
where
    Lhs: Copy + Zero + Mul<Rhs, Output = Lhs> + Sum,
    Rhs: Copy,
{
    fn mul_assign(&mut self, rhs: Matrix<Rhs, C, C>) {
        let lhs = *self;
        for col in 0..C {
            self[col] = lhs * rhs[col];
        }
    }
}

impl<T> Mul<Vector<T, 3>> for Transform3D<T>
where
    T: Copy + Add<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    type Output = Vector<T, 3>;

    fn mul(self, rhs: Vector<T, 3>) -> Self::Output {
        let factor = self[0].w() * rhs.x() + self[1].w() * rhs.y() + self[2].w() * rhs.z() + self[3].w();
        Vector([
            (self[0].x() * rhs.x() + self[1].x() * rhs.y() + self[2].x() * rhs.z() + self[3].x()) / factor,
            (self[0].y() * rhs.x() + self[1].y() * rhs.y() + self[2].y() * rhs.z() + self[3].y()) / factor,
            (self[0].z() * rhs.x() + self[1].z() * rhs.y() + self[2].z() * rhs.z() + self[3].z()) / factor,
        ])
    }
}

impl<T, const R: usize, const C: usize> std::fmt::Debug for Matrix<T, R, C>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Matrix{:?}", self.0)
    }
}

pub struct Rectangle<T, const N: usize = 2> {
    pub min: Vector<T, N>,
    pub max: Vector<T, N>,
}

impl<T, const N: usize> Rectangle<T, N> {
    pub const fn new(min: Vector<T, N>, max: Vector<T, N>) -> Self {
        Self {
            min,
            max,
        }
    }

    pub fn from_span(min: Vector<T, N>, span: Vector<T, N>) -> Self
    where
        T: Copy + Add<Output = T>,
    {
        Self {
            min,
            max: min + span,
        }
    }

    pub fn span(&self) -> Vector<T, N>
    where
        T: Copy + Sub<Output = T>,
    {
        self.max - self.min
    }

    pub fn axis_span(&self, axis: usize) -> T
    where
        T: Copy + Sub<Output = T>,
    {
        self.max[axis] - self.min[axis]
    }

    pub fn flip(&mut self) {
        std::mem::swap(&mut self.min, &mut self.max);
    }

    pub fn flip_axis(&mut self, axis: usize) {
        std::mem::swap(&mut self.min[axis], &mut self.max[axis]);
    }

    pub fn center(&self) -> Vector<T, N>
    where
        T: Copy + One + Add<Output = T> + Div<Output = T>
    {
        (self.min + self.max).div(T::one() + T::one())
    }

    pub fn shift_by(&mut self, amount: Vector<T, N>)
    where
        T: Copy + AddAssign,
    {
        self.min += amount;
        self.max += amount;
    }

    pub fn shift_axis_by(&mut self, axis: usize, amount: T)
    where
        T: Copy + AddAssign,
    {
        self.min[axis] += amount;
        self.max[axis] += amount;
    }

    pub fn shift_min_to(&mut self, target: Vector<T, N>)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.max += target - self.min;
        self.min = target;
    }

    pub fn shift_min_axis_to(&mut self, axis: usize, target: T)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.max[axis] += target - self.min[axis];
        self.min[axis] = target;
    }

    pub fn shift_max_to(&mut self, target: Vector<T, N>)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.min += target - self.max;
        self.max = target;
    }

    pub fn shift_max_axis_to(&mut self, axis: usize, target: T)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.min[axis] += target - self.max[axis];
        self.max[axis] = target;
    }

    pub fn expand_toward(&mut self, amount: Vector<T, N>)
    where
        T: Copy + Zero + AddAssign + PartialOrd,
    {
        for axis in 0..N {
            self.expand_axis_toward(axis, amount[axis]);
        }
    }

    pub fn expand_axis_toward(&mut self, axis: usize, amount: T)
    where
        T: Copy + Zero + AddAssign + PartialOrd,
    {
        if amount >= T::zero() {
            self.max[axis] += amount;
        }
        else {
            self.min[axis] += amount;
        }
    }

    pub fn contains_inclusive(&self, point: Vector<T, N>) -> bool
    where
        T: PartialOrd,
    {
        std::iter::zip(&self.min.0, &self.max.0)
            .zip(&point.0)
            .all(|((min, max), value)| value >= min && value <= max)
    }

    pub fn contains_exclusive(&self, point: Vector<T, N>) -> bool
    where
        T: PartialOrd,
    {
        std::iter::zip(&self.min.0, &self.max.0)
            .zip(&point.0)
            .all(|((min, max), value)| value > min && value < max)
    }

    pub fn intersects_inclusive(&self, other: &Self) -> bool
    where
        T: PartialOrd,
    {
        std::iter::zip(
            std::iter::zip(&self.min.0, &self.max.0),
            std::iter::zip(&other.min.0, &other.max.0),
        )
            .all(|((self_min, self_max), (other_min, other_max))| {
                self_max > other_min && self_min < other_max
            })
    }

    pub fn intersects_exclusive(&self, other: &Self) -> bool
    where
        T: PartialOrd,
    {
        std::iter::zip(
            std::iter::zip(&self.min.0, &self.max.0),
            std::iter::zip(&other.min.0, &other.max.0),
        )
            .all(|((self_min, self_max), (other_min, other_max))| {
                self_max > other_min && self_min < other_max
            })
    }
}

impl<T> Rectangle<T, 2> {
    pub fn min_x_max_y(&self) -> Vector<T, 2>
    where
        T: Copy,
    {
        Vector([self.min.x(), self.max.y()])
    }

    pub fn max_x_min_y(&self) -> Vector<T, 2>
    where
        T: Copy,
    {
        Vector([self.max.x(), self.min.y()])
    }

    pub fn x_span(&self) -> T
    where
        T: Copy + Sub<Output = T>,
    {
        self.axis_span(0)
    }

    pub fn y_span(&self) -> T
    where
        T: Copy + Sub<Output = T>,
    {
        self.axis_span(1)
    }

    pub fn flip_x(&mut self) {
        self.flip_axis(0);
    }

    pub fn flip_y(&mut self) {
        self.flip_axis(1);
    }

    pub fn shift_x_by(&mut self, amount: T)
    where
        T: Copy + AddAssign,
    {
        self.shift_axis_by(0, amount);
    }

    pub fn shift_y_by(&mut self, amount: T)
    where
        T: Copy + AddAssign,
    {
        self.shift_axis_by(1, amount);
    }

    pub fn shift_min_x_to(&mut self, target: T)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.shift_min_axis_to(0, target);
    }

    pub fn shift_min_y_to(&mut self, target: T)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.shift_min_axis_to(1, target);
    }

    pub fn shift_max_x_to(&mut self, target: T)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.shift_max_axis_to(0, target);
    }

    pub fn shift_max_y_to(&mut self, target: T)
    where
        T: Copy + AddAssign + Sub<Output = T>,
    {
        self.shift_max_axis_to(1, target);
    }

    pub fn expand_x_toward(&mut self, amount: T)
    where
        T: Copy + Zero + AddAssign + PartialOrd,
    {
        self.expand_axis_toward(0, amount);
    }

    pub fn expand_y_toward(&mut self, amount: T)
    where
        T: Copy + Zero + AddAssign + PartialOrd,
    {
        self.expand_axis_toward(1, amount);
    }
}

impl<T, const N: usize> Copy for Rectangle<T, N>
where
    T: Copy,
{}

impl<T, const N: usize> Clone for Rectangle<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.min.clone(), self.max.clone())
    }
}

impl<T, const N: usize> PartialEq for Rectangle<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}

impl<T, const N: usize> Eq for Rectangle<T, N>
where
    T: Eq,
{}

impl<T> std::fmt::Debug for Rectangle<T>
where
    T: std::fmt::Debug,
{
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
