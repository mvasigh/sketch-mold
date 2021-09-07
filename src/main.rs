use nannou::prelude::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const HEADING_ANGLE: f32 = 1.0;
const SENSE_ANGLE: f32 = 1.0;
const SENSE_DISTANCE: f32 = 1.0;
const TURN_ANGLE: f32 = 1.0;
const DEPOSIT_AMOUNT: f32 = 0.7;

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

    pub fn cell_at(&self, row: usize, col: usize) -> &Cell {
        &self.cells[(row * WIDTH) + col]
    }

    pub fn cell_at_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.cells[(row * WIDTH) + col]
    }

    pub fn update(&self) {
        // TODO
        // 1. Perform a blur on the trail array
    }

    pub fn draw(&self, app: &App, model: &Model, frame: &Frame, draw: &Draw) {
        let width = WIDTH as u32;
        let height = HEIGHT as u32;
        let image = nannou::image::ImageBuffer::from_fn(width, height, |x, y| {
            let cell = self.cell_at(x as usize, y as usize);
            let color = map_range(clamp(cell.intensity, 0.0, 1.0), 0.0, 1.0, 0, std::u8::MAX);

            nannou::image::Rgba([color, color, color, std::u8::MAX])
        });
        let flat_samples = image.as_flat_samples();
        model.texture.upload_data(
            app.main_window().swap_chain_device(),
            &mut *frame.command_encoder(),
            &flat_samples.as_slice(),
        );

        draw.texture(&model.texture);
    }
}

struct Model {
    _window: WindowId,
    grid: Grid,
    particles: Vec<Particle>,
    texture: wgpu::Texture,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let width = WIDTH as u32;
    let height = HEIGHT as u32;

    let _window = app
        .new_window()
        .size(width, height)
        .view(view)
        .build()
        .unwrap();

    let grid = Grid::new(WIDTH, HEIGHT);
    let particles = (0..WIDTH * HEIGHT).map(|_| Particle::random()).collect();
    let texture = wgpu::TextureBuilder::new()
        .size([width, height])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .usage(wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED)
        .build(app.main_window().swap_chain_device());

    Model {
        _window,
        grid,
        particles,
        texture,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);

    // Paint the grid!
    model.grid.draw(app, model, &frame, &draw);

    draw.to_frame(app, &frame).unwrap();
}
