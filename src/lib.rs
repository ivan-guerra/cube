pub struct CameraSettings {
    pub fov_angle_deg: f64,
    pub camera_dist: f64,
    near_plane_dist: f64,
    far_plane_dist: f64,
}

impl CameraSettings {
    pub fn new(fov_angle_deg: u16, camera_dist: u16) -> Self {
        CameraSettings {
            fov_angle_deg: f64::from(fov_angle_deg),
            camera_dist: f64::from(camera_dist),
            near_plane_dist: 0.0,
            far_plane_dist: 0.0,
        }
    }
}

struct Point2D(f64, f64);

struct Point3D(f64, f64, f64);

struct Attitude {
    yaw: f64,
    pitch: f64,
    roll: f64,
}

struct Cube {
    vertices: [Point2D; 8],
}

impl Cube {
    fn new() -> Cube {
        Cube {
            vertices: [
                Point2D(0.0, 0.0),
                Point2D(1.0, 0.0),
                Point2D(1.0, 1.0),
                Point2D(0.0, 1.0),
                Point2D(0.0, 0.0),
                Point2D(1.0, 0.0),
                Point2D(1.0, 1.0),
                Point2D(0.0, 1.0),
            ],
        }
    }
}

fn get_rotated_point(point: &Point3D, attitude: &Attitude) -> Point3D {
    let yaw_matrix = [
        [attitude.yaw.cos(), -attitude.yaw.sin(), 0.0],
        [attitude.yaw.sin(), attitude.yaw.cos(), 0.0],
        [0.0, 0.0, 1.0],
    ];

    let pitch_matrix = [
        [attitude.pitch.cos(), 0.0, attitude.pitch.sin()],
        [0.0, 1.0, 0.0],
        [-attitude.pitch.sin(), 0.0, attitude.pitch.cos()],
    ];

    let roll_matrix = [
        [1.0, 0.0, 0.0],
        [0.0, attitude.roll.cos(), -attitude.roll.sin()],
        [0.0, attitude.roll.sin(), attitude.roll.cos()],
    ];

    // Helper function for matrix multiplication with a point
    let multiply_matrix_point =
        |matrix: [[f64; 3]; 3], (px, py, pz): (f64, f64, f64)| -> (f64, f64, f64) {
            (
                px * matrix[0][0] + py * matrix[0][1] + pz * matrix[0][2],
                px * matrix[1][0] + py * matrix[1][1] + pz * matrix[1][2],
                px * matrix[2][0] + py * matrix[2][1] + pz * matrix[2][2],
            )
        };

    // Chain the transformations
    let (x1, y1, z1) = multiply_matrix_point(roll_matrix, (point.0, point.1, point.2));
    let (x2, y2, z2) = multiply_matrix_point(pitch_matrix, (x1, y1, z1));
    let (x3, y3, z3) = multiply_matrix_point(yaw_matrix, (x2, y2, z2));

    Point3D(x3, y3, z3)
}

fn project_3d_to_2d(point: &Point3D, camera_settings: &CameraSettings) -> Point2D {
    let fov_angle_rad = camera_settings.fov_angle_deg.to_radians();
    let half_fov = fov_angle_rad / 2.0;
    let half_fov_tan = half_fov.tan();

    let x = point.0;
    let y = point.1;
    let z = point.2;

    // When z is 0, the point is at the camera distance
    let depth = z + camera_settings.camera_dist;

    // Avoid division by zero and handle points at camera distance
    let scale = if depth != 0.0 {
        camera_settings.camera_dist / depth
    } else {
        1.0
    };

    let x_proj = x * scale / half_fov_tan;
    let y_proj = y * scale / half_fov_tan;

    Point2D(x_proj, y_proj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_3d_to_2d_straight_ahead_projection() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(0.0, 0.0, 0.0);
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.0 - 0.0).abs() < 1e-10);
        assert!((projected.1 - 0.0).abs() < 1e-10);
    }

    #[test]
    fn project_3d_to_2d_offset_point_projection() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(5.0, 3.0, 0.0);
        let projected = project_3d_to_2d(&point, &camera);

        // For 90-degree FOV, tan(45°) = 1
        // So the projection should be scaled by 1
        assert!((projected.0 - 5.0).abs() < 1e-10);
        assert!((projected.1 - 3.0).abs() < 1e-10);
    }

    #[test]
    fn project_3d_to_2d_depth_scaling() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(5.0, 3.0, 10.0); // Point is 10 units further away
        let projected = project_3d_to_2d(&point, &camera);

        // Point should appear half size due to being 20 units away (10 camera + 10 z)
        assert!((projected.0 - 2.5).abs() < 1e-10);
        assert!((projected.1 - 1.5).abs() < 1e-10);
    }

    #[test]
    fn project_3d_to_2d_at_camera_position() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(1.0, 1.0, -10.0); // Point at camera position
        let projected = project_3d_to_2d(&point, &camera);

        // Point should project with scale = 1.0
        assert!((projected.0 - 1.0).abs() < 1e-10);
        assert!((projected.1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn get_rotated_point_no_rotation() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        };
        let point = Point3D(1.0, 1.0, 1.0);
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.0 - 1.0).abs() < 1e-10);
        assert!((rotated.1 - 1.0).abs() < 1e-10);
        assert!((rotated.2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn get_rotated_point_yaw_90_degrees() {
        let attitude = Attitude {
            yaw: std::f64::consts::PI / 2.0, // 90 degrees
            pitch: 0.0,
            roll: 0.0,
        };
        let point = Point3D(1.0, 0.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.0 - 0.0).abs() < 1e-10);
        assert!((rotated.1 - 1.0).abs() < 1e-10);
        assert!((rotated.2 - 0.0).abs() < 1e-10);
    }

    #[test]
    fn get_rotated_point_pitch_90_degrees() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: std::f64::consts::PI / 2.0, // 90 degrees
            roll: 0.0,
        };
        let point = Point3D(1.0, 0.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        // When pitching up 90 degrees, a point at (1,0,0) should rotate to (0,0,-1)
        assert!((rotated.0 - 0.0).abs() < 1e-10);
        assert!((rotated.1 - 0.0).abs() < 1e-10);
        assert!((rotated.2 - -1.0).abs() < 1e-10);
    }

    #[test]
    fn get_rotated_point_roll_90_degrees() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: 0.0,
            roll: std::f64::consts::PI / 2.0, // 90 degrees
        };
        let point = Point3D(0.0, 1.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.0 - 0.0).abs() < 1e-10);
        assert!((rotated.1 - 0.0).abs() < 1e-10);
        assert!((rotated.2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn get_rotated_point_combined_rotation() {
        let attitude = Attitude {
            yaw: std::f64::consts::PI / 2.0,   // 90 degrees
            pitch: std::f64::consts::PI / 2.0, // 90 degrees
            roll: 0.0,
        };
        let point = Point3D(1.0, 0.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        // When applying pitch up 90° and then yaw 90°, point (1,0,0) should rotate to (0,0,-1)
        assert!((rotated.0 - 0.0).abs() < 1e-10);
        assert!((rotated.1 - 0.0).abs() < 1e-10);
        assert!((rotated.2 - -1.0).abs() < 1e-10);
    }
}
