use nannou::prelude::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const HEADING_ANGLE: f32 = 1.0;
const SENSE_ANGLE: f32 = 1.0;
const SENSE_DISTANCE: f32 = 1.0;
const TURN_ANGLE: f32 = 1.0;

struct Particle {
    pos: Vector2,
    heading_angle: f32,
    sense_angle: f32,
    sense_distance: f32,
    turn_angle: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32) -> Particle {
        Particle {
            pos: vec2(x, y),
            heading_angle: HEADING_ANGLE,
            sense_angle: SENSE_ANGLE,
            sense_distance: SENSE_DISTANCE,
            turn_angle: TURN_ANGLE,
        }
    }

    pub fn random() -> Particle {
        let x = random_range(0.0, WIDTH as f32);
        let y = random_range(0.0, HEIGHT as f32);

        Particle::new(x, y)
    }

    pub fn update(&self) {
        // TODO
    }
}

struct Cell {
    intensity: f32,
}

struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        let cells = (0..width * height)
            .map(|_| Cell { intensity: 0.0 })
            .collect();

        Grid {
            cells,
            width,
            height,
        }
    }

    pub fn update(&self) {
        // TODO
    }

    pub fn draw(&self) {
        // TODO
    }
}

struct Model {
    _window: WindowId,
    grid: Grid,
    particles: Vec<Particle>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(WIDTH as u32, HEIGHT as u32)
        .view(view)
        .build()
        .unwrap();

    let grid = Grid::new(WIDTH, HEIGHT);
    let particles = (0..100).map(|_| Particle::random()).collect();

    Model {
        _window,
        grid,
        particles,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);

    draw.to_frame(app, &frame).unwrap();
}
