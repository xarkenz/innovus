pub mod screen;
pub mod color;

use super::tools::{Matrix, Transform3D, Vector};
use gl::types::*;
use std::error::Error;
use std::ffi::CString;
use std::mem::{offset_of, size_of, swap};
use std::num::{ParseFloatError, ParseIntError};
use std::path::Path;
use std::str::FromStr;

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
    attached_shader_ids: Vec<GLuint>,
}

impl Program {
    pub fn create() -> Result<Self, String> {
        let id = unsafe { gl::CreateProgram() };
        if id == 0 {
            Err("Program::new(): failed to create GL program.".into())
        }
        else {
            Ok(Self {
                id,
                attached_shader_ids: Vec::new(),
            })
        }
    }

    pub fn attach_shader(&mut self, shader: &Shader) {
        if self.attached_shader_ids.contains(&shader.id()) {
            // This would normally generate GL_INVALID_OPERATION, but we can just let it slide
            return;
        }
        unsafe {
            gl::AttachShader(self.id, shader.id());
        }
        self.attached_shader_ids.push(shader.id());
    }

    pub fn link(&mut self) -> Result<(), String> {
        unsafe {
            gl::LinkProgram(self.id);
        }

        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success);
        }
        if success == 0 {
            return Err(self.get_info_log());
        }

        for &shader_id in &self.attached_shader_ids {
            unsafe {
                gl::DetachShader(self.id, shader_id);
            }
        }
        self.attached_shader_ids = Vec::new();

        Ok(())
    }

    pub fn with_linked_shaders<'a>(shaders: impl IntoIterator<Item = &'a Shader>) -> Result<Self, String> {
        let mut program = Self::create()?;
        for shader in shaders {
            program.attach_shader(shader);
        }
        program.link()?;
        Ok(program)
    }

    pub fn from_preset(preset: ProgramPreset) -> Result<Self, String> {
        let fetch_source = |path| std::fs::read_to_string(path)
            .map_err(|err| err.to_string());

        Self::with_linked_shaders(&match preset {
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
            String::from_utf8_lossy(&info_log).into()
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum VertexAttributeType {
    F32 = gl::FLOAT as isize,
    I32 = gl::INT as isize,
    U32 = gl::UNSIGNED_INT as isize,
}

pub struct VertexAttribute {
    pub data_type: VertexAttributeType,
    pub component_count: usize,
    pub offset: usize,
}

impl VertexAttribute {
    pub const fn new(data_type: VertexAttributeType, count: usize, offset: usize) -> Self {
        Self {
            data_type,
            component_count: count,
            offset,
        }
    }
}

pub trait Vertex : Clone {
    const ATTRIBUTES: &'static [VertexAttribute];
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertex3D {
    pub pos: Vector<f32, 3>,
    pub color: Vector<f32, 4>,
    pub tex: u32,
    pub uv: Vector<f32, 2>,
    pub norm: Vector<f32, 3>,
}

impl Vertex3D {
    pub fn new(
        pos: Vector<f32, 3>,
        color: Option<Vector<f32, 4>>,
        uv: Option<Vector<f32, 2>>,
        norm: Option<Vector<f32, 3>>,
    ) -> Self {
        Self {
            pos,
            color: color.unwrap_or(Vector::one()),
            tex: uv.is_some() as u32,
            uv: uv.unwrap_or(Vector::zero()),
            norm: norm.unwrap_or(Vector::zero()),
        }
    }

    pub fn colored(pos: Vector<f32, 3>, color: Vector<f32, 4>) -> Vertex3D {
        Vertex3D {
            pos,
            color,
            tex: 0,
            uv: Vector::zero(),
            norm: Vector::zero(),
        }
    }

    pub fn textured(pos: Vector<f32, 3>, uv: Vector<f32, 2>) -> Vertex3D {
        Vertex3D {
            pos,
            color: Vector::one(),
            tex: 1,
            uv,
            norm: Vector::zero(),
        }
    }

    pub fn combined(pos: Vector<f32, 3>, color: Vector<f32, 4>, uv: Vector<f32, 2>) -> Vertex3D {
        Vertex3D {
            pos,
            color,
            tex: 1,
            uv,
            norm: Vector::zero(),
        }
    }

    pub fn has_norm(&self) -> bool {
        self.norm != Vector::zero()
    }
}

impl Vertex for Vertex3D {
    const ATTRIBUTES: &'static [VertexAttribute] = &[
        VertexAttribute::new(VertexAttributeType::F32, 3, offset_of!(Self, pos)),
        VertexAttribute::new(VertexAttributeType::F32, 4, offset_of!(Self, color)),
        VertexAttribute::new(VertexAttributeType::U32, 1, offset_of!(Self, tex)),
        VertexAttribute::new(VertexAttributeType::F32, 2, offset_of!(Self, uv)),
        VertexAttribute::new(VertexAttributeType::F32, 3, offset_of!(Self, norm)),
    ];
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertex2D {
    pub pos: Vector<f32, 3>,
    pub color: Vector<f32, 4>,
    pub tex: u32,
    pub uv: Vector<f32, 2>,
}

impl Vertex2D {
    pub fn new(pos: Vector<f32, 3>, color: Option<Vector<f32, 4>>, uv: Option<Vector<f32, 2>>) -> Vertex2D {
        Vertex2D {
            pos,
            color: color.unwrap_or(Vector::one()),
            tex: uv.is_some() as u32,
            uv: uv.unwrap_or(Vector::zero()),
        }
    }
}

impl Vertex for Vertex2D {
    const ATTRIBUTES: &'static [VertexAttribute] = &[
        VertexAttribute::new(VertexAttributeType::F32, 3, offset_of!(Self, pos)),
        VertexAttribute::new(VertexAttributeType::F32, 4, offset_of!(Self, color)),
        VertexAttribute::new(VertexAttributeType::U32, 1, offset_of!(Self, tex)),
        VertexAttribute::new(VertexAttributeType::F32, 2, offset_of!(Self, uv)),
    ];
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
    vertices: Vec<V>,
    triangles: Vec<[u32; 3]>,
}

impl<V: Vertex> Geometry<V> {
    pub fn new() -> Self {
        Self {
            vao: 0,
            vbo: 0,
            ebo: 0,
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    pub fn with_data(vertices: Vec<V>, triangles: Vec<[u32; 3]>) -> Self {
        Self {
            vao: 0,
            vbo: 0,
            ebo: 0,
            vertices,
            triangles,
        }
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

            for (index, attribute) in V::ATTRIBUTES.iter().enumerate() {
                gl::EnableVertexAttribArray(index as GLuint);
                match attribute.data_type {
                    VertexAttributeType::I32 | VertexAttributeType::U32 => {
                        gl::VertexAttribIPointer(
                            index as GLuint,
                            attribute.component_count as GLint,
                            attribute.data_type as GLenum,
                            size_of::<V>() as GLsizei,
                            attribute.offset as *const GLvoid,
                        );
                    }
                    _ => {
                        gl::VertexAttribPointer(
                            index as GLuint,
                            attribute.component_count as GLint,
                            attribute.data_type as GLenum,
                            gl::FALSE,
                            size_of::<V>() as GLsizei,
                            attribute.offset as *const GLvoid,
                        );
                    }
                }
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

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.triangles.is_empty()
    }

    pub fn vertices(&self) -> &[V] {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut [V] {
        &mut self.vertices
    }

    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    pub fn triangles_mut(&mut self) -> &mut [[u32; 3]] {
        &mut self.triangles
    }

    pub fn render(&self) {
        if self.vao == 0 {
            panic!("Geometry::render(): must call 'Geometry::enable_render()' before rendering.");
        }

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.triangles.len() as GLsizei * 3,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.triangles.clear();

        self.update_vertex_buffer();
        self.update_element_buffer();
    }

    pub fn update_vertex_buffer(&self) {
        if self.vbo != 0 {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.len() * size_of::<V>()) as GLsizeiptr,
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
                    (self.triangles.len() * size_of::<[u32; 3]>()) as GLsizeiptr,
                    self.triangles.as_ptr() as *const GLvoid,
                    gl::DYNAMIC_DRAW,
                );
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn update_buffer_slice(&self, slice: &GeometrySlice) {
        if slice.vertex_count != 0 {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    (slice.first_vertex * size_of::<V>()) as GLintptr,
                    (slice.vertex_count * size_of::<V>()) as GLsizeiptr,
                    self.vertices[slice.first_vertex..(slice.first_vertex + slice.vertex_count)]
                        .as_ptr() as *const GLvoid,
                );
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
        if slice.face_count != 0 {
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::BufferSubData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (slice.first_face * size_of::<[u32; 3]>()) as GLintptr,
                    (slice.face_count * size_of::<[u32; 3]>()) as GLsizeiptr,
                    self.triangles[slice.first_face..(slice.first_face + slice.face_count)]
                        .as_ptr() as *const GLvoid,
                );
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn vertex_at(&self, index: usize) -> &V {
        &self.vertices[index]
    }

    pub fn vertex_at_mut(&mut self, index: usize) -> &mut V {
        &mut self.vertices[index]
    }

    pub fn add(&mut self, vertices: &[V], triangles: &[[u32; 3]]) -> GeometrySlice {
        let first_vertex = self.vertices.len();
        let vertex_count = vertices.len();
        let first_face = self.triangles.len();
        let face_count = triangles.len();

        if !triangles.is_empty() {
            for &face in triangles {
                self.triangles.push(face.map(|index| first_vertex as u32 + index))
            }
            self.update_element_buffer();
        }

        if !vertices.is_empty() {
            self.vertices.extend_from_slice(vertices);
            self.update_vertex_buffer();
        }

        GeometrySlice {
            first_vertex,
            vertex_count,
            first_face,
            face_count,
        }
    }

    pub fn add_geometry(&mut self, geometry: &Self) -> GeometrySlice {
        self.add(geometry.vertices(), geometry.triangles())
    }

    pub fn as_slice(&self) -> GeometrySlice {
        GeometrySlice {
            first_vertex: 0,
            vertex_count: self.vertices.len(),
            first_face: 0,
            face_count: self.triangles.len(),
        }
    }
}

impl Geometry<Vertex3D> {
    fn generate_icosahedron(
        center: Vector<f32, 3>,
        radius: f32,
        color: Vector<f32, 4>,
    ) -> (Vec<Vertex3D>, Vec<[u32; 3]>) {
        const MINOR: f32 = 0.525731112119133606;
        const MAJOR: f32 = 0.850650808352039932;
        let minor = MINOR * radius;
        let major = MAJOR * radius;
        (
            vec![
                Vertex3D::new(
                    center + Vector([-minor, 0.0, major]),
                    Some(color),
                    None,
                    Some(Vector([-MINOR, 0.0, MAJOR])),
                ),
                Vertex3D::new(
                    center + Vector([minor, 0.0, major]),
                    Some(color),
                    None,
                    Some(Vector([MINOR, 0.0, MAJOR])),
                ),
                Vertex3D::new(
                    center + Vector([-minor, 0.0, -major]),
                    Some(color),
                    None,
                    Some(Vector([-MINOR, 0.0, -MAJOR])),
                ),
                Vertex3D::new(
                    center + Vector([minor, 0.0, -major]),
                    Some(color),
                    None,
                    Some(Vector([MINOR, 0.0, -MAJOR])),
                ),
                Vertex3D::new(
                    center + Vector([0.0, major, minor]),
                    Some(color),
                    None,
                    Some(Vector([0.0, MAJOR, MINOR])),
                ),
                Vertex3D::new(
                    center + Vector([0.0, major, -minor]),
                    Some(color),
                    None,
                    Some(Vector([0.0, MAJOR, -MINOR])),
                ),
                Vertex3D::new(
                    center + Vector([0.0, -major, minor]),
                    Some(color),
                    None,
                    Some(Vector([0.0, -MAJOR, MINOR])),
                ),
                Vertex3D::new(
                    center + Vector([0.0, -major, -minor]),
                    Some(color),
                    None,
                    Some(Vector([0.0, -MAJOR, -MINOR])),
                ),
                Vertex3D::new(
                    center + Vector([major, minor, 0.0]),
                    Some(color),
                    None,
                    Some(Vector([MAJOR, MINOR, 0.0])),
                ),
                Vertex3D::new(
                    center + Vector([-major, minor, 0.0]),
                    Some(color),
                    None,
                    Some(Vector([-MAJOR, MINOR, 0.0])),
                ),
                Vertex3D::new(
                    center + Vector([major, -minor, 0.0]),
                    Some(color),
                    None,
                    Some(Vector([MAJOR, -MINOR, 0.0])),
                ),
                Vertex3D::new(
                    center + Vector([-major, -minor, 0.0]),
                    Some(color),
                    None,
                    Some(Vector([-MAJOR, -MINOR, 0.0])),
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
        color: Vector<f32, 4>,
        subdivisions: u32,
    ) -> GeometrySlice {
        let (mut vertices, mut faces) = Self::generate_icosahedron(center, radius, color);

        for _ in 0..subdivisions {
            let mut add_vertices: Vec<Vertex3D> = Vec::new();
            let mut new_faces: Vec<[u32; 3]> = Vec::new();
            let mut edge_midpoint_map: Vec<((u32, u32), u32)> = Vec::new();
            for face in faces {
                let mut fetch_midpoint = |mut v1, mut v2| {
                    if v1 > v2 {
                        swap(&mut v1, &mut v2);
                    }
                    for (edge, midpoint) in edge_midpoint_map.iter() {
                        if edge.0 == v1 && edge.1 == v2 {
                            return *midpoint;
                        }
                    }
                    let idx = (vertices.len() + add_vertices.len()) as u32;
                    let normal = (vertices[v1 as usize].norm + vertices[v2 as usize].norm).normalized();
                    add_vertices.push(Vertex3D::new(
                        normal * radius + center,
                        Some(color),
                        None,
                        Some(normal),
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

    pub fn transform(&mut self, slice: &GeometrySlice, matrix: Transform3D<f32>) {
        for index in 0..slice.vertex_count {
            let index = slice.first_vertex + index;
            let vertex = self.vertex_at_mut(index);
            vertex.pos = matrix * vertex.pos;
            if vertex.has_norm() {
                vertex.norm = matrix.affine() * vertex.norm;
            }
        }
        self.update_buffer_slice(slice);
    }

    pub fn rotate_x(&mut self, slice: &GeometrySlice, angle: f32, axis_y: f32, axis_z: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(Vector([0.0, axis_y, axis_z]));
        rotation.rotate_x(angle);
        rotation.translate(Vector([0.0, -axis_y, -axis_z]));
        self.transform(slice, rotation);
    }

    pub fn rotate_y(&mut self, slice: &GeometrySlice, angle: f32, axis_x: f32, axis_z: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(Vector([axis_x, 0.0, axis_z]));
        rotation.rotate_y(angle);
        rotation.translate(Vector([-axis_x, 0.0, -axis_z]));
        self.transform(slice, rotation);
    }

    pub fn rotate_z(&mut self, slice: &GeometrySlice, angle: f32, axis_x: f32, axis_y: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(Vector([axis_x, axis_y, 0.0]));
        rotation.rotate_z(angle);
        rotation.translate(Vector([-axis_x, -axis_y, 0.0]));
        self.transform(slice, rotation);
    }
}

#[derive(Clone, Debug)]
pub struct ParseGeometryError {
    message: String,
}

impl Error for ParseGeometryError {}

impl std::fmt::Display for ParseGeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        fn parse_f32(s: &str) -> Result<f32, ParseGeometryError> {
            s.parse::<f32>().map_err(ParseGeometryError::from)
        }
        fn parse_clamp_f32(s: &str) -> Result<f32, ParseGeometryError> {
            match parse_f32(s)? {
                num if 0.0 <= num && num <= 1.0 => Ok(num),
                _ => Err(ParseGeometryError {
                    message: format!("'{}' not in range 0-1", s),
                }),
            }
        }
        fn parse_usize(s: &str) -> Result<usize, ParseGeometryError> {
            s.parse::<usize>().map_err(ParseGeometryError::from)
        }
        fn parse_face_element(s: &str) -> Result<[usize; 4], ParseGeometryError> {
            let mut indices: [usize; 4] = [0; 4];
            let mut next_index: usize = 0;
            let mut index_start: Option<usize> = None;
            for (i, c) in s.chars().enumerate() {
                if c.is_ascii_digit() {
                    if index_start.is_none() {
                        index_start = Some(i);
                    }
                }
                else {
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
            }
            else {
                Ok(indices)
            }
        }
        fn missing_error<T>() -> Result<T, ParseGeometryError> {
            Err(ParseGeometryError { message: "missing command argument".into() })
        }

        let mut positions: Vec<Vector<f32, 3>> = Vec::new();
        let mut textures: Vec<Vector<f32, 2>> = Vec::new();
        let mut normals: Vec<Vector<f32, 3>> = Vec::new();
        let mut colors: Vec<Vector<f32, 4>> = Vec::new();
        let mut face_elements: Vec<[[usize; 4]; 3]> = Vec::new();

        for entry in data.lines() {
            let mut entry = entry.split_whitespace();
            match entry.next() {
                Some("V") | Some("v") => positions.push(Vector([
                    entry.next().map_or_else(missing_error, parse_f32)?,
                    entry.next().map_or_else(missing_error, parse_f32)?,
                    entry.next().map_or_else(missing_error, parse_f32)?,
                ])),
                Some("VT") | Some("vt") => textures.push(Vector([
                    entry.next().map_or_else(missing_error, parse_f32)?,
                    entry.next().map_or_else(missing_error, parse_f32)?,
                ])),
                Some("VN") | Some("vn") => normals.push(Vector([
                    entry.next().map_or_else(missing_error, parse_f32)?,
                    entry.next().map_or_else(missing_error, parse_f32)?,
                    entry.next().map_or_else(missing_error, parse_f32)?,
                ])),
                Some("VC") | Some("vc") => colors.push(Vector([
                    entry.next().map_or_else(missing_error, parse_clamp_f32)?,
                    entry.next().map_or_else(missing_error, parse_clamp_f32)?,
                    entry.next().map_or_else(missing_error, parse_clamp_f32)?,
                    entry.next().map_or(Ok(1.0), parse_clamp_f32)?,
                ])),
                Some("F") | Some("f") => face_elements.push([
                    entry.next().map_or_else(missing_error, parse_face_element)?,
                    entry.next().map_or_else(missing_error, parse_face_element)?,
                    entry.next().map_or_else(missing_error, parse_face_element)?,
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

        Ok(Self::with_data(vertices, faces))
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
        let mut data = Vec::with_capacity(width as usize * height as usize * 4);
        data.resize(data.capacity(), 0);
        Self {
            data,
            width,
            height,
        }
    }

    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let input = image::ImageReader::open(path)
            .map_err(|err| format!("failed to open image at '{}'. ({err})", path.display()))?
            .decode()
            .map_err(|err| format!("failed to decode image at '{}'. ({err})", path.display()))?;
        Ok(Self {
            data: input.to_rgba8().to_vec(),
            width: input.width(),
            height: input.height(),
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> Vector<u32, 2> {
        Vector([self.width, self.height])
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub enum ImageAtlasFlow {
    LeftToRight,
    #[default] // Most efficient to construct
    TopToBottom,
}

pub struct ImageAtlas {
    image: Image,
    flow: ImageAtlasFlow,
    flow_x: u32,
    flow_y: u32,
}

impl ImageAtlas {
    pub fn new(flow: ImageAtlasFlow) -> Self {
        Self {
            image: Image {
                data: Vec::new(),
                width: 0,
                height: 0,
            },
            flow,
            flow_y: 0,
            flow_x: 0,
        }
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn image_mut(&mut self) -> &mut Image {
        &mut self.image
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn size(&self) -> Vector<u32, 2> {
        self.image.size()
    }

    pub fn data(&self) -> &[u8] {
        self.image.data()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.image.data_mut()
    }

    pub fn add_image(&mut self, image: &Image) -> Vector<u32, 2> {
        let mut flow_x = self.flow_x;
        let mut flow_y = self.flow_y;

        // Check bounds and calculate the new flow point
        match self.flow {
            ImageAtlasFlow::LeftToRight => {
                self.flow_x = flow_x.checked_add(image.width()).unwrap_or_else(|| {
                    // Overflow into a new row
                    self.flow_y = self.height();
                    flow_y = self.flow_y;
                    flow_x = 0;
                    image.width()
                })
            }
            ImageAtlasFlow::TopToBottom => {
                self.flow_y = flow_y.checked_add(image.height()).unwrap_or_else(|| {
                    // Overflow into a new column
                    self.flow_x = self.width();
                    flow_x = self.flow_x;
                    flow_y = 0;
                    image.height()
                })
            }
        }

        if flow_x + image.width() > self.width() {
            // Expand the atlas width (and maybe height). Might as well just make a new array, lol
            let expanded_width = flow_x + image.width();
            let expanded_height = self.height().max(flow_y + image.height());
            let mut expanded_data = Vec::with_capacity(expanded_width as usize * expanded_height as usize * 4);
            expanded_data.resize(expanded_data.capacity(), 0);
            // Copy from old data to new data, row by row
            for y in 0 .. self.height() as usize {
                let src_start = y * self.width() as usize * 4;
                let dst_start = y * expanded_width as usize * 4;
                let length = self.width() as usize * 4;
                expanded_data[dst_start .. dst_start + length]
                    .copy_from_slice(&self.data()[src_start .. src_start + length]);
            }
            // Replace the old image data
            self.image.data = expanded_data;
            self.image.width = expanded_width;
            self.image.height = expanded_height;
        }
        else if flow_y + image.height() > self.height() {
            // Only expand the atlas in the Y direction. No need for mass copying in this case
            self.image.height = flow_y + image.height();
            self.image.data.resize(self.width() as usize * self.height() as usize * 4, 0);
        }

        // Write the new image into place, row by row
        for y_offset in 0 .. image.height() as usize {
            let src_start = y_offset * image.width() as usize * 4;
            let dst_start = ((flow_y as usize + y_offset) * self.width() as usize * 4) + flow_x as usize * 4;
            let length = image.width() as usize * 4;
            self.data_mut()[dst_start .. dst_start + length]
                .copy_from_slice(&image.data()[src_start .. src_start + length]);
        }

        Vector([flow_x, flow_y])
    }

    pub fn next_flow(&mut self) {
        match self.flow {
            ImageAtlasFlow::LeftToRight => {
                self.flow_y = self.height();
                self.flow_x = 0;
            }
            ImageAtlasFlow::TopToBottom => {
                self.flow_x = self.width();
                self.flow_y = 0;
            }
        }
    }

    pub fn clear(&mut self) {
        self.image.data.clear();
        self.image.width = 0;
        self.image.height = 0;
        self.flow_x = 0;
        self.flow_y = 0;
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

    pub fn upload_image(&mut self, image: &Image) {
        self.bind();
        unsafe {
            // Allocate a texture whose size is the next power of 2 for each dimension.
            // This is useful for e.g. texture atlases, where floating-point precision is important.
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                image.width().next_power_of_two() as GLsizei,
                image.height().next_power_of_two() as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            // Download the image onto the texture
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                image.width() as GLsizei,
                image.height() as GLsizei,
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
