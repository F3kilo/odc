#[derive(Default)]
pub struct Camera {
    view: ViewInfo,
    proj: ProjInfo,
}

impl Camera {
    fn new(view: ViewInfo, proj: ProjInfo) -> Self {
        Self { view, proj }
    }

    pub fn view_transform(&self) -> Transform {
        self.view.transform().to_cols_array_2d()
    }

    pub fn proj_transform(&self) -> Transform {
        self.proj.transform().to_cols_array_2d()
    }

    pub fn view_proj_transform(&self) -> Transform {
        (self.view.transform() * self.proj.transform()).to_cols_array_2d()
    }

    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.view.set_position(position.into());
        self
    }

    pub fn set_target(&mut self, target: Vec3) -> &mut Self {
        self.view.set_target(target.into());
        self
    }

    pub fn set_up(&mut self, up: Vec3) -> &mut Self {
        self.view.set_up(up.into());
        self
    }
}

pub type Vec3 = [f32; 3];
pub type Transform = [[f32; 4]; 4];

#[derive(Default)]
pub struct CameraBuilder {
    view: Option<ViewInfo>,
    proj: Option<ProjInfo>,
}

impl CameraBuilder {
    pub fn look_at(mut self, position: Vec3, target: Vec3, up: Vec3) -> Self {
        self.view = Some(ViewInfo::new(position.into(), target.into(), up.into()));
        self
    }

    pub fn perspective(mut self, fov_y: f32, aspect: f32, near: f32, far: Option<f32>) -> Self {
        self.proj = Some(ProjInfo::new_perspective(fov_y, aspect, near, far));
        self
    }

    pub fn orthographic(
        mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        self.proj = Some(ProjInfo::new_orthographic(
            left, right, bottom, top, near, far,
        ));
        self
    }

    pub fn build(self) -> Camera {
        let view = self.view.unwrap_or_default();
        let proj = self.proj.unwrap_or_default();
        Camera::new(view, proj)
    }
}

struct ViewInfo {
    position: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
}

impl Default for ViewInfo {
    fn default() -> Self {
        let pos = glam::Vec3::new(0.0, 0.0, 0.0);
        let target = glam::Vec3::new(0.0, 0.0, 1.0);
        let up = glam::Vec3::new(0.0, 1.0, 0.0);
        Self::new(pos, target, up)
    }
}

impl ViewInfo {
    pub fn new(position: glam::Vec3, mut target: glam::Vec3, up: glam::Vec3) -> Self {
        if position == target {
            // watch to z axis by default
            target = position + glam::Vec3::new(0.0, 0.0, 1.0);
        }

        Self {
            position,
            target,
            up,
        }
    }

    pub fn transform(&self) -> glam::Mat4 {
        glam::Mat4::look_at_lh(self.position, self.target, self.up)
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = position
    }

    pub fn set_target(&mut self, target: glam::Vec3) {
        self.target = target
    }

    pub fn set_up(&mut self, up: glam::Vec3) {
        self.up = up
    }
}

enum ProjInfo {
    Perspective(Perspective),
    Orthographic(Orthographic),
}

impl ProjInfo {
    pub fn new_perspective(fov_y: f32, aspect: f32, near: f32, far: Option<f32>) -> Self {
        Self::Perspective(Perspective {
            fov_y,
            aspect,
            near,
            far,
        })
    }

    pub fn new_orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self::Orthographic(Orthographic {
            left,
            right,
            bottom,
            top,
            near,
            far,
        })
    }

    pub fn transform(&self) -> glam::Mat4 {
        match self {
            ProjInfo::Perspective(p) => {
                if let Some(far) = p.far {
                    glam::Mat4::perspective_lh(p.fov_y, p.aspect, p.near, far)
                } else {
                    glam::Mat4::perspective_infinite_lh(p.fov_y, p.aspect, p.near)
                }
            }
            ProjInfo::Orthographic(o) => {
                glam::Mat4::orthographic_lh(o.left, o.right, o.bottom, o.top, o.near, o.far)
            }
        }
    }
}

impl Default for ProjInfo {
    fn default() -> Self {
        Self::Orthographic(Orthographic {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1.0,
        })
    }
}

struct Perspective {
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: Option<f32>,
}

struct Orthographic {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}
