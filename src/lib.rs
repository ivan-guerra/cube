use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::{event, graphics, mint, Context, GameResult};

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

struct Attitude {
    yaw: f32,
    pitch: f32,
    roll: f32,
}

struct Cube {
    vertices: [mint::Point3<f32>; 8],
}

impl Cube {
    fn default() -> Cube {
        Cube {
            vertices: [
                mint::Point3 {
                    x: -1.0,
                    y: -1.0,
                    z: -1.0,
                }, // Front bottom left
                mint::Point3 {
                    x: 1.0,
                    y: -1.0,
                    z: -1.0,
                }, // Front bottom right
                mint::Point3 {
                    x: 1.0,
                    y: 1.0,
                    z: -1.0,
                }, // Front top right
                mint::Point3 {
                    x: -1.0,
                    y: 1.0,
                    z: -1.0,
                }, // Front top left
                mint::Point3 {
                    x: -1.0,
                    y: -1.0,
                    z: 1.0,
                }, // Back bottom left
                mint::Point3 {
                    x: 1.0,
                    y: -1.0,
                    z: 1.0,
                }, // Back bottom right
                mint::Point3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }, // Back top right
                mint::Point3 {
                    x: -1.0,
                    y: 1.0,
                    z: 1.0,
                }, // Back top left
            ],
        }
    }
}

fn get_rotated_point(point: &mint::Point3<f32>, attitude: &Attitude) -> mint::Point3<f32> {
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

    let multiply_matrix_point =
        |matrix: [[f32; 3]; 3], point: (f32, f32, f32)| -> (f32, f32, f32) {
            (
                point.0 * matrix[0][0] + point.1 * matrix[0][1] + point.2 * matrix[0][2],
                point.0 * matrix[1][0] + point.1 * matrix[1][1] + point.2 * matrix[1][2],
                point.0 * matrix[2][0] + point.1 * matrix[2][1] + point.2 * matrix[2][2],
            )
        };

    let (x1, y1, z1) = multiply_matrix_point(roll_matrix, (point.x, point.y, point.z));
    let (x2, y2, z2) = multiply_matrix_point(pitch_matrix, (x1, y1, z1));
    let (x3, y3, z3) = multiply_matrix_point(yaw_matrix, (x2, y2, z2));

    mint::Point3 {
        x: x3,
        y: y3,
        z: z3,
    }
}

fn project_3d_to_2d(
    point: &mint::Point3<f32>,
    camera_settings: &CameraSettings,
) -> mint::Point2<f32> {
    let fov_angle_rad = camera_settings.fov_angle_deg.to_radians();
    let half_fov = fov_angle_rad / 2.0;
    let half_fov_tan = half_fov.tan();

    let depth = point.z + camera_settings.camera_dist;

    let scale = if depth != 0.0 {
        camera_settings.camera_dist / depth
    } else {
        1.0
    };

    let x_proj = point.x * scale / half_fov_tan;
    let y_proj = point.y * scale / half_fov_tan;

    mint::Point2 {
        x: x_proj,
        y: y_proj,
    }
}

struct CubeState {
    camera_settings: CameraSettings,
    cube: Cube,
    cursor: mint::Point2<f32>,
    screen_width: f32,
    screen_height: f32,
}

impl CubeState {
    fn new(camera_settings: CameraSettings, ctx: &Context) -> CubeState {
        let (width, height) = ctx.gfx.drawable_size();
        CubeState {
            camera_settings,
            cursor: mint::Point2 {
                x: width / 2.0,
                y: height / 2.0,
            },
            cube: Cube::default(),
            screen_width: width,
            screen_height: height,
        }
    }

