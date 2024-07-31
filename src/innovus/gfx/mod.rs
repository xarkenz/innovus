extern crate gl;
extern crate glfw;
extern crate image;

pub mod screen;

use super::tools::{Matrix, Transform3D, Vector};
use gl::types::*;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::mem::{size_of, swap};
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

pub enum Color {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    FloatRGB(f32, f32, f32),
    FloatRGBA(f32, f32, f32, f32),
    Black,
    White,
}

pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize,
    Compute = gl::COMPUTE_SHADER as isize,
    Geometry = gl::GEOMETRY_SHADER as isize,
    TessControl = gl::TESS_CONTROL_SHADER as isize,
    TessEvaluation = gl::TESS_EVALUATION_SHADER as isize,
}

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn create(source: &str, shader_type: ShaderType) -> Result<Self, String> {
        let source = CString::new(source)
            .map_err(|err| err.to_string())?;

        let id = unsafe { gl::CreateShader(shader_type as GLenum) };
        if id == 0 {
            return Err("Shader::create(): failed to create GL shader object.".into());
        }
        let shader = Self { id };

        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut success: GLint = 1;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }
        if success == 0 {
            return Err(shader.get_info_log());
        }

        Ok(shader)
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn get_info_log(&self) -> String {
        // Determine the number of bytes the information log occupies, including the null terminator
        let mut length: GLint = 0;
        unsafe {
            gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut length);
        }

        if length <= 0 {
            // The shader has no information log
            String::new()
        }
        else {
            // Allocate space for the information log and read it
            let mut info_log = Vec::with_capacity(length as usize);
            unsafe {
                gl::GetShaderInfoLog(
                    self.id,
                    length,
                    std::ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                // Now that it contains data, update the length of the info log buffer,
                // excluding the null terminator (`length` is at least 1)
                info_log.set_len(length as usize - 1);
            }

            // It should be valid UTF-8, but check it to be sure
            String::from_utf8(info_log).unwrap()
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub trait ShaderUniformType {
    fn upload_uniform(self, location: GLint);
}

impl ShaderUniformType for GLfloat {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform1f(location, self as GLfloat);
        }
    }
}

impl ShaderUniformType for GLint {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform1i(location, self as GLint);
        }
    }
}

impl ShaderUniformType for GLuint {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform1ui(location, self as GLuint);
        }
    }
}

impl ShaderUniformType for GLboolean {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform1ui(location, self as GLuint);
        }
    }
}

impl ShaderUniformType for Vector<f32, 2> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform2f(location, self.x() as GLfloat, self.y() as GLfloat);
        }
    }
}

impl ShaderUniformType for Vector<f32, 3> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform3f(
                location,
                self.x() as GLfloat,
                self.y() as GLfloat,
                self.z() as GLfloat,
            );
        }
    }
}

impl ShaderUniformType for Vector<f32, 4> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform4f(
                location,
                self.x() as GLfloat,
                self.y() as GLfloat,
                self.z() as GLfloat,
                self.w() as GLfloat,
            );
        }
    }
}

impl ShaderUniformType for Matrix<f32, 2, 2> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::UniformMatrix2fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

impl ShaderUniformType for Matrix<f32, 3, 3> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::UniformMatrix3fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

impl ShaderUniformType for Matrix<f32, 4, 4> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

impl ShaderUniformType for Matrix<f32, 4, 2> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::UniformMatrix2x4fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

impl ShaderUniformType for Matrix<f32, 2, 4> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::UniformMatrix4x2fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

impl ShaderUniformType for Matrix<f32, 4, 3> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::UniformMatrix3x4fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

impl ShaderUniformType for Matrix<f32, 3, 4> {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::UniformMatrix4x3fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

impl ShaderUniformType for &Texture2D {
    fn upload_uniform(self, location: GLint) {
        unsafe {
            gl::Uniform1ui(location, self.slot());
        }
    }
}

pub enum ProgramPreset {
    Default2DShader,
    Default3DShader,
}

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Self, String> {
        let id = unsafe { gl::CreateProgram() };
        if id == 0 {
            return Err("Program::from_shaders(): failed to create GL program.".into());
        }
        let program = Self { id };

        for shader in shaders {
            unsafe {
                gl::AttachShader(id, shader.id());
            }
        }

        unsafe {
            gl::LinkProgram(id);
        }

        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }
        if success == 0 {
            return Err(program.get_info_log());
        }

