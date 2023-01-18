extern crate gl;
extern crate glfw;
extern crate image;

pub mod screen;

use std::mem::{size_of, swap};
use std::ffi::CString;
use std::fmt;
use std::str::FromStr;
use std::marker::PhantomData;
use gl::types::*;
use super::util::{Vector, Matrix, Transform3D};


fn whitespace_cstring(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}


pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize,
    Compute = gl::COMPUTE_SHADER as isize,
    Geometry = gl::GEOMETRY_SHADER as isize,
    TessControl = gl::TESS_CONTROL_SHADER as isize,
    TessEval = gl::TESS_EVALUATION_SHADER as isize,
}

pub struct Shader {
    id: GLuint,
}

impl Shader {

    pub fn create(source: &str, shader_type: ShaderType) -> Result<Shader, String> {
        let cstring_source = CString::new(source).map_err(|err| err.to_string())?;

        let id = unsafe { gl::CreateShader(shader_type as GLenum) };
        if id == 0 {
            return Err("Shader::create(): failed to create GL shader object.".to_string());
        }

        unsafe {
            gl::ShaderSource(id, 1, &cstring_source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut success: GLint = 1;
        unsafe { gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success); }
        if success == 0 {
            let mut len: GLint = 0;
            unsafe { gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len); }
            let error = whitespace_cstring(len as usize);
            unsafe { gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar); }
            return Err(error.to_string_lossy().into_owned());
        }

        Ok(Shader{ id })
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

}

impl Drop for Shader {

    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }

}


pub trait ShaderUniformType {
    fn upload_uniform(self, location: GLint);
}

impl ShaderUniformType for GLfloat {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform1f(location, self as GLfloat); }
    }
}

impl ShaderUniformType for GLint {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform1i(location, self as GLint); }
    }
}

impl ShaderUniformType for GLuint {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform1ui(location, self as GLuint); }
    }
}

impl ShaderUniformType for GLboolean {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform1ui(location, self as GLuint); }
    }
}

impl ShaderUniformType for &Vector<2> {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform2f(location, self.at(0) as GLfloat, self.at(1) as GLfloat); }
    }
}

impl ShaderUniformType for &Vector<3> {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform3f(location, self.at(0) as GLfloat, self.at(1) as GLfloat, self.at(2) as GLfloat); }
    }
}

impl ShaderUniformType for &Vector<4> {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform4f(location, self.at(0) as GLfloat, self.at(1) as GLfloat, self.at(2) as GLfloat, self.at(3) as GLfloat); }
    }
}

impl ShaderUniformType for &Matrix<2, 2> {
    fn upload_uniform(self, location: GLint) {
        let data = self.data();
        unsafe { gl::UniformMatrix2fv(location, 1, gl::TRUE, data.as_ptr() as *const GLfloat); }
    }
}

impl ShaderUniformType for &Matrix<3, 3> {
    fn upload_uniform(self, location: GLint) {
        let data = self.data();
        unsafe { gl::UniformMatrix3fv(location, 1, gl::TRUE, data.as_ptr() as *const GLfloat); }
    }
}

impl ShaderUniformType for &Matrix<4, 4> {
    fn upload_uniform(self, location: GLint) {
        let data = self.data();
        unsafe { gl::UniformMatrix4fv(location, 1, gl::TRUE, data.as_ptr() as *const GLfloat); }
    }
}

impl ShaderUniformType for &Matrix<4, 2> {
    fn upload_uniform(self, location: GLint) {
        let data = self.data();
        unsafe { gl::UniformMatrix2x4fv(location, 1, gl::TRUE, data.as_ptr() as *const GLfloat); }
    }
}

impl ShaderUniformType for &Matrix<2, 4> {
    fn upload_uniform(self, location: GLint) {
        let data = self.data();
        unsafe { gl::UniformMatrix4x2fv(location, 1, gl::TRUE, data.as_ptr() as *const GLfloat); }
    }
}

impl ShaderUniformType for &Matrix<4, 3> {
    fn upload_uniform(self, location: GLint) {
        let data = self.data();
        unsafe { gl::UniformMatrix3x4fv(location, 1, gl::TRUE, data.as_ptr() as *const GLfloat); }
    }
}

impl ShaderUniformType for &Matrix<3, 4> {
    fn upload_uniform(self, location: GLint) {
        let data = self.data();
        unsafe { gl::UniformMatrix4x3fv(location, 1, gl::TRUE, data.as_ptr() as *const GLfloat); }
    }
}

