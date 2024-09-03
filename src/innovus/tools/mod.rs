use num::{traits::NumAssign, Float, Signed};
use std::cmp::Ordering;
use std::{fmt, ops::*};

pub use uuid::Uuid;

pub mod phys;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector<T: NumAssign + Copy, const N: usize>(
    pub [T; N],
);

pub type Vector2i = Vector<i32, 2>;
pub type Vector2f = Vector<f32, 2>;
pub type Vector2d = Vector<f64, 2>;

impl<T: NumAssign + Copy, const N: usize> Eq for Vector<T, N> where T: Eq {}

impl<T: NumAssign + Copy, const N: usize> Ord for Vector<T, N> where T: Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        <Self as PartialOrd>::partial_cmp(self, other).unwrap()
    }
}

impl<T: NumAssign + Copy, const N: usize> Vector<T, N> {
    pub fn zero() -> Self {
        Vector([T::zero(); N])
    }

    pub fn one() -> Self {
        Vector([T::one(); N])
    }

    pub fn content(&self) -> [T; N] {
        self.0
    }

    pub fn at(&self, index: usize) -> T {
        self.0[index]
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.0[index] = value;
    }

    pub fn fill(&mut self, value: T) {
        self.0 = [value; N];
    }

    pub fn dot(&self, rhs: &Self) -> T {
        let mut dot_product = T::zero();
        for index in 0..N {
            dot_product += self.at(index) * rhs.at(index);
        }
        dot_product
    }
}

impl<T: Float + NumAssign, const N: usize> Vector<T, N> {
    pub fn magnitude(&self) -> T {
        self.dot(self).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let mut normal = *self;
        let magnitude = normal.magnitude();
        for index in 0..N {
            normal.0[index] /= magnitude;
        }
        normal
    }
}

impl<T: NumAssign + Copy> Vector<T, 2> {
    pub fn x(&self) -> T {
        self.0[0]
    }

    pub fn set_x(&mut self, x: T) {
        self.0[0] = x;
    }

    pub fn y(&self) -> T {
        self.0[1]
    }

    pub fn set_y(&mut self, y: T) {
        self.0[1] = y;
    }
}

impl<T: NumAssign + Copy> Vector<T, 3> {
    pub fn x(&self) -> T {
        self.0[0]
    }

    pub fn set_x(&mut self, x: T) {
        self.0[0] = x;
    }

    pub fn y(&self) -> T {
        self.0[1]
    }

    pub fn set_y(&mut self, y: T) {
        self.0[1] = y;
    }

    pub fn z(&self) -> T {
        self.0[2]
    }

    pub fn set_z(&mut self, z: T) {
        self.0[2] = z;
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Vector([
            self.y() * rhs.z() - self.z() * rhs.y(),
            self.z() * rhs.x() - self.x() * rhs.z(),
            self.x() * rhs.y() - self.y() * rhs.x(),
        ])
    }
}

impl<T: NumAssign + Copy> Vector<T, 4> {
    pub fn x(&self) -> T {
        self.0[0]
    }

    pub fn set_x(&mut self, x: T) {
        self.0[0] = x;
    }

    pub fn y(&self) -> T {
        self.0[1]
    }

    pub fn set_y(&mut self, y: T) {
        self.0[1] = y;
    }

    pub fn z(&self) -> T {
        self.0[2]
    }

    pub fn set_z(&mut self, z: T) {
        self.0[2] = z;
    }

    pub fn w(&self) -> T {
        self.0[3]
    }

    pub fn set_w(&mut self, w: T) {
        self.0[3] = w;
    }
}

impl<T: NumAssign + Copy, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T: NumAssign + Copy, const N: usize> IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T: Signed + NumAssign + Copy, const N: usize> Neg for Vector<T, N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vector(self.0.map(|x| -x))
    }
}

impl<T: NumAssign + Copy, const N: usize> Add<Self> for Vector<T, N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] += rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy, const N: usize> AddAssign<Self> for Vector<T, N> {
    fn add_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] += rhs[index];
        }
    }
}

impl<T: NumAssign + Copy, const N: usize> Sub<Self> for Vector<T, N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] -= rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy, const N: usize> SubAssign<Self> for Vector<T, N> {
    fn sub_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] -= rhs[index];
        }
    }
}

impl<T: NumAssign + Copy, const N: usize> Mul<Self> for Vector<T, N> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] *= rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy, const N: usize> MulAssign<Self> for Vector<T, N> {
    fn mul_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] *= rhs[index];
        }
    }
}

impl<T: NumAssign + Copy, const N: usize> Div<Self> for Vector<T, N> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] /= rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy, const N: usize> DivAssign<Self> for Vector<T, N> {
    fn div_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] /= rhs[index];
        }
    }
}

impl<T: NumAssign + Copy, const N: usize> Mul<T> for Vector<T, N> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] *= rhs;
        }
        output
    }
}

impl<T: NumAssign + Copy, const N: usize> MulAssign<T> for Vector<T, N> {
    fn mul_assign(&mut self, rhs: T) {
        for index in 0..N {
            self[index] *= rhs;
        }
    }
}

impl<T: NumAssign + Copy, const N: usize> Div<T> for Vector<T, N> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] /= rhs;
        }
        output
    }
}

impl<T: NumAssign + Copy, const N: usize> DivAssign<T> for Vector<T, N> {
    fn div_assign(&mut self, rhs: T) {
        for index in 0..N {
            self[index] /= rhs;
        }
    }
}

impl<T: NumAssign + Copy, const N: usize> fmt::Debug for Vector<T, N> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector{:?}", self.0)
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Matrix<T: Float + NumAssign, const R: usize, const C: usize>(
    pub [Vector<T, R>; C],
);

