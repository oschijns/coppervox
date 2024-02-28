use crate::Element;
use bevy::ecs::component::Component;

/// Specify how a value should be rendered on the material of a instance
pub trait RenderedValue {
    /// update the shader parameter of the material
    fn set_material_parameter(&self);
}

/// a render component to attach to logic elements
/// to update their look as data pass through them
#[derive(Component)]
pub struct Render<R: Element> {
    value: R,
}

impl<R: Element + Default> Default for Render<R> {
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl<R: Element> Render<R> {
    pub fn refresh<'a, V>(&mut self, value: &'a V)
    where
        R: From<&'a V>,
    {
        self.value = R::from(value);
    }
}
