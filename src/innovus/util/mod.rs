use std::{fmt, f32::consts, ops::*};


pub struct Vector<const N: usize> {
    data: [f32; N],
}

impl<const N: usize> Vector<N> {

    pub const fn new(data: [f32; N]) -> Vector<N> {
        Vector{ data }
    }

    pub const fn zero() -> Vector<N> {
        Vector{ data: [0.0; N] }
    }

    pub fn at(&self, pos: usize) -> f32 {
        self.data[pos]
    }

    pub fn data(&self) -> Vec<f32> {
        self.data.to_vec()
    }

    pub fn set(&mut self, pos: usize, value: f32) {
        self.data[pos] = value;
    }

    pub fn fill(&mut self, value: f32) {
        for pos in 0..N {
            self.data[pos] = value;
        }
    }

    pub fn magnitude(&self) -> f32 {
        let mut sum_squares: f32 = 0.0;
        for pos in 0..N {
            sum_squares += self.data[pos] * self.data[pos];
        }
        sum_squares.sqrt()
    }

    pub fn normalized(&self) -> Vector<N> {
        let inv_mag = 1.0 / self.magnitude();
        let mut data = self.data.clone();
        for pos in 0..N {
            data[pos] *= inv_mag;
        }
        Vector{ data }
    }

    pub fn dot(&self, with: &Vector<N>) -> f32 {
        let mut output: f32 = 0.0;
        for pos in 0..N {
            output += self.data[pos] * with.data[pos];
        }
        output
    }

}

impl Vector<2> {
    
    pub fn x(&self) -> f32 {
        self.data[0]
    }

    pub fn set_x(&mut self, x: f32) {
        self.data[0] = x;
    }

    pub fn y(&self) -> f32 {
        self.data[1]
    }

    pub fn set_y(&mut self, y: f32) {
        self.data[1] = y;
    }
    
}

impl Vector<3> {

    pub fn x(&self) -> f32 {
        self.data[0]
    }

    pub fn set_x(&mut self, x: f32) {
        self.data[0] = x;
    }

    pub fn y(&self) -> f32 {
        self.data[1]
    }

    pub fn set_y(&mut self, y: f32) {
        self.data[1] = y;
    }

    pub fn z(&self) -> f32 {
        self.data[2]
    }

    pub fn set_z(&mut self, z: f32) {
        self.data[2] = z;
    }

    pub fn cross(&self, with: &Vector<3>) -> Vector<3> {
        Vector{ data: [
            self.y() * with.z() - self.z() * with.y(),
            self.z() * with.x() - self.x() * with.z(),
            self.x() * with.y() - self.y() * with.x(),
        ] }
    }

}

impl Vector<4> {

    pub fn x(&self) -> f32 {
        self.data[0]
    }

    pub fn set_x(&mut self, x: f32) {
        self.data[0] = x;
    }

    pub fn y(&self) -> f32 {
        self.data[1]
    }

    pub fn set_y(&mut self, y: f32) {
        self.data[1] = y;
    }

    pub fn z(&self) -> f32 {
        self.data[2]
    }

    pub fn set_z(&mut self, z: f32) {
        self.data[2] = z;
    }

    pub fn w(&self) -> f32 {
        self.data[3]
    }

    pub fn set_w(&mut self, w: f32) {
        self.data[3] = w;
    }
    
}

impl<const N: usize> Neg for &Vector<N> {

    type Output = Vector<N>;

    fn neg(self) -> Self::Output {
        self * -1.0_f32
    }

}

impl<const N: usize> Add<&Vector<N>> for &Vector<N> {

    type Output = Vector<N>;

    fn add(self, with: &Vector<N>) -> Self::Output {
        let mut data: [f32; N] = [0.0; N];
        for pos in 0..N {
            data[pos] = self.data[pos] + with.data[pos];
        }
        Vector{ data }
    }

}

impl<const N: usize> Sub<&Vector<N>> for &Vector<N> {

    type Output = Vector<N>;

    fn sub(self, with: &Vector<N>) -> Self::Output {
        let mut data: [f32; N] = [0.0; N];
        for pos in 0..N {
            data[pos] = self.data[pos] - with.data[pos];
        }
        Vector{ data }
    }

}

impl<const N: usize> Mul<f32> for &Vector<N> {

    type Output = Vector<N>;

    fn mul(self, by: f32) -> Self::Output {
        let mut data = self.data.clone();
        for pos in 0..N {
            data[pos] *= by;
        }
        Vector{ data }
    }

}

impl<const N: usize> Div<f32> for &Vector<N> {

    type Output = Vector<N>;

    fn div(self, by: f32) -> Self::Output {
        let mut data = self.data.clone();
        for pos in 0..N {
            data[pos] *= by;
        }
        Vector{ data }
    }

}

