use nalgebra;
pub struct Camera {
    target: nalgebra::Vector4<f32>,
    projection: nalgebra::Perspective3<f32>,
    rotation: nalgebra::UnitQuaternion<f32>,
}

impl Camera {
    pub fn new(
        aspect_ratio: f32,
        field_of_view: f32,
        near_z_plane: f32,
        far_z_plane: f32,
    ) -> Camera {
        Camera {
            target: nalgebra::Vector4::new(0.0, 0.0, 0.0, 0.0),
            projection: nalgebra::Perspective3::new(aspect_ratio, field_of_view, near_z_plane, far_z_plane),
            rotation: nalgebra::UnitQuaternion::from_axis_angle(&nalgebra::Vector3::x_axis(), std::f32::consts::PI / 4.0),
        }
    }

    pub fn direction(&self) -> nalgebra::Vector3<f32> {
        self.rotation * nalgebra::Vector3::new(0.0, 0.0, -1.0)
    }

    pub fn change_aspect_ratio(&mut self, new_aspect_ratio: f32) {
        self.projection.set_aspect(new_aspect_ratio);
    }
}