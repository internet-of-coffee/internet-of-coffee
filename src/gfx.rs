extern crate sdl2;
//extern crate sdl2_ttf;
extern crate rand;

use self::rand::distributions::{IndependentSample, Range};

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::path::Path;


use gfx::sdl2::image::LoadTexture;
use gfx::sdl2::event::Event;
use gfx::sdl2::keyboard::Keycode;
use gfx::sdl2::rect::Rect;
use gfx::sdl2::render::{TextureQuery, Renderer};
use gfx::sdl2::pixels::Color;
use gfx::sdl2::render::Texture;
use gfx::sdl2::ttf::Font;
//use gfx::sdl2_ttf::Font;

use std::time::{Duration, SystemTime};

use super::LevelConfig;
use super::CoffeeLevel;
use super::TtyReaderAndLogger;

use super::{select_level};

pub struct LevelTextures {
    high: Texture,
    normal: Texture,
    low: Texture,
    texture_label: Texture,

    target_level: Rect,
    target_label: Rect,
}

static SCREEN_WIDTH: u32 = 800;
static SCREEN_HEIGHT: u32 = 480;

macro_rules! rect (
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        println!("Scaling down! The text will look worse!");
        if wr > hr {
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (cons_width as i32 - w) / 2;
    let cy = (cons_height as i32 - h) / 2;
    rect!(cx, cy, w, h)
}

fn init_gfx(font: &mut Font, renderer: &mut Renderer, disp_size: Rect) -> LevelTextures {
    let surface_label = font.render("Level:")
        .blended(Color::RGBA(196, 151, 102, 255)).unwrap();
    let surface_level_high = font.render("HIGH")
        .blended(Color::RGBA(0, 207, 20, 255)).unwrap();
    let surface_level_normal = font.render("NORMAL")
        .blended(Color::RGBA(255, 194, 51, 255)).unwrap();
    let surface_level_low = font.render("LOW")
        .blended(Color::RGBA(255, 43, 26, 255)).unwrap();

    let texture_label = renderer.create_texture_from_surface(&surface_label).unwrap();
    let texture_level_high = renderer.create_texture_from_surface(&surface_level_high).unwrap();
    let texture_level_normal = renderer.create_texture_from_surface(&surface_level_normal).unwrap();
    let texture_level_low = renderer.create_texture_from_surface(&surface_level_low).unwrap();

    let TextureQuery { width, height, .. } = texture_label.query();
    let width_label = width;
    let height_label = height;
    let TextureQuery { width, height, .. } = texture_level_normal.query();
    let width_level = width;
    let height_level = height;

    // If the example text is too big for the screen, downscale it (and center irregardless)
    let padding = 32;
    let mut target_label = get_centered_rect(width_label, height_label, disp_size.width() - padding, disp_size.height() - padding);
    let mut target_level = get_centered_rect(width_level, height_level, disp_size.width() - padding, disp_size.height() - padding);

    let new_y_label = (target_label.y() as f32 - (target_label.height() as f32 / 2f32)) as i32;
    let new_y_level = (target_level.y() as f32 + (target_level.height() as f32 / 2f32)) as i32;
    target_label.set_y(new_y_label);
    target_level.set_y(new_y_level);

    LevelTextures {
        high: texture_level_high,
        normal: texture_level_normal,
        low: texture_level_low,
        texture_label: texture_label,

        target_label: target_label,
        target_level: target_level,
    }
}

fn select_tex_for_level(level: CoffeeLevel, tex_levels: &LevelTextures) -> &Texture {
    match level {
        CoffeeLevel::HIGH => &tex_levels.high,
        CoffeeLevel::NORMAL => &tex_levels.normal,
        CoffeeLevel::LOW => &tex_levels.low,
    }
}


#[derive(Copy, Clone)]
struct Flake<'a> {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: usize,
    tex: &'a Texture,
}

impl<'a> Flake<'a> {
    fn anim(&mut self, delta_ms: f32) {
        self.x += self.vx * delta_ms;
        self.y += self.vy * delta_ms;

        if self.y > SCREEN_HEIGHT as f32 || self.x < -64.0 || self.x > SCREEN_WIDTH as f32 + 64.0 {
            let between = Range::new(0f32, 1.);
            let mut rng = rand::thread_rng();
            self.y = -64.;
            self.x = between.ind_sample(&mut rng) * SCREEN_WIDTH as f32;
            self.x -= 32.;
            self.size = (FLAKE_SIZE as f32 + between.ind_sample(&mut rng) * 16.) as usize;
        }
     }
}

const NUM_FLAKES: usize = 50;
const FLAKE_SIZE: usize = 4;
struct RenderCtx<'a> {
    renderer: Renderer<'a>,
    level_config: LevelConfig,
    tex_levels: LevelTextures,
    font_percent: Font<'a>,
    disp_size: Rect,
    flakes: [Flake<'a>; NUM_FLAKES],
}
const MIN_DOWN_SPEED: f32 = 0.05;
impl<'a> RenderCtx<'a> {
    fn init_flakes(&mut self) {
        let between = Range::new(0f32, 1.);
        let mut rng = rand::thread_rng();
        for i in 0..self.flakes.len() {
            let mut f = &mut self.flakes[i];
            f.y = between.ind_sample(&mut rng) * SCREEN_HEIGHT as f32;
            f.y -= FLAKE_SIZE as f32;
            f.x = between.ind_sample(&mut rng) * SCREEN_WIDTH as f32;
            f.x -= FLAKE_SIZE as f32 / 2.;
            f.vx = (between.ind_sample(&mut rng) - 0.5) * 0.2;
            let vy = between.ind_sample(&mut rng) * 0.2;
            f.vy = if vy > MIN_DOWN_SPEED {vy} else { MIN_DOWN_SPEED };
            f.size = (FLAKE_SIZE as f32 + between.ind_sample(&mut rng) * 16.) as usize;
        };
    }

