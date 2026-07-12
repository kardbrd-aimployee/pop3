use cgmath::{
    perspective, Angle, Deg, Matrix4, PerspectiveFov, Point2, Point3, Rad, SquareMatrix, Vector3,
};

pub struct Camera {
    pub angle_x: i16,
    pub angle_y: i16,
    pub angle_z: i16,
    pub pos: Vector3<f32>,
}

pub struct Screen {
    pub width: u32,
    pub height: u32,
}

pub struct MVP {
    pub transform: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub eye: Vector3<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            angle_x: 0,
            angle_y: 0,
            angle_z: 0,
            pos: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl MVP {
    pub fn new(screen: &Screen, camera: &Camera, focus: Vector3<f32>) -> MVP {
        Self::with_zoom(screen, camera, 1.0, focus, 0.0)
    }

    pub fn with_zoom(
        screen: &Screen,
        camera: &Camera,
        zoom: f32,
        focus: Vector3<f32>,
        min_z: f32,
    ) -> MVP {
        let az = Rad::from(Deg(camera.angle_z as f32));
        let ax = Rad::from(Deg(camera.angle_x as f32));
        let radius = 1.5 / zoom;

        // Orbit camera position around focus point, clamped above terrain
        let eye_pos = Point3::new(
            focus.x + radius * Rad::cos(ax) * Rad::sin(az),
            focus.y + radius * Rad::cos(ax) * Rad::cos(az),
            (focus.z - radius * Rad::sin(ax)).max(min_z),
        );
        let target = Point3::new(focus.x, focus.y, focus.z);

        // World geometry stays static — no rotation transform
        let transform = Matrix4::identity();

        // Z is up in world space
        let view = Matrix4::look_at_rh(eye_pos, target, Vector3::new(0.0, 0.0, 1.0));

        let projection = {
            let asp = screen.width as f32 / screen.height as f32;
            perspective(Rad(1.0), asp, 0.1, 10000.0)
        };

        let eye = Vector3::new(eye_pos.x, eye_pos.y, eye_pos.z);
        MVP {
            transform,
            view,
            projection,
            eye,
        }
    }

    fn make_fov(screen: &Screen) -> PerspectiveFov<f32> {
        let aspect = screen.width as f32 / screen.height as f32;
        PerspectiveFov {
            fovy: Rad(1.0),
            aspect,
            near: 1.0,
            far: 10000.0,
        }
    }
}

/*
 * opengl (x;y) + asp
 * (-1;1) (1;1)
 * (-1;-1) (1:-1)
 *
 * screen (w;h)
 * (0;0) (w;0)
 * (0;h) (w;h)
 */
pub fn screen_to_scene_zoom(
    screen: &Screen,
    camera: &Camera,
    pos_screen: &Point2<f32>,
    zoom: f32,
    focus: Vector3<f32>,
    min_z: f32,
) -> (Vector3<f32>, Vector3<f32>) {
    let mvp = MVP::with_zoom(screen, camera, zoom, focus, min_z);
    let asp = screen.width as f32 / screen.height as f32;
    let x: f32 = {
        let w = screen.width as f32;
        asp * 2.0 * (pos_screen.x as f32 - w / 2.0) / w
    };
    let y: f32 = {
        let h = screen.height as f32;
        2.0 * (h / 2.0 - pos_screen.y as f32) / h
    };

    let vec_screen_s: Vector3<f32> = mvp.eye;
    let vec_screen_e: Vector3<f32> = {
        let per_fov = MVP::make_fov(screen);
        let x_norm = x / asp;
        let y_norm = y;
        let z = 10000.0;
        let far_ymax = z * Rad::tan(Rad(1.0) / 2.0);
        let far_xmax = far_ymax * per_fov.aspect;
        let x_far = far_xmax * x_norm;
        let y_far = far_ymax * y_norm;
        Vector3 {
            x: x_far,
            y: y_far,
            z: -z,
        }
    };
    let v1 = {
        let mvp_m = mvp.transform;
        let mvp_t = mvp_m.invert().unwrap();
        (mvp_t * vec_screen_s.extend(1.0)).truncate()
    };
    let v2 = {
        let mvp_t1 = (mvp.view * mvp.transform).invert().unwrap();
        (mvp_t1 * vec_screen_e.extend(1.0)).truncate()
    };
    (v1, v2)
}
