use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyInput}; // Add this import at the top
use ggez::mint;
use ggez::{event, Context, GameResult};

pub struct CameraSettings {
    pub fov_angle_deg: f32,
    pub camera_dist: f32,
}

impl CameraSettings {
    pub fn new(fov_angle_deg: u16, camera_dist: u16) -> Self {
        CameraSettings {
            fov_angle_deg: f32::from(fov_angle_deg),
            camera_dist: f32::from(camera_dist),
        }
    }
}

struct Point2D(f32, f32);

struct Point3D(f32, f32, f32);

struct Attitude {
    yaw: f32,
    pitch: f32,
    roll: f32,
}

struct Cube {
    vertices: [Point3D; 8],
}

impl Cube {
    fn default() -> Cube {
        let scale = 50.0; // Current scale factor
        Cube {
            vertices: [
                Point3D(-1.0 * scale, -1.0 * scale, -1.0 * scale), // Front bottom left
                Point3D(1.0 * scale, -1.0 * scale, -1.0 * scale),  // Front bottom right
                Point3D(1.0 * scale, 1.0 * scale, -1.0 * scale),   // Front top right
                Point3D(-1.0 * scale, 1.0 * scale, -1.0 * scale),  // Front top left
                Point3D(-1.0 * scale, -1.0 * scale, 1.0 * scale),  // Back bottom left
                Point3D(1.0 * scale, -1.0 * scale, 1.0 * scale),   // Back bottom right
                Point3D(1.0 * scale, 1.0 * scale, 1.0 * scale),    // Back top right
                Point3D(-1.0 * scale, 1.0 * scale, 1.0 * scale),   // Back top left
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
        |matrix: [[f32; 3]; 3], (px, py, pz): (f32, f32, f32)| -> (f32, f32, f32) {
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

struct CubeState {
    camera_settings: CameraSettings,
    cube: Cube,
    cursor: Point2D,
    screen_width: f32,
    screen_height: f32,
}

impl CubeState {
    fn new(camera_settings: CameraSettings, ctx: &Context) -> CubeState {
        let (width, height) = ctx.gfx.drawable_size();
        CubeState {
            camera_settings,
            cursor: Point2D(width / 2.0, height / 2.0),
            cube: Cube::default(),
            screen_width: width,
            screen_height: height,
        }
    }

    fn update_cursor(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => self.cursor.1 -= 10.0,
            KeyCode::Down => self.cursor.1 += 10.0,
            KeyCode::Left => self.cursor.0 += 10.0,
            KeyCode::Right => self.cursor.0 -= 10.0,
            _ => (),
        }

        // Clamp cursor to screen bounds
        self.cursor.0 = self.cursor.0.max(0.0).min(self.screen_width);
        self.cursor.1 = self.cursor.1.max(0.0).min(self.screen_height);
    }
}

impl event::EventHandler<ggez::GameError> for CubeState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(key) = input.keycode {
            self.update_cursor(key);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        let cursor_x_ratio = (self.cursor.0 / self.screen_width) * std::f32::consts::PI;
        let cursor_y_ratio = (self.cursor.1 / self.screen_height) * std::f32::consts::PI;
        let attitude = Attitude {
            yaw: 0.0,
            pitch: cursor_x_ratio,
            roll: cursor_y_ratio,
        };
        let projected_vertices: Vec<Point2D> = self
            .cube
            .vertices
            .iter()
            .map(|point_3d| get_rotated_point(point_3d, &attitude))
            .map(|point_3d| project_3d_to_2d(&point_3d, &self.camera_settings))
            .map(|point_2d| {
                Point2D(
                    point_2d.0 + self.screen_width / 2.0,
                    point_2d.1 + self.screen_height / 2.0,
                )
            })
            .collect();

        // Define the edges of the cube using vertex indices
        let edges = [
            // Front face
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            // Back face
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            // Connecting edges
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        // Draw the edges
        for (start, end) in edges.iter() {
            let line = graphics::Mesh::new_line(
                ctx,
                &[
                    mint::Point2 {
                        x: projected_vertices[*start].0,
                        y: projected_vertices[*start].1,
                    },
                    mint::Point2 {
                        x: projected_vertices[*end].0,
                        y: projected_vertices[*end].1,
                    },
                ],
                2.0, // line width
                graphics::Color::WHITE,
            )?;
            canvas.draw(&line, graphics::DrawParam::default());
        }

        // Draw each point as a small circle
        for point in &projected_vertices {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                mint::Point2 {
                    x: point.0,
                    y: point.1,
                },
                5.0, // radius of 5 pixels
                0.1, // tolerance
                graphics::Color::WHITE,
            )?;
            canvas.draw(&circle, graphics::DrawParam::default());
        }

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn run(camera_settings: CameraSettings) -> GameResult {
    let cb = ggez::ContextBuilder::new("cube", "ieg");
    let (ctx, event_loop) = cb.build()?;
    let state = CubeState::new(camera_settings, &ctx);

    event::run(ctx, event_loop, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a constant epsilon for floating-point comparisons
    const EPSILON: f32 = 1e-6;

    #[test]
    fn project_3d_to_2d_straight_ahead_projection() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(0.0, 0.0, 0.0);
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.0).abs() < EPSILON);
        assert!((projected.1).abs() < EPSILON);
    }

    #[test]
    fn project_3d_to_2d_offset_point_projection() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(5.0, 3.0, 0.0);
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.0 - 5.0).abs() < EPSILON);
        assert!((projected.1 - 3.0).abs() < EPSILON);
    }

    #[test]
    fn project_3d_to_2d_depth_scaling() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(5.0, 3.0, 10.0);
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.0 - 2.5).abs() < EPSILON);
        assert!((projected.1 - 1.5).abs() < EPSILON);
    }

    #[test]
    fn project_3d_to_2d_at_camera_position() {
        let camera = CameraSettings::new(90, 10);
        let point = Point3D(1.0, 1.0, -10.0);
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.0 - 1.0).abs() < EPSILON);
        assert!((projected.1 - 1.0).abs() < EPSILON);
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

        assert!((rotated.0 - 1.0).abs() < EPSILON);
        assert!((rotated.1 - 1.0).abs() < EPSILON);
        assert!((rotated.2 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_yaw_90_degrees() {
        let attitude = Attitude {
            yaw: std::f32::consts::PI / 2.0,
            pitch: 0.0,
            roll: 0.0,
        };
        let point = Point3D(1.0, 0.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.0 - 0.0).abs() < EPSILON);
        assert!((rotated.1 - 1.0).abs() < EPSILON);
        assert!((rotated.2 - 0.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_pitch_90_degrees() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: std::f32::consts::PI / 2.0,
            roll: 0.0,
        };
        let point = Point3D(1.0, 0.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.0 - 0.0).abs() < EPSILON);
        assert!((rotated.1 - 0.0).abs() < EPSILON);
        assert!((rotated.2 - -1.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_roll_90_degrees() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: 0.0,
            roll: std::f32::consts::PI / 2.0,
        };
        let point = Point3D(0.0, 1.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.0 - 0.0).abs() < EPSILON);
        assert!((rotated.1 - 0.0).abs() < EPSILON);
        assert!((rotated.2 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_combined_rotation() {
        let attitude = Attitude {
            yaw: std::f32::consts::PI / 2.0,
            pitch: std::f32::consts::PI / 2.0,
            roll: 0.0,
        };
        let point = Point3D(1.0, 0.0, 0.0);
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.0 - 0.0).abs() < EPSILON);
        assert!((rotated.1 - 0.0).abs() < EPSILON);
        assert!((rotated.2 - -1.0).abs() < EPSILON);
    }
}