impl ShaderUniformType for &Texture {
    fn upload_uniform(self, location: GLint) {
        unsafe { gl::Uniform1ui(location, self.slot().expect("attempted to set an unbound texture uniform.")); }
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

    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let id = unsafe { gl::CreateProgram() };
        if id == 0 {
            return Err("Program::from_shaders(): failed to create GL program.".to_string());
        }

        for shader in shaders {
            unsafe { gl::AttachShader(id, shader.id()); }
        }

        unsafe { gl::LinkProgram(id); }

        let mut success: GLint = 1;
        unsafe { gl::GetProgramiv(id, gl::LINK_STATUS, &mut success); }
        if success == 0 {
            let mut len: GLint = 0;
            unsafe { gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len); }
            let error = whitespace_cstring(len as usize);
            unsafe { gl::GetProgramInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar); }
            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(id, shader.id()); }
        }

        Ok(Program{ id })
    }

    pub fn from_preset(preset: ProgramPreset) -> Result<Program, String> {
        let get_source = |path| std::fs::read_to_string(path).map_err(|err| err.to_string());
        Program::from_shaders(&match preset {
            ProgramPreset::Default2DShader => vec![
                Shader::create(&get_source("./src/innovus/assets/default2d.v.glsl")?, ShaderType::Vertex)?,
                Shader::create(&get_source("./src/innovus/assets/default2d.f.glsl")?, ShaderType::Fragment)?,
            ],
            ProgramPreset::Default3DShader => vec![
                Shader::create(&get_source("./src/innovus/assets/default3d.v.glsl")?, ShaderType::Vertex)?,
                Shader::create(&get_source("./src/innovus/assets/default3d.g.glsl")?, ShaderType::Geometry)?,
                Shader::create(&get_source("./src/innovus/assets/default3d.f.glsl")?, ShaderType::Fragment)?,
            ],
        })
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.id); }
    }

    pub fn set(&self, name: &str, value: impl ShaderUniformType) {
        let cstring_name = CString::new(name).unwrap();
        unsafe { gl::UseProgram(self.id); }
        value.upload_uniform(unsafe { gl::GetUniformLocation(self.id, cstring_name.as_ptr() as *const GLchar) });
    }

}

impl Drop for Program {

    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
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

    pub fn new(pos: [f32; 3], color: Option<[f32; 4]>, uv: Option<[f32; 2]>, norm: Option<[f32; 3]>) -> Vertex3D {
        Vertex3D {
            pos,
            color: color.unwrap_or([1.0; 4]),
            tex: uv.is_some(),
            uv: uv.unwrap_or([0.0; 2]),
            norm: norm.unwrap_or([0.0; 3]),
        }
    }

    pub fn colored(pos: [f32; 3], color: [f32; 4]) -> Vertex3D {
        Vertex3D { pos, color, tex: false, uv: [0.0; 2], norm: [0.0; 3] }
    }

    pub fn textured(pos: [f32; 3], uv: [f32; 2]) -> Vertex3D {
        Vertex3D { pos, color: [1.0; 4], tex: true, uv, norm: [0.0; 3] }
    }

