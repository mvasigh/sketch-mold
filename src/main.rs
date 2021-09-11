use nannou::prelude::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const HEADING_DISTANCE: f32 = 0.8;
const SENSE_ANGLE: f32 = 1.5;
const SENSE_DISTANCE: f32 = 4.0;
const TURN_ANGLE: f32 = 0.2;
const DEPOSIT_AMOUNT: f32 = 0.8;
const DECAY_AMOUNT: f32 = 0.02;

fn cart_to_canvas(pt: Vector2) -> Vector2 {
    let x = pt.x + (WIDTH as f32 / 2.0);
    let y = (pt.y - (WIDTH as f32 / 2.0)).abs();

    vec2(x, y)
}

fn canvas_to_cart(pt: Vector2) -> Vector2 {
    let x = pt.x - (WIDTH as f32 / 2.0);
    let y = (HEIGHT as f32 / 2.0) - pt.y;

    vec2(x, y)
}

fn move_coords(curr: Vector2, d: f32, angle: f32) -> Vector2 {
    let x = curr.x + (d * angle.cos());
    let y = curr.y + (d * angle.sin());

    vec2(x, y)
}

struct Particle {
    pos: Vector2,
    heading_angle: f32,
    heading_distance: f32,
    sense_angle: f32,
    sense_distance: f32,
    turn_angle: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32) -> Particle {
        Particle {
            pos: vec2(x, y),
            heading_angle: random_range(0.0, 6.28),
            heading_distance: HEADING_DISTANCE,
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

    pub fn update(&mut self, grid: &mut Grid) {
        // 1. Get left, center, and right sensor points
        let left_pt = cart_to_canvas(move_coords(
            canvas_to_cart(self.pos),
            self.sense_distance,
            self.heading_angle - self.sense_angle,
        ));
        let center_pt = cart_to_canvas(move_coords(
            canvas_to_cart(self.pos),
            self.sense_distance,
            self.heading_angle,
        ));
        let right_pt = cart_to_canvas(move_coords(
            canvas_to_cart(self.pos),
            self.sense_distance,
            self.heading_angle + self.sense_angle,
        ));

        // 2. Get the cells at each center point
        let left_cell = grid.cell_at_pt(left_pt);
        let center_cell = grid.cell_at_pt(center_pt);
        let right_cell = grid.cell_at_pt(right_pt);

        // 3. Determine the new heading angle based on the sensors
        if left_cell.intensity > center_cell.intensity && left_cell.intensity > right_cell.intensity
        {
            self.heading_angle -= self.turn_angle;
        } else if right_cell.intensity > center_cell.intensity
            && right_cell.intensity > left_cell.intensity
        {
            self.heading_angle += self.turn_angle;
        } else if left_cell.intensity == right_cell.intensity {
            match random_f32() > 0.5 {
                true => self.heading_angle -= self.turn_angle,
                false => self.heading_angle += self.turn_angle,
            }
        }

        // 4. Deposit some pheromone at current location
        let curr_cell = grid.cell_at_pt_mut(self.pos);
        curr_cell.deposit();

        // 5. Move particle to new destination
        self.pos = cart_to_canvas(move_coords(
            canvas_to_cart(self.pos),
            self.heading_distance,
            self.heading_angle,
        ));

        let width = WIDTH as f32;
        let height = HEIGHT as f32;

        if self.pos.x < 0.0 {
            self.pos.x = width - self.pos.x;
        }

        if self.pos.x > width {
            self.pos.x = self.pos.x - width;
        }

        if self.pos.y < 0.0 {
            self.pos.y = width - self.pos.y;
        }

        if self.pos.y > height {
            self.pos.y = height - self.pos.y;
        }
    }
}

struct Cell {
    intensity: f32,
}

impl Cell {
    pub fn decay(&mut self) {
        self.intensity = clamp(self.intensity - DECAY_AMOUNT, 0.0, 1.0);
    }

    pub fn deposit(&mut self) {
        self.intensity = clamp(self.intensity + DEPOSIT_AMOUNT, 0.0, 1.0);
    }
}

struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        let cells = (0..width * height)
            .map(|_| Cell {
                intensity: random_f32(),
            })
            .collect();

        Grid {
            cells,
            width,
            height,
        }
    }

    pub fn cell_at(&self, row: usize, col: usize) -> &Cell {
        &self.cells[(row * self.width) + col]
    }

    pub fn cell_at_pt(&self, loc: Vector2<f32>) -> &Cell {
        let x_size = loc.x as usize;
        let y_size = loc.y as usize;

        let x = (self.width + x_size) % self.width;
        let y = (self.height + y_size) % self.height;

        &self.cells[(x * self.width) + y as usize]
    }

    pub fn cell_at_pt_mut(&mut self, loc: Vector2<f32>) -> &mut Cell {
        let x_size = loc.x as usize;
        let y_size = loc.y as usize;

        let x = (self.width + x_size) % self.width;
        let y = (self.height + y_size) % self.height;

        &mut self.cells[(x * self.width) + y as usize]
    }

    pub fn cell_at_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.cells[(row * self.width) + col]
    }

    pub fn update(&mut self) {
        // 1. Decay every cell
        for cell in self.cells.iter_mut() {
            cell.decay();
        }
        // 2. Perform a blur on the trail array
        // TODO
    }

    pub fn draw(&self, app: &App, model: &Model, frame: &Frame, draw: &Draw) {
        let width = self.width as u32;
        let height = self.height as u32;
        let image = nannou::image::ImageBuffer::from_fn(width, height, |x, y| {
            let cell = self.cell_at(x as usize, y as usize);
            let color = map_range(
                clamp(cell.intensity, 0.0, 1.0),
                0.0,
                1.0,
                3,
                std::u8::MAX - 5,
            );

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
    let particles = (0..10000).map(|_| Particle::random()).collect();
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

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Update all of the particles
    for particle in model.particles.iter_mut() {
        particle.update(&mut model.grid);
    }

    // Update the grid
    model.grid.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(BLACK);

    // Paint the grid!
    model.grid.draw(app, model, &frame, &draw);

    draw.to_frame(app, &frame).unwrap();
}
