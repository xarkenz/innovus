use std::error::Error;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use crate::gfx::Gfx;
use crate::gfx::buffer::DynamicArrayBuffer;
use crate::tools::{Transform3D, Vector};

pub trait Vertex : bytemuck::Zeroable + bytemuck::Pod {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        }
    }
}

#[macro_export]
macro_rules! impl_vertex {
    (for $t:ty, $($attr:ident: $fmt:ident @ $loc:expr),* $(,)?) => {
        impl $crate::gfx::mesh::Vertex for $t {
            const ATTRIBUTES: &'static [::wgpu::VertexAttribute] = &[$(
                ::wgpu::VertexAttribute {
                    offset: ::std::mem::offset_of!(Self, $attr) as ::wgpu::BufferAddress,
                    shader_location: $loc,
                    format: ::wgpu::VertexFormat::$fmt,
                },
            )*];
        }
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex3D {
    pub position: Vector<f32, 3>,
    pub color: Vector<f32, 4>,
    pub uv: Vector<f32, 2>,
    pub normal: Vector<f32, 3>,
}

impl Vertex3D {
    pub fn new(
        position: Vector<f32, 3>,
        color: Option<Vector<f32, 4>>,
        uv: Option<Vector<f32, 2>>,
        normal: Option<Vector<f32, 3>>,
    ) -> Self {
        Self {
            position,
            color: color.unwrap_or(Vector::one()),
            uv: uv.unwrap_or(Vector::splat(f32::NAN)),
            normal: normal.unwrap_or(Vector::zero()),
        }
    }

    pub fn colored(position: Vector<f32, 3>, color: Vector<f32, 4>) -> Self {
        Self {
            position,
            color,
            uv: Vector::splat(f32::NAN),
            normal: Vector::zero(),
        }
    }

    pub fn textured(position: Vector<f32, 3>, uv: Vector<f32, 2>) -> Self {
        Self {
            position,
            color: Vector::one(),
            uv,
            normal: Vector::zero(),
        }
    }

    pub fn combined(position: Vector<f32, 3>, color: Vector<f32, 4>, uv: Vector<f32, 2>) -> Self {
        Self {
            position,
            color,
            uv,
            normal: Vector::zero(),
        }
    }

    pub fn has_normal(&self) -> bool {
        self.normal != Vector::zero()
    }
}