impl<const N: usize> Clone for Vector<N> {

    fn clone(&self) -> Self {
        Vector{ data: self.data.clone() }
    }

}

impl<const N: usize> fmt::Debug for Vector<N> {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use fmt::Write;

        write!(formatter, "Vector({})[", N)?;
        if N != 0 {
            for pos in 0..N {
                if pos != 0 {
                    formatter.write_str(", ")?;
                }
                write!(formatter, "{}", self.data[pos])?;
            }
        }
        formatter.write_char(']')
    }

}


pub struct Matrix<const R: usize, const C: usize> {
    data: [[f32; C]; R],
}

pub type Transform2D = Matrix<3, 3>;
pub type Transform3D = Matrix<4, 4>;

impl<const R: usize, const C: usize> Matrix<R, C> {

    pub const fn new(data: [[f32; C]; R]) -> Matrix<R, C> {
        Matrix{ data }
    }

    pub const fn zero() -> Matrix<R, C> {
        Matrix{ data: [[0.0; C]; R] }
    }

    pub fn at(&self, row: usize, col: usize) -> f32 {
        self.data[row][col]
    }

    pub fn data(&self) -> Vec<f32> {
        let mut data: Vec<f32> = Vec::with_capacity(R * C);
        for row in 0..R {
            data.append(&mut self.data[row].to_vec())
        }
        data
    }

    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.data[row][col] = value;
    }

    pub fn fill(&mut self, value: f32) {
        for row in 0..R {
            for col in 0..C {
                self.data[row][col] = value;
            }
        }
    }

    pub fn swap_rows(&mut self, row1: usize, row2: usize) {
        for col in 0..C {
            let temp = self.data[row1][col];
            self.data[row1][col] = self.data[row2][col];
            self.data[row2][col] = temp;
        }
    }

    pub fn mul_row(&mut self, row: usize, value: f32) {
        for col in 0..C {
            self.data[row][col] *= value;
        }
    }

    pub fn div_row(&mut self, row: usize, value: f32) {
        for col in 0..C {
            self.data[row][col] /= value;
        }
    }

    pub fn add_row(&mut self, row: usize, from: usize, mul: f32) {
        for col in 0..C {
            self.data[row][col] += self.data[from][col] * mul;
        }
    }

    pub fn rref(&self) -> Self {
        let mut mat = self.clone();
        let mut target_row: usize = 0;
        for col in 0..C {
            if target_row >= R {
                break;
            }
            for row in target_row..R {
                if mat.at(row, col) != 0.0 {
                    mat.swap_rows(row, target_row);
                    mat.div_row(target_row, mat.at(target_row, col));
                    for cancel_row in 0..R {
                        if cancel_row != target_row && mat.at(cancel_row, col) != 0.0 {
                            mat.add_row(cancel_row, target_row, -mat.at(cancel_row, col));
                        }
                    }
                    target_row += 1;
                    break;
                }
            }
        }
        mat
    }

}

impl<const N: usize> Matrix<N, N> {

    pub fn identity() -> Matrix<N, N> {
        let mut mat: Matrix<N, N> = Matrix::zero();
        for n in 0..N {
            mat.data[n][n] = 1.0;
        }
        mat
    }

    pub fn reset_to_identity(&mut self) {
        for row in 0..N {
            for col in 0..N {
                self.data[row][col] = if row == col { 1.0 } else { 0.0 };
            }
        }
    }

}

impl Transform3D {

    pub fn affine(&self) -> Matrix<3, 3> {
        Matrix::new([
            [self.data[0][0], self.data[0][1], self.data[0][2]],
            [self.data[1][0], self.data[1][1], self.data[1][2]],
            [self.data[2][0], self.data[2][1], self.data[2][2]],
        ])
    }

    pub fn rotate_x(&mut self, angle: f32) {
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(&Matrix::new([
            [1.0,  0.0, 0.0, 0.0],
            [0.0,  cos, sin, 0.0],
            [0.0, -sin, cos, 0.0],
            [0.0,  0.0, 0.0, 1.0]]));
    }

    pub fn rotate_y(&mut self, angle: f32) {
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(&Matrix::new([
            [cos, 0.0, -sin, 0.0],
            [0.0, 1.0,  0.0, 0.0],
            [sin, 0.0,  cos, 0.0],
            [0.0, 0.0,  0.0, 1.0]]));
    }

