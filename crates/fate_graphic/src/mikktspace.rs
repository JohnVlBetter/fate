use cgmath::vec4;

use crate::mesh::Vertex;

const VERTEX_PER_FACE: usize = 3;

type Face = [u32; 3];

struct Mesh<'a> {
    faces: Vec<Face>,
    vertices: &'a mut [Vertex],
}

impl<'a> Mesh<'a> {
    fn get_vertex(&self, face: usize, vert: usize) -> Vertex {
        let face = self.faces[face];
        self.vertices[face[vert] as usize]
    }

    fn get_vertex_mut(&mut self, face: usize, vert: usize) -> &mut Vertex {
        let face = self.faces[face];
        &mut self.vertices[face[vert] as usize]
    }
}

impl<'a> mikktspace::Geometry for Mesh<'a> {
    fn num_faces(&self) -> usize {
        self.faces.len()
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        VERTEX_PER_FACE
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let pos = self.get_vertex(face, vert).pos;
        [pos[0], pos[1], pos[2]]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let normal = self.get_vertex(face, vert).normal;
        [normal[0], normal[1], normal[2]]
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let tex_coord = self.get_vertex(face, vert).tex_coord;
        [tex_coord[0], tex_coord[1]]
    }

    fn set_tangent(
        &mut self,
        tangent: [f32; 3],
        _bi_tangent: [f32; 3],
        _f_mag_s: f32,
        _f_mag_t: f32,
        bi_tangent_preserves_orientation: bool,
        face: usize,
        vert: usize,
    ) {
        let sign = if bi_tangent_preserves_orientation {
            -1.0
        } else {
            1.0
        };
        let vertex = self.get_vertex_mut(face, vert);
        vertex.tangent = vec4(tangent[0], tangent[1], tangent[2], sign);
    }
}

pub fn generate_tangents(indices: Option<&[u32]>, vertices: &mut [Vertex]) {
    log::info!("生成切线");

    let index_count = indices.map_or(0, |indices| indices.len());
    if !can_generate_inputs(index_count, vertices.len()) {
        log::warn!("无法生成切线");
        return;
    }

    let faces = if let Some(indices) = indices {
        (0..index_count)
            .step_by(VERTEX_PER_FACE)
            .map(|i| [indices[i], indices[i + 1], indices[i + 2]])
            .collect::<Vec<_>>()
    } else {
        let vertex_count = vertices.len() as u32;
        (0..vertex_count)
            .step_by(VERTEX_PER_FACE)
            .map(|i| [i, i + 1, i + 2])
            .collect::<Vec<_>>()
    };

    let mut mesh = Mesh { faces, vertices };

    mikktspace::generate_tangents(&mut mesh);
}

fn can_generate_inputs(index_count: usize, vertex_count: usize) -> bool {
    if vertex_count == 0 {
        log::warn!("生成切线最少需要一个顶点");
        return false;
    }

    if index_count > 0 && index_count % VERTEX_PER_FACE != 0 {
        log::warn!("顶点索引不是三的倍数");
        return false;
    }

    if index_count == 0 && vertex_count % VERTEX_PER_FACE != 0 {
        log::warn!(
            "顶点数不是三的倍数"
        );
        return false;
    }

    true
}