impl_vertex! {
    for Vertex3D,
    position: Float32x3 @ 0,
    color: Float32x4 @ 1,
    uv: Float32x2 @ 2,
    normal: Float32x3 @ 3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex2D {
    pub position: Vector<f32, 3>,
    pub color: Vector<f32, 4>,
    pub uv: Vector<f32, 2>,
}

impl Vertex2D {
    pub fn new(position: Vector<f32, 3>, color: Option<Vector<f32, 4>>, uv: Option<Vector<f32, 2>>) -> Self {
        Self {
            position,
            color: color.unwrap_or(Vector::one()),
            uv: uv.unwrap_or(Vector::splat(f32::NAN)),
        }
    }
}

impl_vertex! {
    for Vertex2D,
    position: Float32x3 @ 0,
    color: Float32x4 @ 1,
    uv: Float32x2 @ 2_u32,
}

#[derive(Copy, Clone, Debug)]
pub struct MeshSlice {
    pub first_vertex: usize,
    pub vertex_count: usize,
    pub first_triangle: usize,
    pub triangle_count: usize,
}

#[derive(Clone, Debug)]
pub struct Mesh<V: Vertex> {
    vertices: Vec<V>,
    triangles: Vec<[u32; 3]>,
}

impl<V: Vertex> Mesh<V> {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    pub fn with_data(vertices: Vec<V>, triangles: Vec<[u32; 3]>) -> Self {
        Self {
            vertices,
            triangles,
        }
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

    pub fn vertex_at(&self, index: usize) -> &V {
        &self.vertices[index]
    }

    pub fn vertex_at_mut(&mut self, index: usize) -> &mut V {
        &mut self.vertices[index]
    }

    pub fn slice_vertices(&self, slice: &MeshSlice) -> &[V] {
        &self.vertices[slice.first_vertex .. slice.first_vertex + slice.vertex_count]
    }

    pub fn slice_vertices_mut(&mut self, slice: &MeshSlice) -> &mut [V] {
        &mut self.vertices[slice.first_vertex .. slice.first_vertex + slice.vertex_count]
    }

    pub fn slice_triangles(&self, slice: &MeshSlice) -> &[[u32; 3]] {
        &self.triangles[slice.first_triangle .. slice.first_triangle + slice.triangle_count]
    }

    pub fn slice_triangles_mut(&mut self, slice: &MeshSlice) -> &mut [[u32; 3]] {
        &mut self.triangles[slice.first_triangle .. slice.first_triangle + slice.triangle_count]
    }

    pub fn as_slice(&self) -> MeshSlice {
        MeshSlice {
            first_vertex: 0,
            vertex_count: self.vertices.len(),
            first_triangle: 0,
            triangle_count: self.triangles.len(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.triangles.clear();
    }

    pub fn add(&mut self, vertices: &[V], triangles: &[[u32; 3]]) -> MeshSlice {
        let first_vertex = self.vertices.len();
        let vertex_count = vertices.len();
        let first_face = self.triangles.len();
        let face_count = triangles.len();

        for &face in triangles {
            self.triangles.push(face.map(|index| first_vertex as u32 + index))
        }
        self.vertices.extend_from_slice(vertices);

        MeshSlice {
            first_vertex,
            vertex_count,
            first_triangle: first_face,
            triangle_count: face_count,
        }
    }

    pub fn add_mesh(&mut self, mesh: &Self) -> MeshSlice {
        self.add(mesh.vertices(), mesh.triangles())
    }
}

impl<V: Vertex> Default for Mesh<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh<Vertex3D> {
    pub fn icosahedron(center: Vector<f32, 3>, radius: f32, color: Vector<f32, 4>) -> Self {
        const MINOR: f32 = 0.525731112119133606;
        const MAJOR: f32 = 0.850650808352039932;
        let minor = MINOR * radius;
        let major = MAJOR * radius;
        Self::with_data(
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

    pub fn icosphere(center: Vector<f32, 3>, radius: f32, color: Vector<f32, 4>, subdivisions: u32) -> Self {
        let mut mesh = Self::icosahedron(center, radius, color);

        for _ in 0..subdivisions {
            let mut add_vertices: Vec<Vertex3D> = Vec::new();
            let mut new_triangles: Vec<[u32; 3]> = Vec::new();
            let mut edge_midpoint_map: Vec<((u32, u32), u32)> = Vec::new();
            for triangle in mesh.triangles {
                let mut fetch_midpoint = |mut v1, mut v2| {
                    if v1 > v2 {
                        std::mem::swap(&mut v1, &mut v2);
                    }
                    for (edge, midpoint) in edge_midpoint_map.iter() {
                        if edge.0 == v1 && edge.1 == v2 {
                            return *midpoint;
                        }
                    }
                    let idx = (mesh.vertices.len() + add_vertices.len()) as u32;
                    let normal = (mesh.vertices[v1 as usize].normal + mesh.vertices[v2 as usize].normal).normalized();
                    add_vertices.push(Vertex3D::new(
                        normal.mul(radius) + center,
                        Some(color),
                        None,
                        Some(normal),
                    ));
                    edge_midpoint_map.push(((v1, v2), idx));
                    idx
                };
                let midpoints: [u32; 3] = [
                    fetch_midpoint(triangle[0], triangle[1]),
                    fetch_midpoint(triangle[1], triangle[2]),
                    fetch_midpoint(triangle[2], triangle[0]),
                ];
                new_triangles.push([triangle[0], midpoints[0], midpoints[2]]);
                new_triangles.push([triangle[1], midpoints[1], midpoints[0]]);
                new_triangles.push([triangle[2], midpoints[2], midpoints[1]]);
                new_triangles.push(midpoints);
            }
            mesh.vertices.append(&mut add_vertices);
            mesh.triangles = new_triangles;
        }

        mesh
    }

    pub fn transform(&mut self, slice: &MeshSlice, matrix: Transform3D<f32>) {
        for index in 0..slice.vertex_count {
            let index = slice.first_vertex + index;
            let vertex = self.vertex_at_mut(index);
            vertex.position = matrix * vertex.position;
            if vertex.has_normal() {
                vertex.normal = matrix.affine() * vertex.normal;
            }
        }
    }

    pub fn rotate_x(&mut self, slice: &MeshSlice, angle: f32, axis_y: f32, axis_z: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(Vector([0.0, axis_y, axis_z]));
        rotation.rotate_x(angle);
        rotation.translate(Vector([0.0, -axis_y, -axis_z]));
        self.transform(slice, rotation);
    }

    pub fn rotate_y(&mut self, slice: &MeshSlice, angle: f32, axis_x: f32, axis_z: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(Vector([axis_x, 0.0, axis_z]));
        rotation.rotate_y(angle);
        rotation.translate(Vector([-axis_x, 0.0, -axis_z]));
        self.transform(slice, rotation);
    }

    pub fn rotate_z(&mut self, slice: &MeshSlice, angle: f32, axis_x: f32, axis_y: f32) {
        let mut rotation = Transform3D::identity();
        rotation.translate(Vector([axis_x, axis_y, 0.0]));
        rotation.rotate_z(angle);
        rotation.translate(Vector([-axis_x, -axis_y, 0.0]));
        self.transform(slice, rotation);
    }
}

#[derive(Clone, Debug)]
pub struct ParseMeshError {
    message: String,
}

impl Error for ParseMeshError {}

impl std::fmt::Display for ParseMeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<ParseFloatError> for ParseMeshError {
    fn from(error: ParseFloatError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl From<ParseIntError> for ParseMeshError {
    fn from(error: ParseIntError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl FromStr for Mesh<Vertex3D> {
    type Err = ParseMeshError;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        fn parse_f32(s: &str) -> Result<f32, ParseMeshError> {
            s.parse::<f32>().map_err(ParseMeshError::from)
        }
        fn parse_clamp_f32(s: &str) -> Result<f32, ParseMeshError> {
            match parse_f32(s)? {
                num if 0.0 <= num && num <= 1.0 => Ok(num),
                _ => Err(ParseMeshError {
                    message: format!("'{}' not in range 0-1", s),
                }),
            }
        }
        fn parse_usize(s: &str) -> Result<usize, ParseMeshError> {
            s.parse::<usize>().map_err(ParseMeshError::from)
        }
        fn parse_face_element(s: &str) -> Result<[usize; 4], ParseMeshError> {
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
                            return Err(ParseMeshError {
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
                Err(ParseMeshError {
                    message: "face element index cannot be 0".into(),
                })
            }
            else {
                Ok(indices)
            }
        }
        fn missing_error<T>() -> Result<T, ParseMeshError> {
            Err(ParseMeshError { message: "missing command argument".into() })
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
                    return Err(ParseMeshError {
                        message: format!("invalid element position index: {}", element[0]),
                    });
                }
                if element[1] > textures.len() {
                    return Err(ParseMeshError {
                        message: format!("invalid element UV coordinate index: {}", element[1]),
                    });
                }
                if element[2] > normals.len() {
                    return Err(ParseMeshError {
                        message: format!("invalid element normal index: {}", element[2]),
                    });
                }
                if element[3] > colors.len() {
                    return Err(ParseMeshError {
                        message: format!("invalid element color index: {}", element[3]),
                    });
                }
                vertices.push(Vertex3D::new(
                    positions[element[0] - 1],
                    match element[3] {
                        0 => None,
                        color_index => colors.get(color_index - 1).copied()
                    },
                    match element[1] {
                        0 => None,
                        texture_index => textures.get(texture_index - 1).copied()
                    },
                    match element[2] {
                        0 => None,
                        normal_index => normals.get(normal_index - 1).copied()
                    },
                ));
            }
        }

        Ok(Self::with_data(vertices, faces))
    }
}

#[derive(Debug)]
pub struct MeshRenderer<V: Vertex> {
    vertex_buffer: DynamicArrayBuffer<V>,
    triangle_buffer: DynamicArrayBuffer<[u32; 3]>,
    mesh: Mesh<V>,
}

impl<V: Vertex> MeshRenderer<V> {
    pub fn create(gfx: &Gfx) -> Self {
        Self::create_from_mesh(gfx, Mesh::new())
    }

    pub fn create_from_mesh(gfx: &Gfx, mesh: Mesh<V>) -> Self {
        Self {
            vertex_buffer: DynamicArrayBuffer::create(
                gfx,
                Some("MeshRenderer Vertex Buffer"),
                wgpu::BufferUsages::VERTEX,
                mesh.vertices(),
            ),
            triangle_buffer: DynamicArrayBuffer::create(
                gfx,
                Some("MeshRenderer Triangle Buffer"),
                wgpu::BufferUsages::INDEX,
                mesh.triangles(),
            ),
            mesh,
        }
    }

    pub fn mesh(&self) -> &Mesh<V> {
        &self.mesh
    }

    pub fn mesh_mut(&mut self) -> &mut Mesh<V> {
        &mut self.mesh
    }

    pub fn is_empty(&self) -> bool {
        self.mesh.is_empty()
    }

    pub fn vertices(&self) -> &[V] {
        self.mesh.vertices()
    }

    pub fn vertices_mut(&mut self) -> &mut [V] {
        self.mesh.vertices_mut()
    }

    pub fn triangles(&self) -> &[[u32; 3]] {
        self.mesh.triangles()
    }

    pub fn triangles_mut(&mut self) -> &mut [[u32; 3]] {
        self.mesh.triangles_mut()
    }

    pub fn vertex_at(&self, index: usize) -> &V {
        self.mesh.vertex_at(index)
    }

    pub fn vertex_at_mut(&mut self, index: usize) -> &mut V {
        self.mesh.vertex_at_mut(index)
    }

    pub fn as_slice(&self) -> MeshSlice {
        self.mesh.as_slice()
    }

    pub fn clear(&mut self) {
        self.vertex_buffer.clear();
        self.triangle_buffer.clear();
        self.mesh.clear();
    }

    pub fn add(&mut self, vertices: &[V], triangles: &[[u32; 3]]) -> MeshSlice {
        let slice = self.mesh.add(vertices, triangles);
        self.upload_buffer_slice(&slice);
        slice
    }

    pub fn add_mesh(&mut self, mesh: &Mesh<V>) -> MeshSlice {
        self.add(mesh.vertices(), mesh.triangles())
    }

    pub fn upload_vertex_buffer(&mut self) {
        self.vertex_buffer.rewrite(self.mesh.vertices());
    }

    pub fn upload_triangle_buffer(&mut self) {
        self.triangle_buffer.rewrite(self.mesh.triangles());
    }

    pub fn upload_buffers(&mut self) {
        self.upload_vertex_buffer();
        self.upload_triangle_buffer();
    }

    pub fn upload_buffer_slice(&mut self, slice: &MeshSlice) {
        // Ensure the vertex buffer is the correct size
        self.vertex_buffer.resize(self.mesh.vertices().len() as wgpu::BufferAddress);
        // Write the sliced data to the vertex buffer
        self.vertex_buffer.write(slice.first_vertex as wgpu::BufferAddress, self.mesh.slice_vertices(slice));

        // Ensure the triangle buffer is the correct size
        self.triangle_buffer.resize(self.mesh.triangles().len() as wgpu::BufferAddress);
        // Write the sliced data to the triangle buffer
        self.triangle_buffer.write(slice.first_triangle as wgpu::BufferAddress, self.mesh.slice_triangles(slice));
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if !self.triangles().is_empty() {
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.triangle_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..(self.triangles().len() as u32 * 3), 0, 0..1);
        }
    }
}