    pub fn rotate_z(&mut self, angle: f32) {
        let (sin, cos) = angle.sin_cos();
        self.mul_assign(&Matrix::new([
            [cos, -sin, 0.0, 0.0],
            [sin,  cos, 0.0, 0.0],
            [0.0,  0.0, 1.0, 0.0],
            [0.0,  0.0, 0.0, 1.0]]));
    }

    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.data[0][3] += dx * self.data[0][0] + dy * self.data[0][1] + dz * self.data[0][2];
        self.data[1][3] += dx * self.data[1][0] + dy * self.data[1][1] + dz * self.data[1][2];
        self.data[2][3] += dx * self.data[2][0] + dy * self.data[2][1] + dz * self.data[2][2];
        self.data[3][3] += dx * self.data[3][0] + dy * self.data[3][1] + dz * self.data[3][2];
    }

    pub fn scale(&mut self, x_by: f32, y_by: f32, z_by: f32) {
        self.scale_x(x_by);
        self.scale_y(y_by);
        self.scale_z(z_by);
    }

    pub fn scale_x(&mut self, by: f32) {
        self.data[0][0] *= by;
        self.data[1][0] *= by;
        self.data[2][0] *= by;
        self.data[3][0] *= by;
    }

    pub fn scale_y(&mut self, by: f32) {
        self.data[0][1] *= by;
        self.data[1][1] *= by;
        self.data[2][1] *= by;
        self.data[3][1] *= by;
    }

    pub fn scale_z(&mut self, by: f32) {
        self.data[0][2] *= by;
        self.data[1][2] *= by;
        self.data[2][2] *= by;
        self.data[3][2] *= by;
    }

    pub fn set_look_at(&mut self, eye: &Vector<3>, center: &Vector<3>, up: &Vector<3>) {
        let forward = (eye - center).normalized();
        let right = up.cross(&forward).normalized();
        let up = forward.cross(&right).normalized();

        self.data[0][0] = right.x();
        self.data[0][1] = right.y();
        self.data[0][2] = right.z();
        self.data[0][3] = -right.dot(eye);
        self.data[1][0] = up.x();
        self.data[1][1] = up.y();
        self.data[1][2] = up.z();
        self.data[1][3] = -up.dot(eye);
        self.data[2][0] = forward.x();
        self.data[2][1] = forward.y();
        self.data[2][2] = forward.z();
        self.data[2][3] = -forward.dot(eye);
        self.data[3][0] = 0.0;
        self.data[3][1] = 0.0;
        self.data[3][2] = 0.0;
        self.data[3][3] = 1.0;
    }

    pub fn new_look_at(eye: &Vector<3>, center: &Vector<3>, up: &Vector<3>) -> Transform3D {
        let mut transform = Transform3D::zero();
        transform.set_look_at(eye, center, up);
        return transform;
    }

    pub fn look_at(&mut self, eye: &Vector<3>, center: &Vector<3>, up: &Vector<3>) {
        let transform = Transform3D::new_look_at(eye, center, up);
        self.mul_assign(&transform);
    }

    pub fn orthographic(&mut self, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
        self.translate((left + right) / (left - right), (bottom + top) / (bottom - top), (near + far) / (near - far));
        self.scale(2.0 / (right - left), 2.0 / (top - bottom), 2.0 / (near - far));
    }

    pub fn frustum(&mut self, left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
        let mut transform = Transform3D::zero();
        transform.set(0, 0, 2.0 * near / (right - left));
        transform.set(1, 1, 2.0 * near / (top - bottom));
        transform.set(2, 0, (right + left) / (right - left));
        transform.set(2, 1, (top + bottom) / (top - bottom));
        transform.set(2, 2, (far + near) / (near - far));
        transform.set(2, 3, 2.0 * near * far / (near - far));
        transform.set(3, 2, -1.0);
        self.mul_assign(&transform);
    }

    pub fn perspective(&mut self, field_of_view: f32, aspect_ratio: f32, near: f32, far: f32) {
        let scale: f32 = (0.5 * field_of_view).to_radians().tan() * near;
        self.frustum(-scale * aspect_ratio, scale * aspect_ratio, -scale, scale, near, far);
    }

}

impl<const R: usize, const C: usize> Neg for &Matrix<R, C> {

    type Output = Matrix<R, C>;

    fn neg(self) -> Self::Output {
        self * -1.0_f32
    }

}

impl<const R: usize, const C: usize> Mul<f32> for &Matrix<R, C> {

    type Output = Matrix<R, C>;

    fn mul(self, by: f32) -> Self::Output {
        let mut data = self.data.clone();
        for row in 0..R {
            for col in 0..C {
                data[row][col] *= by;
            }
        }
        Matrix{ data }
    }

}

impl<const R: usize, const C: usize> Div<f32> for &Matrix<R, C> {

    type Output = Matrix<R, C>;

    fn div(self, by: f32) -> Self::Output {
        let mut data = self.data.clone();
        for row in 0..R {
            for col in 0..C {
                data[row][col] /= by;
            }
        }
        Matrix{ data }
    }

}

impl<const R: usize, const C: usize> Mul<&Vector<C>> for &Matrix<R, C> {

    type Output = Vector<R>;

