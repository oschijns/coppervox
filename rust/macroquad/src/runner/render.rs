use super::{Component, Runner};
use macroquad::prelude::QuadGl;

impl Component {
    /// Draw the component in the scene
    #[inline]
    pub fn draw(&self, gl: &mut QuadGl) {
        // Draw mode should be set to triangles beforehand
        // gl.draw_mode(DrawMode::Triangles);

        // Draw the mesh at the expected location
        let mesh = self.mesh.as_ref();
        gl.push_model_matrix(self.transform);
        gl.geometry(mesh.vertices.as_slice(), mesh.indices.as_slice());
        gl.pop_model_matrix();
    }
}
