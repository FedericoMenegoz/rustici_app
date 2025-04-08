use macroquad::{
    camera::{set_camera, Camera3D},
    input::{is_key_down, mouse_position},
    math::{vec3, Vec2, Vec3, Vec3Swizzles},
    miniquad::KeyCode,
    time::get_frame_time,
};

use super::directions::Direction;

/// Info about the lens angle
#[derive(Clone, Copy)]
pub(crate) struct CameraAngles {
    pub(crate) pitch: f32,
    pub(crate) yaw: f32,
}
/// Info about the lens axis
pub(crate) struct CameraAxis {
    pub(crate) front: Vec3,
    pub(crate) right: Vec3,
    pub(crate) up: Vec3,
}

pub(crate) struct Camera {
    pub(crate) position: Vec3,
    world_up: Vec3,
    pub(crate) axis: CameraAxis,
    pub(crate) angles: CameraAngles,
    move_speed: f32,
    move_speed_multiplier: f32,
    look_speed: f32,
    pub(crate) follow_robot: bool,
}

impl Default for Camera {
    fn default() -> Self {
        let world_up = vec3(0.0, 0.0, 1.0);
        let front = vec3(0.012, 0.805, -0.594);
        let right = vec3(1., -0.03, 0.0);
        let up = vec3(0.01, 0.52, 0.86);

        let axis = CameraAxis { front, right, up };
        let angles = CameraAngles {
            pitch: -0.6,
            yaw: 1.5,
        };

        return Self {
            position: vec3(34.0, -13.0, 7.5),
            world_up,
            axis,
            angles,
            move_speed: 0.25,
            move_speed_multiplier: 1.,
            look_speed: -0.07,
            follow_robot: true,
        };
    }
}
impl Camera {
    ///returns [Direction] in which the camera is looking
    pub(crate) fn get_look_direction(&self) -> Direction {
        let facing_vec = self.axis.front.cross(self.world_up);
        let facing_direction = Direction::get_direction(facing_vec.xy());
        return facing_direction;
    }

    ///According to the current mode, it follows the robot or moves freely
    pub(crate) fn handle_position(
        &mut self,
        robot_pos: (usize, usize),
        max_animation_frame: usize,
    ) {
        if !self.follow_robot {
            self.handle_inputs();
            //stops camera from going under the map
            self.position.z = self.position.z.max(0.1);
            return;
        }

        let movement_speed = 1. / max_animation_frame as f32;
        let robot_vec = vec3(robot_pos.0 as f32, robot_pos.1 as f32, self.position.z);
        let direction_x = if self.position.x < robot_vec.x {
            1.
        } else if self.position.x > robot_vec.x {
            -1.
        } else {
            0.
        };

        let allowed_y = -10.;
        let direction_y = if self.position.y - robot_vec.y < allowed_y {
            1.
        } else if self.position.y - robot_vec.y > allowed_y {
            -1.
        } else {
            0.
        };

        self.position.x += direction_x * movement_speed;
        self.position.y += direction_y * movement_speed;
        self.position.z = 7.5;
    }

    ///Last update to the camera after end of the simulation, to avoid an infinite wiggle
    pub(crate) fn fix_final_camera(&mut self, robot_pos: &(usize, usize)) {
        if self.follow_robot {
            let robot_vec = vec3(robot_pos.0 as f32, robot_pos.1 as f32 - 10., 7.5).round();
            self.position = robot_vec;
        }
    }
    ///Handles how the camera moves based on the mouse movements
    pub(crate) fn handle_direction(&mut self, last_mouse_position: &mut Vec2) {
        set_camera(&Camera3D {
            position: self.position,
            up: self.axis.up,
            target: self.position + self.axis.front,
            ..Default::default()
        });

        if self.follow_robot {
            return;
        }
        let delta = get_frame_time();
        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - *last_mouse_position;
        *last_mouse_position = mouse_position;

        self.angles.yaw += mouse_delta.x * delta * self.look_speed;
        self.angles.pitch += mouse_delta.y * delta * self.look_speed;

        //locks camera between +90° and -90°
        self.angles.pitch = self.angles.pitch.min(1.5);
        self.angles.pitch = self.angles.pitch.max(-1.5);
        self.angles.yaw = self.angles.yaw % 6.283;

        self.axis.front = vec3(
            self.angles.yaw.cos() * self.angles.pitch.cos(),
            self.angles.yaw.sin() * self.angles.pitch.cos(),
            self.angles.pitch.sin(),
        )
        .normalize();
        self.axis.right = self.axis.front.cross(self.world_up).normalize();
        self.axis.up = self.axis.right.cross(self.axis.front).normalize();
    }

    ///Handles all keyboard inputs related to the camera
    fn handle_inputs(&mut self) {
        if is_key_down(KeyCode::LeftShift) {
            self.move_speed_multiplier = (self.move_speed_multiplier + 0.06).min(2.6);
        } else {
            self.move_speed_multiplier = (self.move_speed_multiplier - 0.08).max(1.);
        }

        if is_key_down(KeyCode::W) {
            let forward =
                self.axis.front.xy().normalize() * self.move_speed * self.move_speed_multiplier;
            self.position += vec3(forward.x, forward.y, 0.);
        }
        if is_key_down(KeyCode::S) {
            let forward =
                self.axis.front.xy().normalize() * self.move_speed * self.move_speed_multiplier;
            self.position -= vec3(forward.x, forward.y, 0.);
        }
        if is_key_down(KeyCode::A) {
            let forward =
                self.axis.right.xy().normalize() * self.move_speed * self.move_speed_multiplier;
            self.position -= vec3(forward.x, forward.y, 0.);
        }
        if is_key_down(KeyCode::D) {
            let forward =
                self.axis.right.xy().normalize() * self.move_speed * self.move_speed_multiplier;
            self.position += vec3(forward.x, forward.y, 0.);
        }

        if is_key_down(KeyCode::Space) {
            self.position += self.world_up * self.move_speed * 0.9;
        }
        if is_key_down(KeyCode::LeftControl) {
            self.position -= self.world_up * self.move_speed * 0.9;
        }
    }

    ///toggles between free-camera mode and following-robot mode
    pub(crate) fn change_follow_robot(&mut self, robot_pos: &(usize, usize)) {
        self.follow_robot = !self.follow_robot;
        let robot_vec = vec3(robot_pos.0 as f32, robot_pos.1 as f32 - 10., 7.5).round();
        self.position = robot_vec;
        self.axis.front = vec3(0.012, 0.805, -0.594);
        self.axis.right = vec3(1., -0.03, 0.0);
        self.axis.up = vec3(0.013, 0.52, 0.86);
    }
}
