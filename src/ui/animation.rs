//! Animation system with easing functions and frame-based updates
//!
//! Implements T387, T390-T395:
//! - Easing functions (ease-in, ease-out, ease-in-out)
//! - AnimatedValue<T> for smooth transitions
//! - Frame-based animation loop with QueryPerformanceCounter
//! - Spring physics for natural motion
//! - System animation preference detection

use std::time::{Duration, Instant};

/// Easing function types (T390)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingFunction {
    /// Linear interpolation (no easing)
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in and out (slow start and end)
    EaseInOut,
    /// Spring physics (overshoot and settle)
    Spring,
}

impl EasingFunction {
    /// Apply easing function to normalized time (0.0 - 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => t * (2.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingFunction::Spring => {
                // Spring with damping
                let s = 1.70158;
                let t = t - 1.0;
                t * t * ((s + 1.0) * t + s) + 1.0
            }
        }
    }
}

/// Animated value that smoothly transitions between values (T391)
#[derive(Debug, Clone)]
pub struct AnimatedValue<T> {
    /// Current value
    current: T,
    /// Target value
    target: T,
    /// Start value (for interpolation)
    start: T,
    /// Animation duration
    duration: Duration,
    /// Time when animation started
    start_time: Option<Instant>,
    /// Easing function
    easing: EasingFunction,
}

impl AnimatedValue<f32> {
    /// Create new animated float value
    pub fn new(initial: f32, duration: Duration, easing: EasingFunction) -> Self {
        Self {
            current: initial,
            target: initial,
            start: initial,
            duration,
            start_time: None,
            easing,
        }
    }

    /// Set new target value and start animation
    pub fn set_target(&mut self, target: f32) {
        if (self.target - target).abs() > 0.001 {
            self.start = self.current;
            self.target = target;
            self.start_time = Some(Instant::now());
        }
    }

    /// Update animation (call each frame)
    pub fn update(&mut self) {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            
            if elapsed >= self.duration {
                // Animation complete
                self.current = self.target;
                self.start_time = None;
            } else {
                // Interpolate
                let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
                let eased_t = self.easing.apply(t);
                self.current = self.start + (self.target - self.start) * eased_t;
            }
        }
    }

    /// Get current value
    pub fn value(&self) -> f32 {
        self.current
    }

    /// Check if animation is running
    pub fn is_animating(&self) -> bool {
        self.start_time.is_some()
    }

    /// Get animation progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        } else {
            1.0
        }
    }
}

/// Animation manager for frame-based updates (T392)
pub struct AnimationManager {
    /// Last frame time for delta calculation
    last_frame: Instant,
    /// Frame delta time
    delta_time: Duration,
    /// Animation enabled (system preference)
    animations_enabled: bool,
    /// Target FPS for animations
    target_fps: u32,
}

impl AnimationManager {
    /// Create new animation manager (T392)
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            delta_time: Duration::from_millis(16), // ~60 FPS
            animations_enabled: Self::check_system_animations(),
            target_fps: 60,
        }
    }

    /// Begin new frame (call at start of render loop)
    pub fn begin_frame(&mut self) {
        let now = Instant::now();
        self.delta_time = now - self.last_frame;
        self.last_frame = now;
    }

    /// Get delta time since last frame
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    /// Get current FPS
    pub fn current_fps(&self) -> f32 {
        if self.delta_time.as_secs_f32() > 0.0 {
            1.0 / self.delta_time.as_secs_f32()
        } else {
            60.0
        }
    }

    /// Check if animations should be enabled (T395)
    pub fn are_animations_enabled(&self) -> bool {
        self.animations_enabled
    }

    /// Set animation enabled state
    pub fn set_animations_enabled(&mut self, enabled: bool) {
        self.animations_enabled = enabled;
    }

    /// Check if frame should be skipped for target FPS
    pub fn should_skip_frame(&self) -> bool {
        let target_delta = Duration::from_secs_f32(1.0 / self.target_fps as f32);
        self.delta_time < target_delta
    }

    /// Detect system animation preferences (T395)
    ///
    /// Checks Windows Settings > Accessibility > Visual effects > Animation effects
    fn check_system_animations() -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION};
        use windows::core::BOOL;

        unsafe {
            let mut enabled: BOOL = BOOL(1);
            let result = SystemParametersInfoW(
                SPI_GETCLIENTAREAANIMATION,
                0,
                Some(&mut enabled as *mut BOOL as *mut _),
                Default::default(),
            );

            result.is_ok() && enabled.as_bool()
        }
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Spring physics for natural motion (T394)
#[derive(Debug, Clone)]
pub struct SpringPhysics {
    /// Current position
    position: f32,
    /// Current velocity
    velocity: f32,
    /// Target position
    target: f32,
    /// Spring stiffness (higher = faster)
    stiffness: f32,
    /// Damping coefficient (higher = less oscillation)
    damping: f32,
}

