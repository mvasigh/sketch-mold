#[macro_use]
extern crate lazy_static;

use lerp::Lerp;
use nannou::prelude::*;
use rand_distr::{Distribution, Normal};

const SCALE: usize = 2;
const SCALE_F32: f32 = SCALE as f32;
const SCALE_DIST: f32 = 0.75;
const WIDTH: usize = 400 * SCALE;
const HEIGHT: usize = 400 * SCALE;
const NUM_PARTICLES: usize = 200000 * SCALE;
const HEADING_DISTANCE: f32 = 1.125 * SCALE_F32 * SCALE_DIST;
const SENSE_ANGLE: f32 = PI * 0.5;
const SENSE_DISTANCE: f32 = 1.25 * SCALE_F32 * SCALE_DIST;
const TURN_ANGLE: f32 = PI * 0.25;
const DEPOSIT_AMOUNT: f32 = 0.31;
const DECAY_AMOUNT: f32 = 0.065;
const BLUR_RADIUS: isize = 1;
const MIN_CUTOFF: f32 = 0.1;
const MAX_CUTOFF: f32 = 0.8;
const BLUR_INTENSITY: f32 = 0.35;

const MAX_COLOR: [u8; 3] = [139, 77, 219];
const MIN_COLOR: [u8; 3] = [252, 33, 73];

const IMG_OUTPUT: bool = true;

