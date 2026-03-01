/// 2D vector for positions and velocities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[allow(dead_code)]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len < 1e-9 {
            Vec2::ZERO
        } else {
            Vec2::new(self.x / len, self.y / len)
        }
    }

    pub fn distance_to(self, other: Vec2) -> f64 {
        (self - other).length()
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

/// Axis-aligned bounding box for collision detection.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rect { x, y, width, height }
    }

    /// AABB intersection test.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns this rect offset by the given position (for entity hitboxes).
    pub fn at(&self, pos: Vec2) -> Rect {
        Rect::new(pos.x + self.x, pos.y + self.y, self.width, self.height)
    }
}

/// Countdown timer / cooldown helper.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Timer {
    pub duration: f64,
    pub elapsed: f64,
    pub active: bool,
}

#[allow(dead_code)]
impl Timer {
    pub fn new(duration: f64) -> Self {
        Timer { duration, elapsed: 0.0, active: false }
    }

    pub fn start(&mut self) {
        self.elapsed = 0.0;
        self.active = true;
    }

    /// Returns true once when the timer completes. Resets elapsed.
    pub fn tick(&mut self, dt: f64) -> bool {
        if !self.active {
            return false;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.elapsed = 0.0;
            true
        } else {
            false
        }
    }

    pub fn fraction(&self) -> f64 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).min(1.0)
    }

    pub fn remaining(&self) -> f64 {
        (self.duration - self.elapsed).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_rect_no_intersect() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(11.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_rect_touching_edge_no_intersect() {
        // Touching at edge is NOT an intersection (strict less-than)
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(10.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_vec2_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.distance_to(b) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_vec2_normalized() {
        let v = Vec2::new(0.0, 5.0);
        let n = v.normalized();
        assert!((n.x).abs() < 1e-9);
        assert!((n.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_timer_fires() {
        let mut t = Timer::new(1.0);
        t.start();
        assert!(!t.tick(0.5));
        assert!(t.tick(0.6));
    }

    #[test]
    fn test_timer_inactive() {
        let mut t = Timer::new(1.0);
        assert!(!t.tick(2.0));
    }
}