impl SpringPhysics {
    /// Create new spring with default parameters
    pub fn new(initial: f32) -> Self {
        Self {
            position: initial,
            velocity: 0.0,
            target: initial,
            stiffness: 170.0,
            damping: 26.0,
        }
    }

    /// Create spring with custom parameters
    pub fn with_params(initial: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            position: initial,
            velocity: 0.0,
            target: initial,
            stiffness,
            damping,
        }
    }

    /// Set new target
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Update spring physics (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        // Spring force: F = -k * x
        let displacement = self.position - self.target;
        let spring_force = -self.stiffness * displacement;
        
        // Damping force: F = -c * v
        let damping_force = -self.damping * self.velocity;
        
        // Total force
        let force = spring_force + damping_force;
        
        // Update velocity and position
        self.velocity += force * delta_time;
        self.position += self.velocity * delta_time;
    }

    /// Get current position
    pub fn position(&self) -> f32 {
        self.position
    }

    /// Check if spring is at rest (near target with low velocity)
    pub fn is_at_rest(&self) -> bool {
        let position_threshold = 0.01;
        let velocity_threshold = 0.01;
        
        (self.position - self.target).abs() < position_threshold
            && self.velocity.abs() < velocity_threshold
    }
}

/// Common animation presets for UI elements (T393)
pub mod presets {
    use super::*;

    /// Quick button hover animation (150ms)
    pub fn button_hover() -> AnimatedValue<f32> {
        AnimatedValue::new(0.0, Duration::from_millis(150), EasingFunction::EaseOut)
    }

    /// Selection change animation (200ms)
    pub fn selection_change() -> AnimatedValue<f32> {
        AnimatedValue::new(0.0, Duration::from_millis(200), EasingFunction::EaseInOut)
    }

    /// Panel expand/collapse animation (300ms)
    pub fn panel_expand() -> AnimatedValue<f32> {
        AnimatedValue::new(0.0, Duration::from_millis(300), EasingFunction::EaseInOut)
    }

    /// Smooth fade animation (250ms)
    pub fn fade() -> AnimatedValue<f32> {
        AnimatedValue::new(0.0, Duration::from_millis(250), EasingFunction::EaseOut)
    }

    /// Spring bounce for buttons
    pub fn button_spring() -> SpringPhysics {
        SpringPhysics::with_params(0.0, 200.0, 20.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_functions() {
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert!(EasingFunction::EaseIn.apply(0.5) < 0.5);
        assert!(EasingFunction::EaseOut.apply(0.5) > 0.5);
    }

    #[test]
    fn test_animated_value() {
        let mut anim = AnimatedValue::new(0.0, Duration::from_millis(100), EasingFunction::Linear);
        
        assert_eq!(anim.value(), 0.0);
        assert!(!anim.is_animating());
        
        anim.set_target(10.0);
        assert!(anim.is_animating());
        
        // After full duration, should reach target
        std::thread::sleep(Duration::from_millis(110));
        anim.update();
        assert!(!anim.is_animating());
        assert!((anim.value() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_animation_manager() {
        let manager = AnimationManager::new();
        assert_eq!(manager.target_fps, 60);
    }

    #[test]
    fn test_spring_physics() {
        let mut spring = SpringPhysics::new(0.0);
        
        spring.set_target(10.0);
        assert_eq!(spring.position(), 0.0);
        
        // Update for a few frames
        for _ in 0..60 {
            spring.update(1.0 / 60.0);
        }
        
        // Should have moved toward target
        assert!(spring.position() > 0.0);
    }

    #[test]
    fn test_spring_settles() {
        let mut spring = SpringPhysics::new(0.0);
        spring.set_target(5.0);
        
        // Run until settled
        for _ in 0..300 {
            spring.update(1.0 / 60.0);
            if spring.is_at_rest() {
                break;
            }
        }
        
        // Should be close to target
        assert!((spring.position() - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_animation_presets() {
        let hover = presets::button_hover();
        assert!(!hover.is_animating());
        
        let spring = presets::button_spring();
        assert_eq!(spring.position(), 0.0);
    }
}
