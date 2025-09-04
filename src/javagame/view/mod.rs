use innovus::tools::*;

pub mod block_preview;
pub mod text;

pub struct Camera {
    view: Transform3D,
    projection: Transform3D,
    position: Vector<f32, 2>,
    target: Vector<f32, 2>,
    size: Vector<f32, 2>,
    zoom: f32,
    speed: f32,
}

impl Camera {
    pub fn new(position: Vector<f32, 2>, size: Vector<f32, 2>, zoom: f32, speed: f32) -> Self {
        Self {
            view: Transform3D::identity(),
            projection: Transform3D::identity(),
            position,
            target: position,
            size,
            zoom,
            speed,
        }
    }

    pub fn view(&self) -> Transform3D {
        self.view
    }

    pub fn projection(&self) -> Transform3D {
        self.projection
    }

    pub fn view_projection(&self) -> Transform3D {
        self.projection * self.view
    }

    pub fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector<f32, 2>) {
        self.position = position;
    }

    pub fn snap_to_target(&mut self) {
        self.position = self.target;
    }

    pub fn target(&self) -> Vector<f32, 2> {
        self.target
    }

    pub fn set_target(&mut self, target: Vector<f32, 2>) {
        self.target = target;
    }

    pub fn size(&self) -> Vector<f32, 2> {
        self.size
    }

    pub fn set_size(&mut self, size: Vector<f32, 2>) {
        self.size = size;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn get_world_pos(&self, screen_pos: Vector<f32, 2>) -> Vector<f32, 2> {
        (screen_pos - self.size * 0.5) * Vector([1.0, -1.0]) / self.zoom + self.position
    }

    pub fn update(&mut self, dt: f32) {
        self.view.reset_to_identity();
        self.view.look_at(
            Vector([self.position.x(), self.position.y(), 1.0]),
            Vector([self.position.x(), self.position.y(), 0.0]),
            Vector([0.0, 1.0, 0.0]),
        );

        self.projection.reset_to_identity();
        let scale = 0.5 / self.zoom;
        self.projection.orthographic(
            self.size.x() * -scale,
            self.size.x() * scale,
            self.size.y() * -scale,
            self.size.y() * scale,
            100.0,
            -100.0,
        );

        if self.position.equals_delta(&self.target, 1.0 / self.zoom) {
            self.snap_to_target();
        }
        else {
            self.position = self.position.lerp(&self.target, (self.speed * dt).min(1.0));
        }
    }
}
