use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

const WIDTH: usize = 512;
const HEIGHT: usize = 256;

struct Image {
    lines: [[[u8; 3]; WIDTH]; HEIGHT],
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ImageCoord {
    x: usize,
    y: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct ScreenCoord {
    x: f64,
    y: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct NormalizedScreenCoord {
    x: f64,
    y: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct NormalizedScreenColor {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct ScreenColor {
    r: f64,
    g: f64,
    b: f64,
}

fn main() {
    let mut image = Image {
        lines: [[[0, 0, 0]; WIDTH]; HEIGHT],
    };
    draw_triangle(
        &mut image,
        NormalizedScreenCoord { x: 0.0, y: -0.5 },
        NormalizedScreenCoord { x: 0.5, y: 0.0 },
        NormalizedScreenCoord { x: -0.5, y: 0.5 },
        NormalizedScreenColor {
            r: 1.0,
            b: 1.0,
            g: 1.0,
        },
    );
    write_ppm(&image, "output");
}

fn write_ppm(image: &Image, filename: &str) {
    let path = Path::new(filename);
    let handle_io_error = |why: &dyn Error| {
        panic!(
            "Couldn't write to {}: {}",
            path.display(),
            why.description()
        )
    };
    let file = match File::create(&path) {
        Err(why) => panic!("Couldn't create {}: {}", path.display(), why.description()),
        Ok(file) => file,
    };
    let mut stream = BufWriter::new(file);
    println!("Writing image to file {}.", filename);
    match stream.write(format!("P6\n{} {}\n255\n", WIDTH, HEIGHT).as_bytes()) {
        Err(why) => handle_io_error(&why),
        Ok(_) => {}
    }
    image.lines.iter().for_each(|line| {
        line.iter().for_each(|pixel| match stream.write(pixel) {
            Err(why) => handle_io_error(&why),
            Ok(_) => {}
        })
    });
    match stream.flush() {
        Err(why) => handle_io_error(&why),
        Ok(_) => println!("Successfully wrote to {}.", path.display()),
    };
}

fn image_to_screen_coord(image_coord: ImageCoord) -> ScreenCoord {
    ScreenCoord {
        x: image_coord.x as f64,
        y: image_coord.y as f64,
    }
}

fn get_line_eq(a: ScreenCoord, b: ScreenCoord) -> impl Fn(ScreenCoord) -> f64 {
    let t = NormalizedScreenCoord {
        x: b.x - a.x,
        y: b.y - a.y,
    };
    let t_norm = (t.x * t.x + t.y * t.y).sqrt();
    let n = NormalizedScreenCoord {
        x: -t.y / t_norm,
        y: t.x / t_norm,
    };
    move |u| (u.x - a.x) * n.x + (u.y - a.y) * n.y
}

fn denormalize_screen_coord(normalized_coord: NormalizedScreenCoord) -> ScreenCoord {
    ScreenCoord {
        x: ((normalized_coord.x + 1.0) * (WIDTH - 1) as f64 / 2.0),
        y: ((normalized_coord.y + 1.0) * (HEIGHT - 1) as f64 / 2.0),
    }
}

fn denormalize_screen_color(normalized_color: NormalizedScreenColor) -> ScreenColor {
    ScreenColor {
        r: normalized_color.r * 255.0,
        g: normalized_color.g * 255.0,
        b: normalized_color.b * 255.0,
    }
}

fn get_triangle_eq(a: ScreenCoord, b: ScreenCoord, c: ScreenCoord) -> impl Fn(ScreenCoord) -> f64 {
    let ab_eq = get_line_eq(a, b);
    let bc_eq = get_line_eq(b, c);
    let ca_eq = get_line_eq(c, a);
    move |u| ab_eq(u).min(bc_eq(u)).min(ca_eq(u))
}

fn draw_triangle(
    image: &mut Image,
    a: NormalizedScreenCoord,
    b: NormalizedScreenCoord,
    c: NormalizedScreenCoord,
    color: NormalizedScreenColor,
) {
    let a_denorm = denormalize_screen_coord(a);
    let b_denorm = denormalize_screen_coord(b);
    let c_denorm = denormalize_screen_coord(c);
    let color_denorm = denormalize_screen_color(color);
    let triangle_eq = get_triangle_eq(a_denorm, b_denorm, c_denorm);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let screen_coord = image_to_screen_coord(ImageCoord { x, y });
            let intensity = (triangle_eq(screen_coord) + 0.5).max(0.0).min(1.0);
            if intensity > 0.0 {
                image.lines[y][x] = [
                    (intensity * color_denorm.r) as u8,
                    (intensity * color_denorm.g) as u8,
                    (intensity * color_denorm.b) as u8,
                ];
            }
        }
    }
}