        for shader in shaders {
            unsafe {
                gl::DetachShader(id, shader.id());
            }
        }

        Ok(program)
    }

    pub fn from_preset(preset: ProgramPreset) -> Result<Self, String> {
        let fetch_source = |path| std::fs::read_to_string(path)
            .map_err(|err| err.to_string());

        Self::from_shaders(&match preset {
            ProgramPreset::Default2DShader => vec![
                Shader::create(
                    &fetch_source("./src/innovus/assets/default2d.v.glsl")?,
                    ShaderType::Vertex,
                )?,
                Shader::create(
                    &fetch_source("./src/innovus/assets/default2d.f.glsl")?,
                    ShaderType::Fragment,
                )?,
            ],
            ProgramPreset::Default3DShader => vec![
                Shader::create(
                    &fetch_source("./src/innovus/assets/default3d.v.glsl")?,
                    ShaderType::Vertex,
                )?,
                Shader::create(
                    &fetch_source("./src/innovus/assets/default3d.g.glsl")?,
                    ShaderType::Geometry,
                )?,
                Shader::create(
                    &fetch_source("./src/innovus/assets/default3d.f.glsl")?,
                    ShaderType::Fragment,
                )?,
            ],
        })
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn get_info_log(&self) -> String {
        // Determine the number of bytes the information log occupies, including the null terminator
        let mut length: GLint = 0;
        unsafe {
            gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut length);
        }

        if length <= 0 {
            // The program has no information log
            String::new()
        }
        else {
            // Allocate space for the information log and read it
            let mut info_log = Vec::with_capacity(length as usize);
            unsafe {
                gl::GetProgramInfoLog(
                    self.id,
                    length,
                    std::ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                // Now that it contains data, update the length of the info log buffer,
                // excluding the null terminator (`length` is at least 1)
                info_log.set_len(length as usize - 1);
            }

            // It should be valid UTF-8, but check it to be sure
            String::from_utf8(info_log).unwrap()
        }
    }

    pub fn set_uniform(&self, name: &str, value: impl ShaderUniformType) {
        let name = CString::new(name).expect("uniform name must not contain any NUL bytes.");
        self.bind();
        let location = unsafe {
            gl::GetUniformLocation(self.id, name.as_ptr() as *const GLchar)
        };
        value.upload_uniform(location);
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub trait Vertex {
    const SIZE: usize;
    const ATTRIBUTE_SIZES: &'static [usize];

    fn to_raw_data(&self) -> Vec<f32>;
    fn from_raw_data(data: &[f32]) -> Self;
}

#[derive(Clone, Debug)]
pub struct Vertex3D {
    pub pos: [f32; 3],
    pub color: [f32; 4],
    pub tex: bool,
    pub uv: [f32; 2],
    pub norm: [f32; 3],
}

impl Vertex3D {
    pub fn new(
        pos: [f32; 3],
        color: Option<[f32; 4]>,
        uv: Option<[f32; 2]>,
        norm: Option<[f32; 3]>,
    ) -> Vertex3D {
        Vertex3D {
            pos,
            color: color.unwrap_or([1.0; 4]),
            tex: uv.is_some(),
            uv: uv.unwrap_or([0.0; 2]),
            norm: norm.unwrap_or([0.0; 3]),
        }
    }

    pub fn colored(pos: [f32; 3], color: [f32; 4]) -> Vertex3D {
        Vertex3D {
            pos,
            color,
            tex: false,
            uv: [0.0; 2],
            norm: [0.0; 3],
        }
    }

    pub fn textured(pos: [f32; 3], uv: [f32; 2]) -> Vertex3D {
        Vertex3D {
            pos,
            color: [1.0; 4],
            tex: true,
            uv,
            norm: [0.0; 3],
        }
    }

    pub fn combined(pos: [f32; 3], color: [f32; 4], uv: [f32; 2]) -> Vertex3D {
        Vertex3D {
            pos,
            color,
            tex: true,
            uv,
            norm: [0.0; 3],
        }
    }

    pub fn has_norm(&self) -> bool {
        self.norm[0] != 0.0 || self.norm[1] != 0.0 || self.norm[2] != 0.0
    }
}

impl Vertex for Vertex3D {
    const SIZE: usize = 13;
    const ATTRIBUTE_SIZES: &'static [usize] = &[3, 4, 1, 2, 3];

    fn to_raw_data(&self) -> Vec<f32> {
        vec![
            self.pos[0],
            self.pos[1],
            self.pos[2],
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3],
            if self.tex { 1.0 } else { 0.0 },
            self.uv[0],
            self.uv[1],
            self.norm[0],
            self.norm[1],
            self.norm[2],
        ]
    }

    fn from_raw_data(data: &[f32]) -> Self {
        Vertex3D {
            pos: [data[0], data[1], data[2]],
            color: [data[3], data[4], data[5], data[6]],
            tex: data[7] != 0.0,
            uv: [data[8], data[9]],
            norm: [data[10], data[11], data[12]],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vertex2D {
    pub pos: [f32; 3],
    pub color: [f32; 4],
    pub tex: bool,
    pub uv: [f32; 2],
}

impl Vertex2D {
    pub fn new(pos: [f32; 3], color: Option<[f32; 4]>, uv: Option<[f32; 2]>) -> Vertex2D {
        Vertex2D {
            pos,
            color: color.unwrap_or([1.0; 4]),
            tex: uv.is_some(),
            uv: uv.unwrap_or([0.0; 2]),
        }
    }
}

impl Vertex for Vertex2D {
    const SIZE: usize = 10;
    const ATTRIBUTE_SIZES: &'static [usize] = &[3, 4, 1, 2];

    fn to_raw_data(&self) -> Vec<f32> {
        vec![
            self.pos[0],
            self.pos[1],
            self.pos[2],
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3],
            if self.tex { 1.0 } else { 0.0 },
            self.uv[0],
            self.uv[1],
        ]
    }

    fn from_raw_data(data: &[f32]) -> Self {
        Vertex2D {
            pos: [data[0], data[1], data[2]],
            color: [data[3], data[4], data[5], data[6]],
            tex: data[7] != 0.0,
            uv: [data[8], data[9]],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GeometrySlice {
    pub first_vertex: usize,
    pub vertex_count: usize,
    pub first_face: usize,
    pub face_count: usize,
}

#[derive(Clone, Debug)]
pub struct Geometry<V: Vertex> {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    vertices: Vec<f32>,
    elements: Vec<GLuint>,
    vertex_count: usize,
    face_count: usize,
    vertex_type: PhantomData<*const V>,
}

impl<V: Vertex> Geometry<V> {
    pub fn new() -> Self {
        Geometry {
            vao: 0,
            vbo: 0,
            ebo: 0,
            vertices: Vec::new(),
            elements: Vec::new(),
            vertex_count: 0,
            face_count: 0,
            vertex_type: PhantomData,
        }
    }

    pub fn from_data(vertices: &[V], faces: &[[u32; 3]]) -> Self {
        let mut geometry = Self::new();
        geometry.add(vertices, faces);
        geometry
    }

    pub fn new_render() -> Result<Self, String> {
        let mut geometry = Self::new();
        geometry.enable_render()?;
        Ok(geometry)
    }

    pub fn enable_render(&mut self) -> Result<(), String> {
        if self.vao != 0 {
            return Ok(());
        }

        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            if self.vao == 0 {
                return Err(
                    "Geometry::enable_render(): failed to create GL vertex array object.".into(),
                );
            }
            gl::BindVertexArray(self.vao);

            gl::GenBuffers(1, &mut self.vbo);
            if self.vbo == 0 {
                return Err(
                    "Geometry::enable_render(): failed to create GL vertex buffer object.".into(),
                );
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::GenBuffers(1, &mut self.ebo);
            if self.ebo == 0 {
                return Err(
                    "Geometry::enable_render(): failed to create GL element buffer object.".into(),
                );
            }
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

            let mut offset: usize = 0;
            for (index, &size) in V::ATTRIBUTE_SIZES.iter().enumerate() {
                gl::EnableVertexAttribArray(index as GLuint);
                gl::VertexAttribPointer(
                    index as GLuint,
                    size as GLint,
                    gl::FLOAT,
                    gl::FALSE,
                    (V::SIZE * size_of::<f32>()) as GLint,
                    offset as *const GLvoid,
                );
                offset += size * size_of::<f32>();
            }

            // Vertex array must be unbound before anything else!
            // Otherwise, the buffers unbind from the vertex array
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        self.update_vertex_buffer();
        self.update_element_buffer();

        Ok(())
    }

    pub fn vao(&self) -> GLuint {
        self.vao
    }

    pub fn vbo(&self) -> GLuint {
        self.vbo
    }

    pub fn ebo(&self) -> GLuint {
        self.ebo
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn face_count(&self) -> usize {
        self.face_count
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.elements.is_empty()
    }

    pub fn vertex_data(&self) -> Vec<f32> {
        self.vertices.clone()
    }

    pub fn face_elements(&self) -> Vec<GLuint> {
        self.elements.clone()
    }

    pub fn render(&self) {
        if self.vao == 0 {
            panic!("Geometry::render(): must call 'Geometry::enable_render()' before rendering.");
        }

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.elements.len() as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }

    pub fn clear(&mut self) {
        self.vertex_count = 0;
        self.face_count = 0;

        self.vertices.clear();
        self.elements.clear();

        self.update_vertex_buffer();
        self.update_element_buffer();
    }

    pub fn update_vertex_buffer(&self) {
        if self.vbo != 0 {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.len() * size_of::<f32>()) as GLsizeiptr,
                    self.vertices.as_ptr() as *const GLvoid,
                    gl::DYNAMIC_DRAW,
                );
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn update_element_buffer(&self) {
        if self.ebo != 0 {
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (self.elements.len() * size_of::<GLuint>()) as GLsizeiptr,
                    self.elements.as_ptr() as *const GLvoid,
                    gl::DYNAMIC_DRAW,
                );
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn update_buffer_slice(&self, slice: &GeometrySlice) {
        if slice.vertex_count != 0 {
            let start = slice.first_vertex * V::SIZE;
            let size = slice.vertex_count * V::SIZE;
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    (start * size_of::<f32>()) as GLintptr,
                    (size * size_of::<f32>()) as GLsizeiptr,
                    self.vertices[start..(start + size)].as_ptr() as *const GLvoid,
                );
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
        if slice.face_count != 0 {
            let start = slice.first_face * 3;
            let size = slice.face_count * 3;
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::BufferSubData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (start * size_of::<GLuint>()) as GLintptr,
                    (size * size_of::<GLuint>()) as GLsizeiptr,
                    self.elements[start..(start + size)].as_ptr() as *const GLvoid,
                );
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn get_vertex(&self, index: usize) -> V {
        V::from_raw_data(&self.vertices[(index * V::SIZE)..((index + 1) * V::SIZE)])
    }

    pub fn set_vertex(&mut self, index: usize, vertex: &V) {
        self.vertices[(index * V::SIZE)..((index + 1) * V::SIZE)]
            .copy_from_slice(&vertex.to_raw_data());
    }

    pub fn add(&mut self, vertices: &[V], faces: &[[u32; 3]]) -> GeometrySlice {
        let first_vertex = self.vertex_count;
        let vertex_count = vertices.len();
        let first_face = self.face_count;
        let face_count = faces.len();

        if !faces.is_empty() {
            for face in faces {
                let mut data = face.to_vec();
                for idx in 0..data.len() {
                    data[idx] += self.vertex_count as u32;
                }
                self.elements.append(&mut data);
                self.face_count += 1;
            }
            self.update_element_buffer();
        }

        if !vertices.is_empty() {
            for vertex in vertices {
                self.vertices.append(&mut vertex.to_raw_data().to_vec());
                self.vertex_count += 1;
            }
            self.update_vertex_buffer();
        }

        GeometrySlice {
            first_vertex,
            vertex_count,
            first_face,
            face_count,
        }
    }

    pub fn append(&mut self, geometry: Self) -> GeometrySlice {
        let first_vertex = self.vertex_count;
        let vertex_count = geometry.vertex_count();
        let first_face = self.face_count;
        let face_count = geometry.face_count();

        let mut vertices = geometry.vertex_data();
        let mut indices = geometry.face_elements();

        if !indices.is_empty() {
            for idx in indices.iter_mut() {
                *idx += self.vertex_count as u32;
            }
            self.face_count += indices.len() / 3;
            self.elements.append(&mut indices);
            self.update_element_buffer();
        }

        if !vertices.is_empty() {
            self.vertex_count += vertices.len() / V::SIZE;
            self.vertices.append(&mut vertices);
            self.update_vertex_buffer();
        }

        GeometrySlice {
            first_vertex,
            vertex_count,
            first_face,
            face_count,
        }
    }

    pub fn as_slice(&self) -> GeometrySlice {
        GeometrySlice {
            first_vertex: 0,
            vertex_count: self.vertex_count,
            first_face: 0,
            face_count: self.face_count,
        }
    }
}

impl Geometry<Vertex3D> {
    fn generate_icosahedron(
        cx: f32,
        cy: f32,
        cz: f32,
        r: f32,
        color: [f32; 4],
    ) -> (Vec<Vertex3D>, Vec<[u32; 3]>) {
        const MINOR: f32 = 0.525731112119133606;
        const MAJOR: f32 = 0.850650808352039932;
        let minor = MINOR * r;
        let major = MAJOR * r;
        (
            vec![
                Vertex3D::new(
                    [cx - minor, cy, cz + major],
                    Some(color),
                    None,
                    Some([-MINOR, 0.0, MAJOR]),
                ),
                Vertex3D::new(
                    [cx + minor, cy, cz + major],
                    Some(color),
                    None,
                    Some([MINOR, 0.0, MAJOR]),
                ),
                Vertex3D::new(
                    [cx - minor, cy, cz - major],
                    Some(color),
                    None,
                    Some([-MINOR, 0.0, -MAJOR]),
                ),
                Vertex3D::new(
                    [cx + minor, cy, cz - major],
                    Some(color),
                    None,
                    Some([MINOR, 0.0, -MAJOR]),
                ),
                Vertex3D::new(
                    [cx, cy + major, cz + minor],
                    Some(color),
                    None,
                    Some([0.0, MAJOR, MINOR]),
                ),
                Vertex3D::new(
                    [cx, cy + major, cz - minor],
                    Some(color),
                    None,
                    Some([0.0, MAJOR, -MINOR]),
                ),
                Vertex3D::new(
                    [cx, cy - major, cz + minor],
                    Some(color),
                    None,
                    Some([0.0, -MAJOR, MINOR]),
                ),
                Vertex3D::new(
                    [cx, cy - major, cz - minor],
                    Some(color),
                    None,
                    Some([0.0, -MAJOR, -MINOR]),
                ),
                Vertex3D::new(
                    [cx + major, cy + minor, cz],
                    Some(color),
                    None,
                    Some([MAJOR, MINOR, 0.0]),
                ),
                Vertex3D::new(
                    [cx - major, cy + minor, cz],
                    Some(color),
                    None,
                    Some([-MAJOR, MINOR, 0.0]),
                ),
                Vertex3D::new(
                    [cx + major, cy - minor, cz],
                    Some(color),
                    None,
                    Some([MAJOR, -MINOR, 0.0]),
                ),
                Vertex3D::new(
                    [cx - major, cy - minor, cz],
                    Some(color),
                    None,
                    Some([-MAJOR, -MINOR, 0.0]),
                ),
            ],
            vec![
                [00, 01, 04],
                [00, 04, 09],
                [09, 04, 05],
                [04, 08, 05],
                [04, 01, 08],
                [08, 01, 10],
                [08, 10, 03],
                [05, 08, 03],
                [05, 03, 02],
                [02, 03, 07],
                [07, 03, 10],
                [07, 10, 06],
                [07, 06, 11],
                [11, 06, 00],
                [00, 06, 01],
                [06, 10, 01],
                [09, 11, 00],
                [09, 02, 11],
                [09, 05, 02],
                [07, 11, 02],
            ],
        )
    }

    pub fn add_icosphere(
        &mut self,
        center: Vector<f32, 3>,
        radius: f32,
        color: [f32; 4],
        subdivisions: u32,
    ) -> GeometrySlice {
        let (mut vertices, mut faces) = Geometry::generate_icosahedron(
            center.at(0),
            center.at(1),
            center.at(2),
            radius,
            color.clone(),
        );

        for _ in 0..subdivisions {
            let mut add_vertices: Vec<Vertex3D> = Vec::new();
            let mut new_faces: Vec<[u32; 3]> = Vec::new();
            let mut edge_midpoint_map: Vec<((u32, u32), u32)> = Vec::new();
            for face in faces {
                let mut fetch_midpoint = |v1, v2| {
                    let mut v1: u32 = v1;
                    let mut v2: u32 = v2;
                    if v1 > v2 {
                        swap(&mut v1, &mut v2);
                    }
                    for (edge, midpoint) in edge_midpoint_map.iter() {
                        if edge.0 == v1 && edge.1 == v2 {
                            return *midpoint;
                        }
                    }
                    let idx = (vertices.len() + add_vertices.len()) as u32;
                    let normal = (Vector(vertices[v1 as usize].norm)
                        + Vector(vertices[v2 as usize].norm))
                    .normalized();
                    add_vertices.push(Vertex3D::new(
                        (normal * radius + center).content(),
                        Some(color),
                        None,
                        Some(normal.content()),
                    ));
                    edge_midpoint_map.push(((v1, v2), idx));
                    idx
                };
                let midpoints: [u32; 3] = [
                    fetch_midpoint(face[0], face[1]),
                    fetch_midpoint(face[1], face[2]),
                    fetch_midpoint(face[2], face[0]),
                ];
                new_faces.push([face[0], midpoints[0], midpoints[2]]);
                new_faces.push([face[1], midpoints[1], midpoints[0]]);
                new_faces.push([face[2], midpoints[2], midpoints[1]]);
                new_faces.push(midpoints);
            }
            vertices.append(&mut add_vertices);
            faces = new_faces;
        }

        self.add(&vertices, &faces)
    }

    pub fn transform(&mut self, slice: &GeometrySlice, matrix: Transform3D) {
        for i in 0..slice.vertex_count {
            let i = slice.first_vertex + i;
            let mut vertex = self.get_vertex(i);
            vertex.pos = (matrix * Vector(vertex.pos)).content();
            if vertex.has_norm() {
                vertex.norm = (matrix.affine() * Vector(vertex.norm)).content();
            }
            self.set_vertex(i, &vertex);
        }
        self.update_buffer_slice(slice);
    }

    pub fn rotate_x(&mut self, slice: &GeometrySlice, angle: f32, axis_y: f32, axis_z: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(0.0, axis_y, axis_z);
        rotation.rotate_x(angle);
        rotation.translate(0.0, -axis_y, -axis_z);
        self.transform(slice, rotation);
    }

    pub fn rotate_y(&mut self, slice: &GeometrySlice, angle: f32, axis_x: f32, axis_z: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(axis_x, 0.0, axis_z);
        rotation.rotate_y(angle);
        rotation.translate(-axis_x, 0.0, -axis_z);
        self.transform(slice, rotation);
    }

    pub fn rotate_z(&mut self, slice: &GeometrySlice, angle: f32, axis_x: f32, axis_y: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(axis_x, axis_y, 0.0);
        rotation.rotate_z(angle);
        rotation.translate(-axis_x, -axis_y, 0.0);
        self.transform(slice, rotation);
    }
}

#[derive(Clone, Debug)]
pub struct ParseGeometryError {
    message: String,
}

impl Error for ParseGeometryError {}

impl fmt::Display for ParseGeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<ParseFloatError> for ParseGeometryError {
    fn from(error: ParseFloatError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl From<ParseIntError> for ParseGeometryError {
    fn from(error: ParseIntError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl FromStr for Geometry<Vertex3D> {
    type Err = ParseGeometryError;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let parse_f32 = |s: &str| {
            s.parse::<f32>()
                .map_err(ParseGeometryError::from)
        };
        let parse_clamp_f32 = |s: &str| match parse_f32(s)? {
            num if 0.0 <= num && num <= 1.0 => Ok(num),
            _ => Err(ParseGeometryError {
                message: format!("'{}' not in range 0-1", s),
            }),
        };
        let parse_usize = |s: &str| {
            s.parse::<usize>()
                .map_err(ParseGeometryError::from)
        };
        let parse_face_element = |s: &str| {
            let mut indices: [usize; 4] = [0; 4];
            let mut next_index: usize = 0;
            let mut index_start: Option<usize> = None;
            for (i, c) in s.chars().enumerate() {
                if c.is_ascii_digit() {
                    if index_start.is_none() {
                        index_start = Some(i);
                    }
                } else {
                    if index_start.is_some() {
                        indices[next_index] = parse_usize(&s[index_start.unwrap()..i])?;
                        index_start = None;
                    }
                    if c == '/' {
                        next_index += 1;
                        if next_index >= indices.len() {
                            return Err(ParseGeometryError {
                                message: "4 indices allowed per face element at maximum"
                                    .to_string(),
                            });
                        }
                    }
                }
            }
            if index_start.is_some() {
                indices[next_index] = parse_usize(&s[index_start.unwrap()..s.len()])?;
            }
            if indices[0] == 0 {
                Err(ParseGeometryError {
                    message: "face element index cannot be 0".into(),
                })
            } else {
                Ok(indices)
            }
        };

        let missing_error = ParseGeometryError {
            message: "missing command argument".into(),
        };

        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut textures: Vec<[f32; 2]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut colors: Vec<[f32; 4]> = Vec::new();
        let mut face_elements: Vec<[[usize; 4]; 3]> = Vec::new();

        for entry in data.lines() {
            let mut entry = entry.split_whitespace();
            match entry.next() {
                Some("V") | Some("v") => positions.push([
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                ]),
                Some("VT") | Some("vt") => textures.push([
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                ]),
                Some("VN") | Some("vn") => normals.push([
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_f32)?,
                ]),
                Some("VC") | Some("vc") => colors.push([
                    entry
                        .next()
                        .map_or(Err(missing_error.clone()), parse_clamp_f32)?,
                    entry
                        .next()
                        .map_or(Err(missing_error.clone()), parse_clamp_f32)?,
                    entry
                        .next()
                        .map_or(Err(missing_error.clone()), parse_clamp_f32)?,
                    entry.next().map_or(Ok(1.0), parse_clamp_f32)?,
                ]),
                Some("F") | Some("f") => face_elements.push([
                    entry
                        .next()
                        .map_or(Err(missing_error.clone()), parse_face_element)?,
                    entry
                        .next()
                        .map_or(Err(missing_error.clone()), parse_face_element)?,
                    entry
                        .next()
                        .map_or(Err(missing_error.clone()), parse_face_element)?,
                ]),
                _ => {}
            }
        }

        let mut vertices = Vec::with_capacity(face_elements.len() * 3);
        let mut faces = Vec::with_capacity(face_elements.len());
        for face in face_elements {
            faces.push([
                vertices.len() as u32,
                vertices.len() as u32 + 1,
                vertices.len() as u32 + 2,
            ]);
            for element in face {
                if element[0] == 0 || element[0] > positions.len() {
                    return Err(ParseGeometryError {
                        message: format!("invalid element position index: {}", element[0]),
                    });
                }
                if element[1] > textures.len() {
                    return Err(ParseGeometryError {
                        message: format!("invalid element UV coordinate index: {}", element[1]),
                    });
                }
                if element[2] > normals.len() {
                    return Err(ParseGeometryError {
                        message: format!("invalid element normal index: {}", element[2]),
                    });
                }
                if element[3] > colors.len() {
                    return Err(ParseGeometryError {
                        message: format!("invalid element color index: {}", element[3]),
                    });
                }
                vertices.push(Vertex3D::new(
                    positions[element[0] - 1],
                    colors.get(element[3] - 1).copied(),
                    textures.get(element[1] - 1).copied(),
                    normals.get(element[2] - 1).copied(),
                ));
            }
        }

        Ok(Self::from_data(&vertices, &faces))
    }
}

impl<V: Vertex> Drop for Geometry<V> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}

pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        let size_bytes = (width * height * 4) as usize;
        Self {
            data: Vec::from_iter(std::iter::repeat(0).take(size_bytes)),
            width,
            height,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let input = image::io::Reader::open(path)
            .map_err(|err| format!("Image::from_file(): failed to open '{path}'. ({err})"))?
            .decode()
            .map_err(|err| format!("Image::from_file(): failed to decode '{path}'. ({err})"))?;
        Ok(Self {
            data: input.to_rgba8().to_vec(),
            width: input.width(),
            height: input.height(),
        })
    }

    pub fn new_atlas(images: &[Self]) -> (Self, Vec<Vector<u32, 2>>) {
        let atlas_width = images.iter().map(Self::width).max().unwrap_or(0);
        let atlas_height = images.iter().map(Self::height).sum();

        let mut atlas = Self::new(atlas_width, atlas_height);
        let mut image_origins = Vec::with_capacity(images.len());

        let mut origin_y = 0;
        for image in images {
            let mut row_offset = (origin_y * atlas_width * 4) as usize;
            for row_data in image.data().chunks_exact((image.width() * 4) as usize) {
                atlas.data_mut()[row_offset .. (row_offset + row_data.len())]
                    .copy_from_slice(row_data);
                row_offset += (atlas_width * 4) as usize;
            }

            image_origins.push(Vector([0, origin_y]));
            origin_y += image.height();
        }

        (atlas, image_origins)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TextureSampling {
    Nearest = gl::NEAREST as isize,
    Linear = gl::LINEAR as isize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TextureWrap {
    ClampToEdge = gl::CLAMP_TO_EDGE as isize,
    ClampToBorder = gl::CLAMP_TO_BORDER as isize,
    MirroredRepeat = gl::MIRRORED_REPEAT as isize,
    Repeat = gl::REPEAT as isize,
    MirrorClampToEdge = gl::MIRROR_CLAMP_TO_EDGE as isize,
}

pub struct Texture2D {
    id: GLuint,
    slot: GLuint,
}

impl Texture2D {
    pub fn new(slot: GLuint) -> Self {
        let mut texture = Self { id: 0, slot };
        unsafe {
            gl::GenTextures(1, &mut texture.id);
        }
        texture
    }

    pub fn set_parameter_int(&mut self, parameter: GLenum, value: GLint) {
        self.bind();
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, parameter, value);
        }
    }

    pub fn set_minify_filter(&mut self, sampling: TextureSampling) {
        self.set_parameter_int(gl::TEXTURE_MIN_FILTER, sampling as GLint);
    }

    pub fn set_magnify_filter(&mut self, sampling: TextureSampling) {
        self.set_parameter_int(gl::TEXTURE_MAG_FILTER, sampling as GLint);
    }

    pub fn set_wrap_s(&mut self, wrap: TextureWrap) {
        self.set_parameter_int(gl::TEXTURE_WRAP_S, wrap as GLint);
    }

    pub fn set_wrap_t(&mut self, wrap: TextureWrap) {
        self.set_parameter_int(gl::TEXTURE_WRAP_T, wrap as GLint);
    }

    pub fn load_from_image(&mut self, image: &Image) {
        self.bind();
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                image.width() as GLsizei,
                image.height() as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.data().as_ptr() as *const GLvoid,
            );
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn slot(&self) -> GLuint {
        self.slot
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + self.slot);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
