extern crate sdl2;
extern crate sdl2_ttf;

use std::path::Path;
use std::fs::File;

use gfx::sdl2::event::Event;
use gfx::sdl2::keyboard::Keycode;
use gfx::sdl2::rect::Rect;
use gfx::sdl2::render::TextureQuery;
use gfx::sdl2::render::Renderer;
use gfx::sdl2::pixels::Color;
use gfx::sdl2::render::Texture;
use gfx::sdl2_ttf::Font;

use super::LevelConfig;
use super::CoffeeLevel;

use super::{select_level, read_and_log};

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

    let mut texture_label = renderer.create_texture_from_surface(&surface_label).unwrap();
    let mut texture_level_high = renderer.create_texture_from_surface(&surface_level_high).unwrap();
    let mut texture_level_normal = renderer.create_texture_from_surface(&surface_level_normal).unwrap();
    let mut texture_level_low = renderer.create_texture_from_surface(&surface_level_low).unwrap();

    let TextureQuery { width, height, .. } = texture_label.query();
    let mut width_label = width;
    let mut height_label = height;
    let TextureQuery { width, height, .. } = texture_level_normal.query();
    let mut width_level = width;
    let mut height_level = height;

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

pub fn run(font_path: &Path, level_config: &LevelConfig, tty_usb: &mut File, mut log_file: &mut File) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();

    let disp_size = video_subsys.display_bounds(0).ok().expect("Could not read size of display 0");
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

    let mut tex_levels = init_gfx(&mut font, &mut renderer, disp_size);

    'mainloop: loop {
        // factor this into a separate thread/future/concept for concurrency of the day
        match read_and_log(tty_usb, &mut log_file, level_config) {
            Some(weight) => {
                renderer.set_draw_color(Color::RGBA(102, 58, 23, 255)); // Brown
                renderer.clear();

                let mut tex_level = select_tex_for_level(select_level(weight, level_config), &tex_levels);
                renderer.copy(&tex_levels.texture_label, None, Some(tex_levels.target_label));
                renderer.copy(&mut tex_level, None, Some(tex_levels.target_level));


                let corrected_weight = if weight < level_config.min { level_config.min } else { weight };
                let mut coffee_ratio = (corrected_weight - level_config.min) as f32 / (level_config.max - level_config.min) as f32;
                let coffee_percent = if coffee_ratio < 0f32 { 0f32 } else { coffee_ratio * 100f32 };
                let surface_coffee_percent = font_percent.render(format!("{}% kaffe", coffee_percent).as_str())
                    .blended(Color::RGBA(255, 255, 255, 255)).unwrap();
                let mut coffee_tex = renderer.create_texture_from_surface(&surface_coffee_percent).unwrap();
                let TextureQuery { width, height, .. } = coffee_tex.query();
                let coffe_tex_rect = rect!(disp_size.width() - 32 - width, disp_size.height() - 32 - height, width, height);
                renderer.copy(&coffee_tex, None, Some(coffe_tex_rect));

                renderer.present();
            },
            None => {},
        }

        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'mainloop,
                _ => {}
            }
        }
    }
}

