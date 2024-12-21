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

struct Car {
    pub dimensions: Vector2<f64>,
    pub pos: Point2<f64>,
    pub rotation: Rotation2<f64>,
    velocity: Vector2<f64>,
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
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x as i32, self.pos.y as i32, 10, 10)
    }
}

impl Car {
    pub fn new() -> Car {
        Car {
            dimensions: Vector2::new(50., 100.),
            pos: Point2::new(1000., 700.),
            rotation: Rotation2::new(0.),
            velocity: Vector2::zeros(),
            max_speed: 50.,
        }
    }

    pub fn center(&self) -> Point2<f64> {
        self.pos + self.dimensions.scale(0.5)
    }

    pub fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x as i32,
            self.pos.y as i32,
            self.dimensions.x as u32,
            self.dimensions.y as u32,
        )
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

    fn draw_checkerboard<T: RenderTarget>(canvas: &mut Canvas<T>) {
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
                    .fill_rect(Rect::new(x as i32, y as i32, square_size, square_size))
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

        if key_state.is_scancode_pressed(Scancode::E) {
            self.car.rotation *= Rotation2::new(0.1);
        }

        if key_state.is_scancode_pressed(Scancode::W) {
            self.car.velocity += self.car.rotation * Vector2::new(0., -1.)
        }
        if key_state.is_scancode_pressed(Scancode::S) {
            // only gonna implement brake for now, but will also need to detect when to go into
            // reverse some time
            let brake_force = 0.5;
            self.car.velocity += -self.car.velocity.normalize() * brake_force;
        }
        if key_state.is_scancode_pressed(Scancode::A) {
            self.car.rotation *= Rotation2::new(-0.1);
        }
        if key_state.is_scancode_pressed(Scancode::D) {
            self.car.rotation *= Rotation2::new(0.1);
        }

        let friction_coefficient = 0.01;
        let friction_force = -self.car.velocity * friction_coefficient;

        self.car.velocity += friction_force;
        if self.car.velocity.magnitude() < 0.01 {
            self.car.velocity = Vector2::zeros();
        }

        self.car.pos += self.car.velocity;

        // CAMERA

        // self.camera.velocity = self.camera.velocity.slerp(&self.car.pos.coords, 1.0);
        // self.camera.pos = self.camera.velocity;
        self.camera.pos = self
            .camera
            .pos
            .coords
            .lerp(&self.car.center().coords, 0.2)
            .into();

        Ok(None)
    }

    fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        texture_creator: &TextureCreator<WindowContext>,
    ) {
        // canvas.set_draw_color(Color::GREY);
        // canvas.clear();
        Level::draw_checkerboard(canvas);

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
        canvas
            .copy_ex(
                &car_texture,
                None,
                Some(self.car.rect()),
                self.car.rotation.angle() * 180. / std::f64::consts::PI,
                None,
                false,
                false,
            )
            .unwrap();

        canvas.set_draw_color(Color::BLACK);
        canvas.fill_rect(self.camera.rect()).unwrap();
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Sdl2 test", 1920, 1080)
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut level = Level::new();
    'running: loop {
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
