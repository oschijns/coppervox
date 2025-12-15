use super::Component;
use macroquad::prelude::{GlPipeline, QuadGl};

impl Component {
    /// Draw the component in the scene
    pub fn draw(&self, gl: &mut QuadGl, pipeline: GlPipeline) {
        // Draw mode should be set to triangles beforehand
        // gl.draw_mode(DrawMode::Triangles);

        // Draw the mesh at the expected location
        let mesh = self.mesh.as_ref();
        let color = self.colorize.get_color(self.current_value);
        gl.push_model_matrix(self.transform);
        gl.set_uniform(pipeline, "u_color", color);
        gl.geometry(mesh.vertices.as_slice(), mesh.indices.as_slice());
        gl.pop_model_matrix();
    }
}