    fn update_cursor(&mut self, key: KeyCode) {
        let (dx, dy) = match key {
            KeyCode::Up => (0.0, -10.0),
            KeyCode::Down => (0.0, 10.0),
            KeyCode::Left => (10.0, 0.0),
            KeyCode::Right => (-10.0, 0.0),
            _ => return,
        };

        self.cursor.x = (self.cursor.x + dx + self.screen_width) % self.screen_width;
        self.cursor.y = (self.cursor.y + dy + self.screen_height) % self.screen_height;
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

        let cursor_x_ratio = (self.cursor.x / self.screen_width) * std::f32::consts::PI;
        let cursor_y_ratio = (self.cursor.y / self.screen_height) * std::f32::consts::PI;
        let attitude = Attitude {
            yaw: 0.0,
            pitch: cursor_x_ratio,
            roll: cursor_y_ratio,
        };
        let projected_vertices: Vec<mint::Point2<f32>> = self
            .cube
            .vertices
            .iter()
            .map(|point_3d| get_rotated_point(point_3d, &attitude))
            .map(|point_3d| project_3d_to_2d(&point_3d, &self.camera_settings))
            .map(|point_2d| mint::Point2 {
                x: point_2d.x + self.screen_width / 2.0,
                y: point_2d.y + self.screen_height / 2.0,
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
                        x: projected_vertices[*start].x,
                        y: projected_vertices[*start].y,
                    },
                    mint::Point2 {
                        x: projected_vertices[*end].x,
                        y: projected_vertices[*end].y,
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
                    x: point.x,
                    y: point.y,
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

    const EPSILON: f32 = 1e-6;

    #[test]
    fn project_3d_to_2d_straight_ahead_projection() {
        let camera = CameraSettings::new(90, 10);
        let point = mint::Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.x).abs() < EPSILON);
        assert!((projected.y).abs() < EPSILON);
    }

    #[test]
    fn project_3d_to_2d_offset_point_projection() {
        let camera = CameraSettings::new(90, 10);
        let point = mint::Point3 {
            x: 5.0,
            y: 3.0,
            z: 0.0,
        };
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.x - 5.0).abs() < EPSILON);
        assert!((projected.y - 3.0).abs() < EPSILON);
    }

    #[test]
    fn project_3d_to_2d_depth_scaling() {
        let camera = CameraSettings::new(90, 10);
        let point = mint::Point3 {
            x: 5.0,
            y: 3.0,
            z: 10.0,
        };
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.x - 2.5).abs() < EPSILON);
        assert!((projected.y - 1.5).abs() < EPSILON);
    }

    #[test]
    fn project_3d_to_2d_at_camera_position() {
        let camera = CameraSettings::new(90, 10);
        let point = mint::Point3 {
            x: 1.0,
            y: 1.0,
            z: -10.0,
        };
        let projected = project_3d_to_2d(&point, &camera);

        assert!((projected.x - 1.0).abs() < EPSILON);
        assert!((projected.y - 1.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_no_rotation() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        };
        let point = mint::Point3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.x - 1.0).abs() < EPSILON);
        assert!((rotated.y - 1.0).abs() < EPSILON);
        assert!((rotated.z - 1.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_yaw_90_degrees() {
        let attitude = Attitude {
            yaw: std::f32::consts::PI / 2.0,
            pitch: 0.0,
            roll: 0.0,
        };
        let point = mint::Point3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.x - 0.0).abs() < EPSILON);
        assert!((rotated.y - 1.0).abs() < EPSILON);
        assert!((rotated.z - 0.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_pitch_90_degrees() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: std::f32::consts::PI / 2.0,
            roll: 0.0,
        };
        let point = mint::Point3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.x - 0.0).abs() < EPSILON);
        assert!((rotated.y - 0.0).abs() < EPSILON);
        assert!((rotated.z - -1.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_roll_90_degrees() {
        let attitude = Attitude {
            yaw: 0.0,
            pitch: 0.0,
            roll: std::f32::consts::PI / 2.0,
        };
        let point = mint::Point3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.x - 0.0).abs() < EPSILON);
        assert!((rotated.y - 0.0).abs() < EPSILON);
        assert!((rotated.z - 1.0).abs() < EPSILON);
    }

    #[test]
    fn get_rotated_point_combined_rotation() {
        let attitude = Attitude {
            yaw: std::f32::consts::PI / 2.0,
            pitch: std::f32::consts::PI / 2.0,
            roll: 0.0,
        };
        let point = mint::Point3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let rotated = get_rotated_point(&point, &attitude);

        assert!((rotated.x - 0.0).abs() < EPSILON);
        assert!((rotated.y - 0.0).abs() < EPSILON);
        assert!((rotated.z - -1.0).abs() < EPSILON);
    }

    #[test]
    fn update_cursor_no_wraparound() {
        let mut state = CubeState {
            camera_settings: CameraSettings::new(90, 10),
            cursor: mint::Point2 { x: 400.0, y: 300.0 },
            cube: Cube::default(),
            screen_width: 800.0,
            screen_height: 600.0,
        };

        // Test right movement
        let initial_x = state.cursor.x;
        state.update_cursor(KeyCode::Right);
        assert_eq!(
            state.cursor.x,
            (initial_x - 10.0 + state.screen_width) % state.screen_width
        );

        // Test left movement
        let initial_x = state.cursor.x;
        state.update_cursor(KeyCode::Left);
        assert_eq!(
            state.cursor.x,
            (initial_x + 10.0 + state.screen_width) % state.screen_width
        );

        // Test down movement
        let initial_y = state.cursor.y;
        state.update_cursor(KeyCode::Down);
        assert_eq!(
            state.cursor.y,
            (initial_y + 10.0 + state.screen_height) % state.screen_height
        );

        // Test up movement
        let initial_y = state.cursor.y;
        state.update_cursor(KeyCode::Up);
        assert_eq!(
            state.cursor.y,
            (initial_y - 10.0 + state.screen_height) % state.screen_height
        );
    }

    #[test]
    fn update_cursor_with_wraparound() {
        let mut state = CubeState {
            camera_settings: CameraSettings::new(90, 10),
            cursor: mint::Point2 { x: 400.0, y: 300.0 },
            cube: Cube::default(),
            screen_width: 800.0,
            screen_height: 600.0,
        };

        // Test x wraparound (right edge)
        state.cursor.x = state.screen_width - 5.0;
        state.update_cursor(KeyCode::Right);
        assert!(state.cursor.x < state.screen_width);
        assert!(state.cursor.x >= 0.0);

        // Test x wraparound (left edge)
        state.cursor.x = 5.0;
        state.update_cursor(KeyCode::Left);
        assert!(state.cursor.x < state.screen_width);
        assert!(state.cursor.x >= 0.0);

        // Test y wraparound (bottom edge)
        state.cursor.y = state.screen_height - 5.0;
        state.update_cursor(KeyCode::Down);
        assert!(state.cursor.y < state.screen_height);
        assert!(state.cursor.y >= 0.0);

        // Test y wraparound (top edge)
        state.cursor.y = 5.0;
        state.update_cursor(KeyCode::Up);
        assert!(state.cursor.y < state.screen_height);
        assert!(state.cursor.y >= 0.0);
    }

    #[test]
    fn update_cursor_does_not_move_cursor_on_invalid_key() {
        let mut state = CubeState {
            camera_settings: CameraSettings::new(90, 10),
            cursor: mint::Point2 { x: 400.0, y: 300.0 },
            cube: Cube::default(),
            screen_width: 800.0,
            screen_height: 600.0,
        };

        let initial_x = state.cursor.x;
        let initial_y = state.cursor.y;

        // Test that cursor doesn't move with invalid key
        state.update_cursor(KeyCode::Space);
        assert_eq!(state.cursor.x, initial_x);
        assert_eq!(state.cursor.y, initial_y);
    }
}
