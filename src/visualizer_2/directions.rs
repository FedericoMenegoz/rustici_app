use std::fmt::Display;

use macroquad::math::Vec2;

#[derive(PartialEq)]
/// Directions used to understand where the camera is looking
pub(crate) enum Direction {
    North,
    East,
    South,
    Ovest,
}
impl Direction {
    pub(crate) fn get_direction(looking_at: Vec2) -> Self {
        let (x_dir, y_dir) = looking_at.normalize().into();
        if x_dir.round().abs() != y_dir.round().abs() {
            match (x_dir.round() as i32, y_dir.round() as i32) {
                (1, 0) => return Self::North,
                (0, -1) => return Self::East,
                (-1, 0) => return Self::South,
                (0, 1) => return Self::Ovest,
                _ => panic!(
                    "Shouldnt be able to reach here. values should already be coverd. {:?}",
                    (x_dir.round() as i32, y_dir.round() as i32)
                ),
            };
        }

        //either North or Ovest
        if x_dir.round() == 1. && y_dir.round() == 1. {
            if x_dir > y_dir {
                return Self::North;
            } else {
                return Self::Ovest;
            }
        }

        //either North or East
        if x_dir.round() == 1. && y_dir.round() == -1. {
            if x_dir > y_dir {
                return Self::North;
            } else {
                return Self::East;
            }
        }

        //either South or Ovest
        if x_dir.round() == -1. && y_dir.round() == 1. {
            if x_dir > y_dir {
                return Self::Ovest;
            } else {
                return Self::South;
            }
        }
        //either South or East
        if x_dir > y_dir {
            return Self::East;
        } else {
            return Self::South;
        }
    }
}
impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Direction::North => write!(f, "North"),
            Direction::East => write!(f, "East"),
            Direction::South => write!(f, "South"),
            Direction::Ovest => write!(f, "Ovest"),
        }
    }
}
