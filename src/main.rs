use std::time::Duration;

use nalgebra::{Point2, Rotation2, Vector2};
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::Color,
    rect::Rect,
    render::{Canvas, RenderTarget, TextureCreator},
    video::WindowContext,
    EventPump,
};
const SCREEN_DIMENSIONS: (i32, i32) = (1920, 1080);

struct Car {
    dimensions: Vector2<f64>,
    pos: Point2<f64>,
    rotation: Rotation2<f64>,
    velocity: Vector2<f64>,

    wheel_speed: f64,
    acceleration: f64,
    max_speed: f64,
}

struct Camera {
    pub pos: Point2<f64>,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            pos: Point2::new(1000., 700.),
        }
    }

    pub fn relative_rect(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x - (self.pos.x as i32 - SCREEN_DIMENSIONS.0 / 2),
            rect.y - (self.pos.y as i32 - SCREEN_DIMENSIONS.1 / 2),
            rect.width(),
            rect.height(),
        )
    }

    fn update(&mut self, car: &Car) {
        self.pos = self.pos.coords.lerp(&car.center().coords, 0.2).into();
    }
}

enum CarSteering {
    Left,
    Right,
    None,
}

enum CarPedal {
    Forward,
    Backward,
    None,
}

impl Car {
    pub fn new() -> Car {
        Car {
            dimensions: Vector2::new(50., 100.),
            pos: Point2::new(1000., 700.),
            rotation: Rotation2::new(0.),
            velocity: Vector2::zeros(),

            wheel_speed: 0.,
            max_speed: 1.,
            acceleration: 0.1,
        }
    }

    pub fn center(&self) -> Point2<f64> {
        self.pos + self.dimensions / 2.
    }

    pub fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x as i32,
            self.pos.y as i32,
            self.dimensions.x as u32,
            self.dimensions.y as u32,
        )
    }

    fn update(&mut self, pedal: CarPedal, steering: CarSteering) {
        if let CarPedal::Forward = pedal {
            self.wheel_speed += self.acceleration;
            let max_backwards_speed = -5.;
            self.wheel_speed = self.wheel_speed.clamp(max_backwards_speed, self.max_speed);
        } else if let CarPedal::Backward = pedal {
            // only gonna implement brake for now, but will also need to detect when to go into
            // reverse some time
            // let brake_force = 0.5;
            // self.velocity += -self.velocity.normalize() * brake_force;
            self.wheel_speed *= 0.5;
            if self.wheel_speed < 0.1 {
                self.wheel_speed = 0.;
            }
        }

        self.pos -= self.dimensions / 2.; // to center the rotation
        let rotation_strength = (self.rotation * self.velocity).magnitude().abs();
        if let CarSteering::Left = steering {
            self.rotation *= Rotation2::new(-0.005 * rotation_strength);
        } else if let CarSteering::Right = steering {
            self.rotation *= Rotation2::new(0.005 * rotation_strength);
        }
        self.pos += self.dimensions / 2.; // to bring the car back to where it should be

        // friction
        let mut local_velocity = self.rotation.inverse() * self.velocity;

        let vertical_friction = 0.02;
        local_velocity.y -= self.wheel_speed;

        self.wheel_speed *= 0.98 - vertical_friction;
        local_velocity.y *= 1. - vertical_friction;

        let horizontal_friction = 0.05;
        local_velocity.x *= 1.0 - horizontal_friction;

        self.velocity = self.rotation * local_velocity;
        self.pos += self.velocity;
    }
}

trait Scene {
    fn update(&mut self, events: &mut EventPump) -> Result<Option<impl Scene>, ()>;
    fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        texture_creator: &TextureCreator<WindowContext>,
    );
}

struct Level {
    car: Car,
    camera: Camera,
}

impl Level {
    pub fn new() -> Level {
        Level {
            car: Car::new(),
            camera: Camera::new(),
        }
    }

    fn draw_checkerboard<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
        let square_size: u32 = 125;
        let (width, height) = canvas.output_size().unwrap();

        (0..width).step_by(square_size as usize).for_each(|x| {
            (0..height).step_by(square_size as usize).for_each(|y| {
                canvas.set_draw_color(if (x / square_size + y / square_size) % 2 == 0 {
                    Color::RGB(60, 180, 35)
                } else {
                    Color::RGB(60, 200, 35)
                });
                canvas
                    .fill_rect(self.camera.relative_rect(Rect::new(
                        x as i32,
                        y as i32,
                        square_size,
                        square_size,
                    )))
                    .unwrap();
            })
        })
    }
}

impl Scene for Level {
    fn update(&mut self, events: &mut EventPump) -> Result<Option<Level>, ()> {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape | Keycode::Q),
                    ..
                } => return Err(()),
                _ => {}
            }
        }

        let key_state = events.keyboard_state();

        let pedal = if key_state.is_scancode_pressed(Scancode::W) {
            CarPedal::Forward
        } else if key_state.is_scancode_pressed(Scancode::S) {
            CarPedal::Backward
        } else {
            CarPedal::None
        };
        let steering = if key_state.is_scancode_pressed(Scancode::A)
            && !key_state.is_scancode_pressed(Scancode::D)
        {
            CarSteering::Left
        } else if key_state.is_scancode_pressed(Scancode::D)
            && !key_state.is_scancode_pressed(Scancode::A)
        {
            CarSteering::Right
        } else {
            CarSteering::None
        };

        self.car.update(pedal, steering);
        self.camera.update(&self.car);

        Ok(None)
    }

    fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        texture_creator: &TextureCreator<WindowContext>,
    ) {
        canvas.set_draw_color(Color::GREY);
        canvas.clear();
        self.draw_checkerboard(canvas);

        let mut car_texture = texture_creator
            .create_texture_target(None, self.car.rect().width(), self.car.rect().height())
            .unwrap();
        canvas
            .with_texture_canvas(&mut car_texture, |texture_canvas| {
                texture_canvas.set_draw_color(Color::RED);
                texture_canvas.clear();
            })
            .unwrap();

        canvas.set_draw_color(Color::RED);
        // let mut car_rect = self.car.rect();
        // car_rect.reposition(self.camera.relative_rect(car_rect.top_left()));
        canvas
            .copy_ex(
                &car_texture,
                None,
                // Some(self.car.rect()),
                Some(self.camera.relative_rect(self.car.rect())),
                self.car.rotation.angle() * 180. / std::f64::consts::PI,
                // Some(self.camera.relative_rect(self.car.rect()).top_left()),
                None,
                false,
                false,
            )
            .unwrap();
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Sdl2 test", 2550, 1440)
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut level = Level::new();
    loop {
        let mut texture = texture_creator
            .create_texture_target(None, 1920, 1080)
            .unwrap();
        canvas
            .with_texture_canvas(&mut texture, |texture_canvas| {
                level.render(texture_canvas, &texture_creator)
            })
            .unwrap();

        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        if let Err(_) = level.update(&mut event_pump) {
            break;
        };

        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }
}