    fn draw_flakes(&mut self, delta_ms: f32) {
        for i in 0..self.flakes.len() {
            let f: &mut Flake = &mut self.flakes[i];
            f.anim(delta_ms);
            let flake_rect = rect!(f.x, f.y, f.size, f.size);
            let _ = self.renderer.copy(&f.tex, None, Some(flake_rect));
        }
    }

    fn render(&mut self, delta_ms: f32, weight: u32) {
        self.renderer.set_draw_color(Color::RGBA(102, 58, 23, 255)); // Brown
        self.renderer.clear();

        self.draw_flakes(delta_ms);

        let mut tex_level = select_tex_for_level(select_level(weight, &self.level_config), &self.tex_levels);
        let _ = self.renderer.copy(&self.tex_levels.texture_label, None, Some(self.tex_levels.target_label));
        let _ = self.renderer.copy(&mut tex_level, None, Some(self.tex_levels.target_level));

        let corrected_weight = if weight < self.level_config.min { self.level_config.min } else { weight };
        let coffee_ratio = (corrected_weight - self.level_config.min) as f32 / (self.level_config.max - self.level_config.min) as f32;
        let coffee_percent = if coffee_ratio < 0f32 { 0f32 } else { coffee_ratio * 100f32 };
        let surface_coffee_percent = self.font_percent.render(format!("{:.2}% kaffe", coffee_percent).as_str())
            .blended(Color::RGBA(255, 255, 255, 255)).unwrap();
        let coffee_tex = self.renderer.create_texture_from_surface(&surface_coffee_percent).unwrap();
        let TextureQuery { width, height, .. } = coffee_tex.query();
        let coffe_tex_rect = rect!(self.disp_size.width() - 32 - width, self.disp_size.height() - 32 - height, width, height);
        let _ = self.renderer.copy(&coffee_tex, None, Some(coffe_tex_rect));

        let surface_weight = self.font_percent.render(format!("Weight: {} g", weight).as_str()).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
        let weight_tex = self.renderer.create_texture_from_surface(&surface_weight).unwrap();
        let TextureQuery { width, height, .. } = weight_tex.query();

        let weight_tex_rect = rect!(32, self.disp_size.height() - 32 - height, width, height);
        let _ = self.renderer.copy(&weight_tex, None, Some(weight_tex_rect));


        self.renderer.present();
    }
}

pub fn run(font_path: &Path, reader_and_logger: TtyReaderAndLogger) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    //let disp_size = video_subsys.display_bounds(0).ok().expect("Could not read size of display 0");
    let disp_size = Rect::new(0i32, 0i32, SCREEN_WIDTH, SCREEN_HEIGHT);
    let window = video_subsys.window("internet-of-coffee", disp_size.width(), disp_size.height())
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    renderer.set_draw_color(Color::RGBA(102, 58, 23, 255)); // Brown
    renderer.clear();
    renderer.present();

    // Load a font
    let mut font = ttf_context.load_font(font_path, 128).unwrap();
    let mut font_percent = ttf_context.load_font(font_path, 32).unwrap();
    font.set_style(sdl2::ttf::STYLE_BOLD);
    font_percent.set_style(sdl2::ttf::STYLE_BOLD);

    let tex_levels = init_gfx(&mut font, &mut renderer, disp_size);


    let (weight_tx, weight_rx) = mpsc::channel();
    let mut previous_weight = 0;

    let flake_tex = renderer.load_texture(Path::new("./gfx/flake64.png")).unwrap();
    let mut render_ctx = RenderCtx {
        level_config: reader_and_logger.level_config.clone(),
        disp_size: disp_size,
        font_percent: font_percent,
        renderer: renderer,
        tex_levels: tex_levels,
        flakes: [Flake { x: 0.0, y: 0.0, vx: 0.0, vy: 0.1, size: 4, tex: &flake_tex }; NUM_FLAKES],
    };

    let reader_arc = Arc::new(Mutex::new(reader_and_logger));
    let _ = thread::spawn(move || {
        loop {
            let mut reader = reader_arc.lock().unwrap();
            match reader.read_and_log() {
                Some(weight) => {
                    previous_weight = weight;
                    let _ = weight_tx.send(weight);
                },
                _ => (),
            }
        }
    });

    render_ctx.init_flakes();

    let frame_rate = 60f32;
    let max_frame_time = 1000f32 / frame_rate;
    let mut frame_time = 0f32;
    let mut start_time;

    'mainloop: loop {
        start_time = SystemTime::now();

        let remaining_time = match max_frame_time - frame_time {
            v if v > 0f32 => v as u32,
            _ => 0,
        };
        let weight = match weight_rx.recv_timeout(Duration::new(0, remaining_time)) {
            Ok(weight) => {
                previous_weight = weight;
                weight
            }
            _ => previous_weight,
        };
        render_ctx.render(frame_time, weight);

        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'mainloop,
                _ => {
                    println!("{:?}", event);
                }
            }
        }

        match start_time.elapsed() {
            Ok(elapsed) => {
                frame_time = elapsed.subsec_nanos() as f32 / 1000_000f32;
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