    fn mul(self, by: &Vector<C>) -> Self::Output {
        let mut data: [f32; R] = [0.0; R];
        for row in 0..R {
            for col in 0..C {
                data[row] += self.at(row, col) * by.at(col);
            }
        }
        Vector{ data }
    }

}

impl<const R: usize, const N: usize, const C: usize> Mul<&Matrix<N, C>> for &Matrix<R, N> {

    type Output = Matrix<R, C>;

    fn mul(self, by: &Matrix<N, C>) -> Self::Output {
        let mut data: [[f32; C]; R] = [[0.0; C]; R];
        for col in 0..C {
            for row in 0..R {
                for n in 0..N {
                    data[row][col] += self.at(row, n) * by.at(n, col);
                }
            }
        }
        Matrix{ data }
    }

}

impl<const R: usize, const C: usize> MulAssign<&Matrix<C, C>> for Matrix<R, C> {

    fn mul_assign(&mut self, by: &Matrix<C, C>) {
        let old = self.clone();
        for col in 0..C {
            for row in 0..R {
                self.data[row][col] = 0.0;
                for n in 0..C {
                    self.data[row][col] += old.at(row, n) * by.at(n, col);
                }
            }
        }
    }

}

impl Mul<&Vector<3>> for &Transform3D {

    type Output = Vector<3>;

    fn mul(self, by: &Vector<3>) -> Self::Output {
        let scale = 1.0 / (self.at(3, 0) * by.x() + self.at(3, 1) * by.y() + self.at(3, 2) * by.z() + self.at(3, 3));
        Vector{ data: [
            (self.at(0, 0) * by.x() + self.at(0, 1) * by.y() + self.at(0, 2) * by.z() + self.at(0, 3)) * scale,
            (self.at(1, 0) * by.x() + self.at(1, 1) * by.y() + self.at(1, 2) * by.z() + self.at(1, 3)) * scale,
            (self.at(2, 0) * by.x() + self.at(2, 1) * by.y() + self.at(2, 2) * by.z() + self.at(2, 3)) * scale,
        ] }
    }

}

impl<const R: usize, const C: usize> Clone for Matrix<R, C> {

    fn clone(&self) -> Self {
        Matrix{ data: self.data.clone() }
    }

}

impl<const R: usize, const C: usize> fmt::Debug for Matrix<R, C> {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use fmt::Write;

        write!(formatter, "Matrix({}x{})[", R, C)?;
        if R != 0 && C != 0 {
            for row in 0..R {
                if row != 0 {
                    formatter.write_str("; ")?;
                }
                for col in 0..C {
                    if col != 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{}", self.data[row][col])?;
                }
            }
        }
        formatter.write_char(']')
    }

}


#[derive(Debug)]
pub struct Clock {
    start_time: std::time::SystemTime,
    mark_time: f32,
}

impl Clock {

    pub fn start() -> Clock {
        Clock { start_time: std::time::SystemTime::now(), mark_time: 0.0 }
    }

    pub fn reset(&mut self) {
        self.start_time = std::time::SystemTime::now();
    }

    pub fn read(&self) -> f32 {
        self.start_time.elapsed().unwrap().as_millis() as f32 * 0.001
    }

    #[deprecated]
    pub fn mark(&mut self) -> f32 {
        let time = self.read();
        self.mark_time = time;
        time
    }

    #[deprecated]
    pub fn since_mark(&self) -> f32 {
        self.read() - self.mark_time
    }

}


pub fn lerp(t: f32, v0: f32, v1: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}

#[derive(Clone, Copy, Debug)]
pub enum Easing {
    None,
    Linear,
    Sine, SineIn, SineOut,
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
                Self::Sine => lerp(0.5 - 0.5 * (t * consts::PI).cos(), v0, v1),
                Self::SineIn => lerp(1.0 - (t * consts::FRAC_PI_2).cos(), v0, v1),
                Self::SineOut => lerp((t * consts::FRAC_PI_2).sin(), v0, v1),
                _ => v1
            }
        }
    }

}


#[derive(Debug)]
pub struct AnimationTimer<'a> {  // TODO: account for duration change when interrupting animation?
    clock: &'a Clock,
    easing: Easing,
    duration: f32,
    origin_time: Option<f32>,
    origin: f32,
    target: f32,
    repeat: bool,
}

impl<'a> AnimationTimer<'a> {

    pub fn new(clock: &'a Clock, easing: Easing, duration: f32, init_value: f32) -> AnimationTimer<'a> {
        AnimationTimer{ clock, easing, duration, origin_time: None, origin: init_value, target: init_value, repeat: false }
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
        self.easing.value((time - origin_time) / self.duration, self.origin, self.target)
    }

    pub fn set_value(&mut self, value: f32) {
        self.set_origin(value);
        self.target = value;
    }

}