pub type Transform2D = Matrix<f32, 3, 3>;
pub type Transform3D = Matrix<f32, 4, 4>;

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
        self.0 = [Vector::zero(); C];
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

impl Transform3D {
    pub fn affine(&self) -> Matrix<f32, 3, 3> {
        Matrix([
            Vector([self[0][0], self[0][1], self[0][2]]),
            Vector([self[1][0], self[1][1], self[1][2]]),
            Vector([self[2][0], self[2][1], self[2][2]]),
        ])
    }

    pub fn rotate_x(&mut self, angle: f32) {
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(Matrix([
            Vector([1.0, 0.0, 0.0, 0.0]),
            Vector([0.0, cos, -sin, 0.0]),
            Vector([0.0, sin, cos, 0.0]),
            Vector([0.0, 0.0, 0.0, 1.0]),
        ]));
    }

    pub fn rotate_y(&mut self, angle: f32) {
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(Matrix([
            Vector([cos, 0.0, sin, 0.0]),
            Vector([0.0, 1.0, 0.0, 0.0]),
            Vector([-sin, 0.0, cos, 0.0]),
            Vector([0.0, 0.0, 0.0, 1.0]),
        ]));
    }

    pub fn rotate_z(&mut self, angle: f32) {
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(Matrix([
            Vector([cos, sin, 0.0, 0.0]),
            Vector([-sin, cos, 0.0, 0.0]),
            Vector([0.0, 0.0, 1.0, 0.0]),
            Vector([0.0, 0.0, 0.0, 1.0]),
        ]));
    }

    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        let to_add = self[0] * dx + self[1] * dy + self[2] * dz;
        self[3] += to_add;
    }

    pub fn scale(&mut self, x_by: f32, y_by: f32, z_by: f32) {
        self.scale_x(x_by);
        self.scale_y(y_by);
        self.scale_z(z_by);
    }

    pub fn scale_x(&mut self, by: f32) {
        self[0] *= by;
    }

    pub fn scale_y(&mut self, by: f32) {
        self[1] *= by;
    }

    pub fn scale_z(&mut self, by: f32) {
        self[2] *= by;
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

    pub fn set_look_at(&mut self, eye: Vector<f32, 3>, center: Vector<f32, 3>, up: Vector<f32, 3>) {
        let forward = (eye - center).normalized();
        let right = up.cross(&forward).normalized();
        let up = forward.cross(&right).normalized();

        self[0].0 = [right.x(), up.x(), forward.x(), 0.0];
        self[1].0 = [right.y(), up.y(), forward.y(), 0.0];
        self[2].0 = [right.z(), up.z(), forward.z(), 0.0];
        self[3].0 = [-right.dot(&eye), -up.dot(&eye), -forward.dot(&eye), 1.0];
    }

    pub fn new_look_at(
        eye: Vector<f32, 3>,
        center: Vector<f32, 3>,
        up: Vector<f32, 3>,
    ) -> Transform3D {
        let mut transform = Transform3D::zero();
        transform.set_look_at(eye, center, up);
        return transform;
    }

    pub fn look_at(&mut self, eye: Vector<f32, 3>, center: Vector<f32, 3>, up: Vector<f32, 3>) {
        let transform = Transform3D::new_look_at(eye, center, up);
        self.mul_assign(transform);
    }

    pub fn orthographic(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        self.translate(
            (left + right) / (left - right),
            (bottom + top) / (bottom - top),
            (near + far) / (near - far),
        );
        self.scale(
            2.0 / (right - left),
            2.0 / (top - bottom),
            2.0 / (near - far),
        );
    }

    pub fn frustum(&mut self, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
        let mut transform = Transform3D::zero();
        transform[0][0] = 2.0 * near / (right - left);
        transform[1][1] = 2.0 * near / (top - bottom);
        transform[0][2] = (right + left) / (right - left);
        transform[1][2] = (top + bottom) / (top - bottom);
        transform[2][2] = (far + near) / (near - far);
        transform[3][2] = 2.0 * near * far / (near - far);
        transform[2][3] = -1.0;
        self.mul_assign(transform);
    }

    pub fn perspective(&mut self, field_of_view: f32, aspect_ratio: f32, near: f32, far: f32) {
        let scale: f32 = (0.5 * field_of_view).to_radians().tan() * near;
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
            output[col] *= rhs;
        }
        output
    }
}

impl<T: Float + NumAssign, const R: usize, const C: usize> Div<T> for Matrix<T, R, C> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let mut output = self;
        for col in 0..C {
            output[col] /= rhs;
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
            output += self[col] * rhs[col];
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

impl Mul<Vector<f32, 3>> for Transform3D {
    type Output = Vector<f32, 3>;

    fn mul(self, rhs: Vector<f32, 3>) -> Self::Output {
        let scale =
            1.0 / (self[0][3] * rhs.x() + self[1][3] * rhs.y() + self[2][3] * rhs.z() + self[3][3]);
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

impl<T: Float + NumAssign, const R: usize, const C: usize> fmt::Debug for Matrix<T, R, C> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix{:?}", self.0)
    }
}

#[derive(Copy, Clone)]
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
        (self.min + self.max) / (T::one() + T::one())
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
        } else {
            self.set_min_x(self.min_x() + amount);
        }
    }

    pub fn expand_y_toward(&mut self, amount: T) {
        if amount >= T::zero() {
            self.set_max_y(self.max_y() + amount);
        } else {
            self.set_min_y(self.min_y() + amount);
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.max_x() > other.min_x() && self.min_x() < other.max_x()
            && self.max_y() > other.min_y() && self.min_y() < other.max_y()
    }
}

impl<T: NumAssign + Copy> fmt::Debug for Rectangle<T> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