fn cart_to_canvas(pt: Vector2) -> Vector2 {
    let x = pt.x + (WIDTH as f32 / 2.0);
    let y = (pt.y - (WIDTH as f32 / 2.0)) * -1.0;

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

fn inf_coords(x: isize, y: isize) -> usize {
    let w = WIDTH as isize;
    let h = HEIGHT as isize;
    let ind_x = (x + w) % w;
    let ind_y = (y + h) % h;
    let index = (ind_y * w) + ind_x;

    index as usize
}

#[derive(Debug)]
struct Particle {
    pos: Vector2,
    heading_angle: f32,
    heading_distance: f32,
    sense_angle: f32,
    sense_distance: f32,
    turn_angle: f32,
    color: [f32; 3],
}

impl Particle {
    pub fn new(x: f32, y: f32, a: f32) -> Particle {
        Particle {
            pos: vec2(x, y),
            heading_angle: a,
            heading_distance: HEADING_DISTANCE,
            sense_angle: SENSE_ANGLE,
            sense_distance: SENSE_DISTANCE,
            turn_angle: TURN_ANGLE,
            color: [1.0, 1.0, 1.0],
        }
    }

    pub fn random() -> Particle {
        let x = random_range(0.0, WIDTH as f32);
        let y = random_range(0.0, HEIGHT as f32);

        Particle::new(x, y, random_range(0.0, TAU))
    }

    pub fn center(radius: f32) -> Particle {
        lazy_static! {
            static ref NORMAL: Normal<f32> = Normal::new(0.0, 0.4).unwrap();
        }

        let v = NORMAL.sample(&mut rand::thread_rng());
        let r = map_range(1.0 - v.abs(), 0.0, 1.0,radius / 4.0, radius);
        let a = random_range(0.0, 2.0 * PI);
        let dx = r * f32::cos(a);
        let dy = r * f32::sin(a);

        Particle::new((WIDTH / 2) as f32 + dx, (HEIGHT / 2) as f32 + dy, a)
    }

    pub fn set_color(&mut self, color: [u8; 3]) {
        self.color = [
            map_range(color[0] as f32, 0.0, std::u8::MAX as f32, 0.0, 1.0),
            map_range(color[1] as f32, 0.0, std::u8::MAX as f32, 0.0, 1.0),
            map_range(color[2] as f32, 0.0, std::u8::MAX as f32, 0.0, 1.0),
        ]
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
        if left_cell.concentration() > center_cell.concentration()
            && left_cell.concentration() > right_cell.concentration()
        {
            self.heading_angle -= self.turn_angle;
        } else if right_cell.concentration() > center_cell.concentration()
            && right_cell.concentration() > left_cell.concentration()
        {
            self.heading_angle += self.turn_angle;
        } else if left_cell.concentration() == right_cell.concentration() {
            match random_f32() > 0.5 {
                true => self.heading_angle -= self.turn_angle,
                false => self.heading_angle += self.turn_angle,
            }
        }

        // 4. Deposit some pheromone at current location
        let curr_cell = grid.cell_at_pt_mut(self.pos);
        curr_cell.deposit(self);

        // 5. Move particle to new destination
        self.pos = cart_to_canvas(move_coords(
            canvas_to_cart(self.pos),
            self.heading_distance,
            self.heading_angle,
        ));

        let width = WIDTH as f32;
        let height = HEIGHT as f32;

        if self.pos.x < 0.0 {
            self.pos.x = width + self.pos.x;
        } else if self.pos.x > width {
            self.pos.x = self.pos.x - width;
        }

        if self.pos.y < 0.0 {
            self.pos.y = height + self.pos.y;
        } else if self.pos.y > height {
            self.pos.y = self.pos.y - height;
        }
    }
}

#[derive(Clone, Debug, Copy)]
struct Cell {
    intensity: [f32; 3],
}

impl Cell {
    pub fn decay(&mut self) {
        for i in 0..3 {
            self.intensity[i] = clamp(self.intensity[i] - DECAY_AMOUNT, 0.0, 1.0);
        }
    }

    pub fn deposit(&mut self, particle: &Particle) {
        for i in 0..3 {
            let deposit_amt = particle.color[i] * DEPOSIT_AMOUNT;
            self.intensity[i] = clamp(self.intensity[i] + deposit_amt, 0.0, 1.0);
        }
    }

    pub fn concentration(&self) -> f32 {
        map_range(
            self.intensity[0] + self.intensity[1] + self.intensity[2],
            0.0,
            3.0,
            0.0,
            1.0,
        )
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
                // TODO
                intensity: [0.0; 3],
            })
            .collect();

        Grid {
            cells,
            width,
            height,
        }
    }

    pub fn cell_at(&self, row: usize, col: usize) -> &Cell {
        let i = (col * self.width) + row;
        &self.cells[i]
    }

    pub fn cell_at_pt(&self, loc: Vector2<f32>) -> &Cell {
        let w = self.width as f32;
        let h = self.height as f32;

        let x = ((w + loc.x) % w) as usize;
        let y = ((h + loc.y) % h) as usize;

        let i = (y * self.width) + x;
        if i > self.cells.len() {
            println!("out of bounds!");
            dbg!(x, y, w, h, i);
        }

        &self.cells[(y * self.width) + x]
    }

    pub fn cell_at_pt_mut(&mut self, loc: Vector2<f32>) -> &mut Cell {
        let w = self.width as f32;
        let h = self.height as f32;

        let x = ((w + loc.x) % w) as usize;
        let y = ((h + loc.y) % h) as usize;

        &mut self.cells[(y * self.width) + x]
    }

    pub fn update(&mut self) {
        self.decay();
        self.blur(BLUR_RADIUS);
    }

    fn decay(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.decay();
        }
    }

    fn blur(&mut self, radius: isize) {
        let mut new_cells = self.cells.to_vec();

        self.blur_horizontal(&mut new_cells, radius);
        self.cells = new_cells;

        new_cells = self.cells.to_vec();

        self.blur_vertical(&mut new_cells, radius);
        self.cells = new_cells;
    }

    fn blur_horizontal(&mut self, dest: &mut Vec<Cell>, radius: isize) {
        for y in 0..self.height as isize {
            for i in 0..3 {
                let mut total: f32 = 0.0;

                for kx in -radius..radius + 1 {
                    total += self.cells[inf_coords(kx, y)].intensity[i];
                }

                // dest[inf_coords(0, y)].intensity[i] = total / ((radius * 2 + 1) as f32);

                dest[inf_coords(0, y)].intensity[i] = Lerp::lerp(
                    dest[inf_coords(0, y)].intensity[i],
                    total / ((radius * 2 + 1) as f32),
                    BLUR_INTENSITY,
                );

                for x in 1..self.width as isize {
                    total -= self.cells[inf_coords(x - radius - 1, y)].intensity[i];
                    total += self.cells[inf_coords(x + radius, y)].intensity[i];

                    dest[inf_coords(x, y)].intensity[i] = Lerp::lerp(
                        dest[inf_coords(x, y)].intensity[i],
                        total / ((radius * 2 + 1) as f32),
                        BLUR_INTENSITY,
                    );
                }
            }
        }
    }

    fn blur_vertical(&mut self, dest: &mut Vec<Cell>, radius: isize) {
        for x in 0..self.width as isize {
            for i in 0..3 {
                let mut total: f32 = 0.0;

                for ky in -radius..radius + 1 {
                    total += self.cells[inf_coords(x, ky)].intensity[i];
                }

                dest[inf_coords(x, 0)].intensity[i] = Lerp::lerp(
                    dest[inf_coords(x, 0)].intensity[i],
                    total / ((radius * 2 + 1) as f32),
                    BLUR_INTENSITY,
                );

                for y in 1..self.height as isize {
                    total -= self.cells[inf_coords(x, y - radius - 1)].intensity[i];
                    total += self.cells[inf_coords(x, y + radius)].intensity[i];

                    dest[inf_coords(x, y)].intensity[i] = Lerp::lerp(
                        dest[inf_coords(x, y)].intensity[i],
                        total / ((radius * 2 + 1) as f32),
                        BLUR_INTENSITY,
                    );
                }
            }
        }
    }

    pub fn draw(&self, app: &App, model: &Model, frame: &Frame, draw: &Draw) {
        let width = self.width as u32;
        let height = self.height as u32;
        let image = nannou::image::ImageBuffer::from_fn(width, height, |x, y| {
            let cell = self.cell_at(x as usize, y as usize);
            let min = MIN_CUTOFF;
            let max = MAX_CUTOFF;

            let r = map_range(
                clamp(cell.intensity[0], min, max),
                min,
                max,
                std::u8::MIN,
                std::u8::MAX,
            );
            let g = map_range(
                clamp(cell.intensity[1], min, max),
                min,
                max,
                std::u8::MIN,
                std::u8::MAX,
            );
            let b = map_range(
                clamp(cell.intensity[2], min, max),
                min,
                max,
                std::u8::MIN,
                std::u8::MAX,
            );

            nannou::image::Rgba([r, g, b, std::u8::MAX])
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
    let particles = (0..NUM_PARTICLES)
        .map(|_| {
            let mut particle = Particle::center(WIDTH as f32 * 0.45);
            let width = WIDTH as f32;
            let height = HEIGHT as f32;
            let x_val = if particle.pos.x < 0.0 {
                width + particle.pos.x
            } else {
                particle.pos.x % width
            };
            let y_val = if particle.pos.y < 0.0 {
                width + particle.pos.y
            } else {
                particle.pos.y % width
            };

            let val = random_f32();
            let max = 1.0;

            particle.set_color([
                map_range(val, 0.0, max, MIN_COLOR[0], MAX_COLOR[0]),
                map_range(val, 0.0, max, MIN_COLOR[1], MAX_COLOR[1]),
                map_range(val, 0.0, max, MIN_COLOR[2], MAX_COLOR[2]),
            ]);

            particle
        })
        .collect();
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
    let window = app
        .window(model._window)
        .expect("Could not get the main window");

    let draw = app.draw();

    draw.background().color(BLACK);

    // Paint the grid!
    model.grid.draw(app, model, &frame, &draw);

    if IMG_OUTPUT {
        let filename = format!("./out/img{:04}.png", app.elapsed_frames());
        window.capture_frame(filename);
    }

    draw.to_frame(app, &frame).unwrap();
}