    pub fn combined(pos: [f32; 3], color: [f32; 4], uv: [f32; 2]) -> Vertex3D {
        Vertex3D { pos, color, tex: true, uv, norm: [0.0; 3] }
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
            self.pos[0], self.pos[1], self.pos[2],
            self.color[0], self.color[1], self.color[2], self.color[3],
            if self.tex { 1.0 } else { 0.0 },
            self.uv[0], self.uv[1],
            self.norm[0], self.norm[1], self.norm[2],
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
            self.pos[0], self.pos[1], self.pos[2],
            self.color[0], self.color[1], self.color[2], self.color[3],
            if self.tex { 1.0 } else { 0.0 },
            self.uv[0], self.uv[1],
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


#[derive(Debug, Clone)]
pub struct GeometrySlice {
    first_vertex: usize,
    vertex_count: usize,
    first_face: usize,
    face_count: usize,
}


#[derive(Debug)]
pub struct Geometry<V: Vertex> {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    vertices: Vec<f32>,
    elements: Vec<GLuint>,
    vertex_count: usize,
    face_count: usize,
    vertex_type: PhantomData<V>,
}

impl<V: Vertex> Geometry<V> {

    pub fn new() -> Geometry<V> {
        Geometry {
            vao: 0, vbo: 0, ebo: 0,
            vertices: Vec::new(), elements: Vec::new(),
            vertex_count: 0, face_count: 0,
            vertex_type: PhantomData
        }
    }

    pub fn from_data(vertices: &[V], faces: &[[u32; 3]]) -> Geometry<V> {
        let mut geometry = Geometry::new();
        geometry.add(vertices, faces);
        geometry
    }

    pub fn enable_render(&mut self) -> Result<(), String> {
        unsafe {
            gl::GenVertexArrays(1, &mut self.vao);
            if self.vao == 0 {
                return Err("Geometry::enable_render(): failed to create GL vertex array object.".to_string());
            }
            gl::BindVertexArray(self.vao);

            gl::GenBuffers(1, &mut self.vbo);
            if self.vbo == 0 {
                return Err("Geometry::enable_render(): failed to create GL vertex buffer object.".to_string());
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::GenBuffers(1, &mut self.ebo);
            if self.ebo == 0 {
                return Err("Geometry::enable_render(): failed to create GL element buffer object.".to_string());
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
                    offset as *const GLvoid);
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
            gl::DrawElements(gl::TRIANGLES, self.elements.len() as GLsizei, gl::UNSIGNED_INT, std::ptr::null());
            gl::BindVertexArray(0);
        }
    }

    pub fn clear(&mut self) {
        if self.vbo != 0 && self.ebo != 0 {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferData(gl::ARRAY_BUFFER, 0 as GLsizeiptr, std::ptr::null(), gl::STATIC_DRAW);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 0 as GLsizeiptr, std::ptr::null(), gl::STATIC_DRAW);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }

        self.vertex_count = 0;
        self.face_count = 0;
        self.vertices.clear();
        self.vertices.shrink_to_fit();
        self.elements.clear();
        self.elements.shrink_to_fit();
    }

    pub fn update_vertex_buffer(&self) {
        if self.vbo != 0 {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferData(gl::ARRAY_BUFFER, (self.vertices.len() * size_of::<f32>()) as GLsizeiptr, self.vertices.as_ptr() as *const GLvoid, gl::DYNAMIC_DRAW);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn update_element_buffer(&self) {
        if self.ebo != 0 {
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (self.elements.len() * size_of::<GLuint>()) as GLsizeiptr, self.elements.as_ptr() as *const GLvoid, gl::DYNAMIC_DRAW);
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
                gl::BufferSubData(gl::ARRAY_BUFFER, (start * size_of::<f32>()) as GLintptr, (size * size_of::<f32>()) as GLsizeiptr, self.vertices[start..(start + size)].as_ptr() as *const GLvoid);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
        if slice.face_count != 0 {
            let start = slice.first_face * 3;
            let size = slice.face_count * 3;
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
                gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, (start * size_of::<GLuint>()) as GLintptr, (size * size_of::<GLuint>()) as GLsizeiptr, self.elements[start..(start + size)].as_ptr() as *const GLvoid);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            }
        }
    }

    pub fn get_vertex(&self, index: usize) -> V {
        V::from_raw_data(&self.vertices[(index * V::SIZE)..((index + 1) * V::SIZE)])
    }

    pub fn set_vertex(&mut self, index: usize, vertex: &V) {
        self.vertices[(index * V::SIZE)..((index + 1) * V::SIZE)].copy_from_slice(&vertex.to_raw_data());
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

        GeometrySlice { first_vertex, vertex_count, first_face, face_count }
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

        GeometrySlice { first_vertex, vertex_count, first_face, face_count }
    }

    pub fn as_slice(&self) -> GeometrySlice {
        GeometrySlice { first_vertex: 0, vertex_count: self.vertex_count, first_face: 0, face_count: self.face_count }
    }

}

impl Geometry<Vertex3D> {

    fn generate_icosahedron(cx: f32, cy: f32, cz: f32, r: f32, color: [f32; 4]) -> (Vec<Vertex3D>, Vec<[u32; 3]>) {
        const MINOR: f32 = 0.525731112119133606;
        const MAJOR: f32 = 0.850650808352039932;
        let minor = MINOR * r;
        let major = MAJOR * r;
        (vec![
            Vertex3D::new([cx - minor, cy, cz + major], Some(color.clone()), None, Some([-minor, 0.0,  major])),
            Vertex3D::new([cx + minor, cy, cz + major], Some(color.clone()), None, Some([ minor, 0.0,  major])),
            Vertex3D::new([cx - minor, cy, cz - major], Some(color.clone()), None, Some([-minor, 0.0, -major])),
            Vertex3D::new([cx + minor, cy, cz - major], Some(color.clone()), None, Some([ minor, 0.0, -major])),
            Vertex3D::new([cx, cy + major, cz + minor], Some(color.clone()), None, Some([0.0,  major,  minor])),
            Vertex3D::new([cx, cy + major, cz - minor], Some(color.clone()), None, Some([0.0,  major, -minor])),
            Vertex3D::new([cx, cy - major, cz + minor], Some(color.clone()), None, Some([0.0, -major,  minor])),
            Vertex3D::new([cx, cy - major, cz - minor], Some(color.clone()), None, Some([0.0, -major, -minor])),
            Vertex3D::new([cx + major, cy + minor, cz], Some(color.clone()), None, Some([ major,  minor, 0.0])),
            Vertex3D::new([cx - major, cy + minor, cz], Some(color.clone()), None, Some([-major,  minor, 0.0])),
            Vertex3D::new([cx + major, cy - minor, cz], Some(color.clone()), None, Some([ major, -minor, 0.0])),
            Vertex3D::new([cx - major, cy - minor, cz], Some(color.clone()), None, Some([-major, -minor, 0.0])),
        ], vec![
            [00, 01, 04], [00, 04, 09], [09, 04, 05], [04, 08, 05], [04, 01, 08],
            [08, 01, 10], [08, 10, 03], [05, 08, 03], [05, 03, 02], [02, 03, 07],
            [07, 03, 10], [07, 10, 06], [07, 06, 11], [11, 06, 00], [00, 06, 01],
            [06, 10, 01], [09, 11, 00], [09, 02, 11], [09, 05, 02], [07, 11, 02],
        ])
    }

    pub fn add_icosphere(&mut self, center: &Vector<3>, radius: f32, color: [f32; 4], subdivisions: u32) -> GeometrySlice {
        let (mut vertices, mut faces) = Geometry::generate_icosahedron(center.at(0), center.at(1), center.at(2), radius, color.clone());

        for _ in 0..subdivisions {
            let mut add_vertices: Vec<Vertex3D> = Vec::new();
            let mut new_faces: Vec<[u32; 3]> = Vec::new();
            let mut edge_midpoint_map: Vec<((u32, u32), u32)> = Vec::new();
            for face in faces {
                let mut fetch_midpoint = |v1, v2| {
                    let mut v1: u32 = v1;
                    let mut v2: u32 = v2;
                    if v1 > v2 { swap(&mut v1, &mut v2); }
                    for (edge, midpoint) in edge_midpoint_map.iter() {
                        if edge.0 == v1 && edge.1 == v2 { return *midpoint; }
                    }
                    let idx = (vertices.len() + add_vertices.len()) as u32;
                    let raw_pos = &(&(&Vector::new(vertices[v1 as usize].pos.clone()) - &center) + &(&Vector::new(vertices[v2 as usize].pos.clone()) - &center)).normalized() * radius;
                    let pos = &raw_pos + &center;
                    add_vertices.push(Vertex3D::new([pos.at(0), pos.at(1), pos.at(2)], Some(color.clone()), None, Some([raw_pos.at(0), raw_pos.at(1), raw_pos.at(2)])));
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

    pub fn transform(&mut self, slice: &GeometrySlice, matrix: &Transform3D) {
        for i in 0..slice.vertex_count {
            let i = slice.first_vertex + i;
            let mut vertex = self.get_vertex(i);
            let new_pos = matrix * &Vector::new(vertex.pos.clone());
            vertex.pos = [new_pos.at(0), new_pos.at(1), new_pos.at(2)];
            if vertex.has_norm() {
                let new_norm = &matrix.affine() * &Vector::new(vertex.norm.clone());
                vertex.norm = [new_norm.at(0), new_norm.at(1), new_norm.at(2)];
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
        self.transform(slice, &rotation);
    }

    pub fn rotate_y(&mut self, slice: &GeometrySlice, angle: f32, axis_x: f32, axis_z: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(axis_x, 0.0, axis_z);
        rotation.rotate_y(angle);
        rotation.translate(-axis_x, 0.0, -axis_z);
        self.transform(slice, &rotation);
    }

    pub fn rotate_z(&mut self, slice: &GeometrySlice, angle: f32, axis_x: f32, axis_y: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(axis_x, axis_y, 0.0);
        rotation.rotate_z(angle);
        rotation.translate(-axis_x, -axis_y, 0.0);
        self.transform(slice, &rotation);
    }

}

#[derive(Clone)]
pub struct ParseGeometryError {
    message: String,
}

impl fmt::Display for ParseGeometryError {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("ParseGeometryError: ")?;
        formatter.write_str(&self.message)
    }

}

impl fmt::Debug for ParseGeometryError {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("ParseGeometryError: ")?;
        formatter.write_str(&self.message)
    }

}

impl FromStr for Geometry<Vertex3D> {

    type Err = ParseGeometryError;

    fn from_str(data: & str) -> Result<Self, Self::Err> {
        let parse_f32 = |s: &str| match s.parse::<f32>() {
            Ok(num) => Ok(num),
            Err(err) => Err(ParseGeometryError { message: err.to_string() }),
        };
        let parse_clamp_f32 = |s: &str| match s.parse::<f32>() {
            Ok(num) if 0.0 <= num && num <= 1.0 => Ok(num),
            Ok(_) => Err(ParseGeometryError { message: format!("Geometry::from_str(): '{}' not in range 0-1", s) }),
            Err(err) => Err(ParseGeometryError { message: err.to_string() }),
        };
        let parse_usize = |s: &str| match s.parse::<usize>() {
            Ok(num) => Ok(num),
            Err(err) => Err(ParseGeometryError { message: err.to_string() }),
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
                            return Err(ParseGeometryError { message: "Geometry::from_str(): a maximum of 4 indices is allowed per face element".to_string() });
                        }
                    }
                }
            }
            if index_start.is_some() {
                indices[next_index] = parse_usize(&s[index_start.unwrap()..s.len()])?;
            }
            if indices[0] == 0 {
                Err(ParseGeometryError { message: "Geometry::from_str(): faces must have a nonzero element position specified".to_string() })
            } else {
                Ok(indices)
            }
        };

        let missing_error = ParseGeometryError { message: "Geometry::from_str(): missing command argument".to_string() };

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
                    entry.next().map_or(Err(missing_error.clone()), parse_clamp_f32)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_clamp_f32)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_clamp_f32)?,
                    entry.next().map_or(Ok(1.0), parse_clamp_f32)?,
                ]),
                Some("F") | Some("f") => face_elements.push([
                    entry.next().map_or(Err(missing_error.clone()), parse_face_element)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_face_element)?,
                    entry.next().map_or(Err(missing_error.clone()), parse_face_element)?,
                ]),
                _ => {}
            }
        }

        let mut vertices = Vec::with_capacity(face_elements.len() * 3);
        let mut faces = Vec::with_capacity(face_elements.len());
        for face in face_elements {
            faces.push([vertices.len() as u32, vertices.len() as u32 + 1, vertices.len() as u32 + 2]);
            for element in face {
                if element[0] == 0 || element[0] > positions.len() {
                    return Err(ParseGeometryError { message: format!("Geometry::from_str(): invalid element position index: {}", element[0]) });
                }
                if element[1] > textures.len() {
                    return Err(ParseGeometryError { message: format!("Geometry::from_str(): invalid element texture coordinate index: {}", element[1]) });
                }
                if element[2] > normals.len() {
                    return Err(ParseGeometryError { message: format!("Geometry::from_str(): invalid element normal index: {}", element[2]) });
                }
                if element[3] > colors.len() {
                    return Err(ParseGeometryError { message: format!("Geometry::from_str(): invalid element color index: {}", element[3]) });
                }
                vertices.push(Vertex3D::new(
                    positions[element[0] - 1],
                    colors.get(element[3] - 1).copied(),
                    textures.get(element[1] - 1).copied(),
                    normals.get(element[2] - 1).copied(),
                ));
            }
        }

        Ok(Geometry::from_data(&vertices, &faces))
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

    pub fn from_file(path: &str) -> Result<Image, String> {
        let input = image::io::Reader::open(path)
            .map_err(|_err| format!("Image::from_file(): failed to open '{}'.", path))?
            .decode()
            .map_err(|_err| format!("Image::from_file(): failed to decode '{}'.", path))?;
        Ok(Image { data: input.to_rgba8().to_vec(), width: input.width(), height: input.height() })
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

}


pub struct Texture {
    id: GLuint,
    slot: Option<GLuint>,
}

impl<'img> Texture {

    pub fn from_image(image: &'img Image) -> Result<Texture, String> { // TODO: options for wrapping and min/mag filters
        let mut tex = Texture { id: 0, slot: None };
        unsafe {
            gl::GenTextures(1, &mut tex.id);
            gl::BindTexture(gl::TEXTURE_2D, tex.id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, image.width() as GLsizei, image.height() as GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, image.data().as_ptr() as *const GLvoid);
        }
        Ok(tex)
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn slot(&self) -> Option<GLuint> {
        self.slot
    }

    pub fn bind(&mut self, slot: u32) {
        self.slot = Some(slot as GLuint);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn unbind(&mut self) {
        self.slot = None;
    }

}

impl Drop for Texture {

    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id); }
    }

}