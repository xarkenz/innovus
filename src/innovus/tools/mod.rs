use num::{traits::NumAssign, Float, Signed};
use std::cmp::Ordering;
use std::{fmt::*, ops::*};

pub use uuid::Uuid;

pub mod phys;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Vector<T: NumAssign + Copy + Debug, const N: usize>(pub [T; N]);

impl<T: NumAssign + Copy + Debug, const N: usize> Eq for Vector<T, N> where T: Eq {}

impl<T: NumAssign + Copy + Debug, const N: usize> Ord for Vector<T, N>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        <Self as PartialOrd>::partial_cmp(self, other).unwrap()
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> Vector<T, N> {
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

impl<T: Float + NumAssign + Debug, const N: usize> Vector<T, N> {
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

impl<T: NumAssign + Copy + Debug> Vector<T, 2> {
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

impl<T: NumAssign + Copy + Debug> Vector<T, 3> {
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

impl<T: NumAssign + Copy + Debug> Vector<T, 4> {
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

impl<T: NumAssign + Copy + Debug, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T: Signed + NumAssign + Copy + Debug, const N: usize> Neg for Vector<T, N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vector(self.0.map(|x| -x))
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> Add<Self> for Vector<T, N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] += rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> AddAssign<Self> for Vector<T, N> {
    fn add_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] += rhs[index];
        }
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> Sub<Self> for Vector<T, N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] -= rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> SubAssign<Self> for Vector<T, N> {
    fn sub_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] -= rhs[index];
        }
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> Mul<Self> for Vector<T, N> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] *= rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> MulAssign<Self> for Vector<T, N> {
    fn mul_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] *= rhs[index];
        }
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> Div<Self> for Vector<T, N> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] /= rhs[index];
        }
        output
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> DivAssign<Self> for Vector<T, N> {
    fn div_assign(&mut self, rhs: Self) {
        for index in 0..N {
            self[index] /= rhs[index];
        }
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> Mul<T> for Vector<T, N> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] *= rhs;
        }
        output
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> MulAssign<T> for Vector<T, N> {
    fn mul_assign(&mut self, rhs: T) {
        for index in 0..N {
            self[index] *= rhs;
        }
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> Div<T> for Vector<T, N> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let mut output = self;
        for index in 0..N {
            output[index] /= rhs;
        }
        output
    }
}

impl<T: NumAssign + Copy + Debug, const N: usize> DivAssign<T> for Vector<T, N> {
    fn div_assign(&mut self, rhs: T) {
        for index in 0..N {
            self[index] /= rhs;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct Matrix<T: Float + NumAssign + Debug, const R: usize, const C: usize>(
    pub [Vector<T, R>; C],
);

pub type Transform2D = Matrix<f32, 3, 3>;
pub type Transform3D = Matrix<f32, 4, 4>;

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> Matrix<T, R, C> {
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

    pub unsafe fn as_ptr(&self) -> *const T {
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

impl<T: Float + NumAssign + Debug, const N: usize> Matrix<T, N, N> {
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

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> Index<usize>
    for Matrix<T, R, C>
{
    type Output = Vector<T, R>;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> IndexMut<usize>
    for Matrix<T, R, C>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T: Signed + Float + NumAssign + Debug, const R: usize, const C: usize> Neg
    for Matrix<T, R, C>
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Matrix(self.0.map(|v| -v))
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> Mul<T> for Matrix<T, R, C> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut output = self;
        for col in 0..C {
            output[col] *= rhs;
        }
        output
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> Div<T> for Matrix<T, R, C> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let mut output = self;
        for col in 0..C {
            output[col] /= rhs;
        }
        output
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> Mul<Vector<T, C>>
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
impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> Mul<Vector<T, C>>
    for &Matrix<T, R, C>
{
    type Output = <Matrix<T, R, C> as Mul<Vector<T, C>>>::Output;
    fn mul(self, rhs: Vector<T, C>) -> Self::Output {
        <Matrix<T, R, C> as Mul<Vector<T, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> Mul<Vector<T, C>>
    for &mut Matrix<T, R, C>
{
    type Output = <Matrix<T, R, C> as Mul<Vector<T, C>>>::Output;
    fn mul(self, rhs: Vector<T, C>) -> Self::Output {
        <Matrix<T, R, C> as Mul<Vector<T, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const N: usize, const C: usize>
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

impl<T: Float + NumAssign + Debug, const R: usize, const N: usize, const C: usize>
    Mul<Matrix<T, N, C>> for &Matrix<T, R, N>
{
    type Output = <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::Output;
    fn mul(self, rhs: Matrix<T, N, C>) -> Self::Output {
        <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const N: usize, const C: usize>
    Mul<Matrix<T, N, C>> for &mut Matrix<T, R, N>
{
    type Output = <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::Output;
    fn mul(self, rhs: Matrix<T, N, C>) -> Self::Output {
        <Matrix<T, R, N> as Mul<Matrix<T, N, C>>>::mul(*self, rhs)
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> MulAssign<Matrix<T, C, C>>
    for Matrix<T, R, C>
{
    fn mul_assign(&mut self, rhs: Matrix<T, C, C>) {
        let original = *self;
        for col in 0..C {
            self[col] = original * rhs[col];
        }
    }
}

impl<T: Float + NumAssign + Debug, const R: usize, const C: usize> MulAssign<Matrix<T, C, C>>
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

#[derive(Copy, Clone, Debug)]
pub struct Rectangle<T: Copy + Debug> {
    x: T,
    y: T,
    width: T,
    height: T,
}

impl<T: Copy + Debug> Rectangle<T> {
    pub const fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }

    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }

    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }

    pub fn set_width(&mut self, width: T) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: T) {
        self.height = height;
    }
}

impl<T: NumAssign + PartialOrd + Copy + Debug> Rectangle<T> {
    pub fn xw(&self) -> T {
        self.x + self.width
    }

    pub fn yh(&self) -> T {
        self.y + self.height
    }

    pub fn xy(&self) -> Vector<T, 2> {
        Vector([self.x, self.y])
    }

    pub fn size(&self) -> Vector<T, 2> {
        Vector([self.width, self.height])
    }

    pub fn center(&self) -> Vector<T, 2> {
        Vector([
            self.x + self.width / (T::one() + T::one()),
            self.y + self.height / (T::one() + T::one()),
        ])
    }

    pub fn set_xy(&mut self, xy: Vector<T, 2>) {
        [self.x, self.y] = xy.content();
    }

    pub fn set_size(&mut self, size: Vector<T, 2>) {
        [self.width, self.height] = size.content();
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.xw() > other.x()
            && self.x() < other.xw()
            && self.yh() > other.y()
            && self.y() < other.yh()
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
