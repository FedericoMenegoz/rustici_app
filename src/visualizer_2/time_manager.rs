/// Controls the time and it's speed
pub(crate) struct TimeManager {
    current_time: usize,
    speed: usize,
    possible_speed: Vec<usize>,
}

impl Default for TimeManager {
    fn default() -> Self {
        Self {
            current_time: 0,
            speed: 4,
            possible_speed: vec![usize::MAX, 100, 60, 45, 33, 25, 15, 10, 5, 2, 1],
        }
    }
}
impl TimeManager {
    ///Get index of the speed
    pub(crate) fn get_speed(&self) -> usize {
        self.speed
    }

    ///Get value of the speed
    pub(crate) fn get_speed_value(&self) -> usize {
        self.possible_speed[self.speed]
    }
    /// If enough frame have passed. It will return true. It's based on the current speed
    pub(crate) fn should_update(&self) -> bool {
        self.current_time % self.get_speed_value() == 0
    }
    /// Updates clock
    pub(crate) fn update(&mut self) {
        self.current_time += 1;
    }
    ///it will try to make time go faster or slower
    pub(crate) fn change_speed(&mut self, increase: bool) {
        if increase {
            self.speed = (self.speed + 1).min(self.possible_speed.len() - 1);
        } else {
            if self.speed > 0 {
                self.speed = self.speed - 1;
            } else {
                self.speed = 0;
            }
        };
    }
}
