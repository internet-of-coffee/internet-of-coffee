extern crate sdl2;
extern crate sdl2_ttf;

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::path::Path;
//use std::fs::File;

use gfx::sdl2::event::Event;
use gfx::sdl2::keyboard::Keycode;
use gfx::sdl2::rect::Rect;
use gfx::sdl2::render::TextureQuery;
use gfx::sdl2::render::Renderer;
use gfx::sdl2::pixels::Color;
use gfx::sdl2::render::Texture;
use gfx::sdl2_ttf::Font;

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

struct RenderCtx<'a> {
    renderer: Renderer<'a>,
    level_config: LevelConfig,
    tex_levels: LevelTextures,
    font_percent: Font,
    disp_size: Rect
}

impl<'a> RenderCtx<'a> {
    fn render(&mut self, weight: u32) {
        self.renderer.set_draw_color(Color::RGBA(102, 58, 23, 255)); // Brown
        self.renderer.clear();

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

//struct LoggingCtx {
//    tty_usb: File,
//    log_file: File,
//    level_config: LevelConfig,
//}

pub fn run(font_path: &Path, reader_and_logger: TtyReaderAndLogger) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();

    //let disp_size = video_subsys.display_bounds(0).ok().expect("Could not read size of display 0");
    let disp_size = Rect::new(0i32, 0i32, SCREEN_WIDTH, SCREEN_HEIGHT);
    let window = video_subsys.window("SDL2_TTF Example", disp_size.width(), disp_size.height())
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
    font.set_style(sdl2_ttf::STYLE_BOLD);
    font_percent.set_style(sdl2_ttf::STYLE_BOLD);

    let tex_levels = init_gfx(&mut font, &mut renderer, disp_size);


    let (weight_tx, weight_rx) = mpsc::channel();
    let mut previous_weight = 0;

    let mut render_ctx = RenderCtx {
        level_config: reader_and_logger.level_config.clone(),
        disp_size: disp_size,
        font_percent: font_percent,
        renderer: renderer,
        tex_levels: tex_levels,
    };

    let reader_arc = Arc::new(Mutex::new(reader_and_logger));
    let reader_clone = reader_arc.clone();
    let _ = thread::spawn(move || {
        loop {
            let mut reader = reader_clone.lock().unwrap();
            match reader.read_and_log() {
                Some(weight) => {
                    previous_weight = weight;
                    let _ = weight_tx.send(weight);
                },
                _ => (),
            }
        }
    });

    let frame_rate = 60f32;
    let max_frame_time = 1000f32 / frame_rate;
    let mut frame_time = 0f32;
    let mut start_time;

    'mainloop: loop {
        start_time = SystemTime::now();

        // factor this into a separate thread/future/concept for concurrency of the day
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
//        previous_weight = weight;
        render_ctx.render(weight);

        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'mainloop,
                _ => {}
            }
        }

        match start_time.elapsed() {
            Ok(elapsed) => {
                frame_time = elapsed.subsec_nanos() as f32 / 1000_000f32;
            }
            Err(e) => {
                // an error occured!
                println!("Error: {:?}", e);
            }
        }
    }
}
