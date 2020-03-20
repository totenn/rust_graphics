use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

const WIDTH: usize = 203;
const HEIGHT: usize = 203;

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

fn main() {
    let mut image = Image {
        lines: [[[0, 0, 0]; WIDTH]; HEIGHT],
    };
    draw_triangle(ScreenCoord { x: 0.0, y: -0.5 }, ScreenCoord { x: 0.5, y: 0.0 }, ScreenCoord { x: -0.5, y: 0.5 }, &mut image);
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

fn screen_to_image_coord(screen_coord: ScreenCoord) -> ImageCoord {
    ImageCoord {
        x: ((screen_coord.x + 1.0) * (WIDTH - 1) as f64 / 2.0) as usize,
        y: ((screen_coord.y + 1.0) * (HEIGHT - 1) as f64 / 2.0) as usize,
    }
}

fn image_to_screen_coord(image_coord: ImageCoord) -> ScreenCoord {
    ScreenCoord {
        x: image_coord.x as f64 * 2.0 / (WIDTH - 1) as f64 - 1.0,
        y: image_coord.y as f64 * 2.0 / (HEIGHT - 1) as f64 - 1.0,
    }
}

fn draw_point(screen_coord: ScreenCoord, image: &mut Image) {
    let image_coord = screen_to_image_coord(screen_coord);
    image.lines[image_coord.y][image_coord.x] = [255, 255, 255];
}

fn get_line_eq(a: ScreenCoord, b: ScreenCoord) -> impl Fn(ScreenCoord) -> f64 {
    let t = ScreenCoord {
        x: b.x - a.x,
        y: b.y - a.y,
    };
    let t_norm = (t.x*t.x + t.y*t.y).sqrt();
    let n = ScreenCoord { x: -t.y/t_norm , y: t.x/t_norm };
    move |u| (u.x - a.x) * n.x + (u.y - a.y) * n.y
}

fn draw_half_space(a: ScreenCoord, b: ScreenCoord, image: &mut Image) {
    let pixel_width = 1.0 / ((HEIGHT * HEIGHT + WIDTH * WIDTH) as f64).sqrt();
    let line_eq = get_line_eq(a, b);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let screen_coord = image_to_screen_coord(ImageCoord { x, y });
            let line_sign = line_eq(screen_coord);
            if line_sign < 0.0 - pixel_width {
                image.lines[y][x] = [255, 255, 255];
            } else if line_sign < 0.0 + pixel_width {
                let intensity = 128 + (line_sign * 128.0) as u8;
                image.lines[y][x] = [intensity, intensity, intensity];
            }
        }
    }
}

fn draw_triangle(a: ScreenCoord, b: ScreenCoord, c: ScreenCoord, image: &mut Image) {
    let pixel_width = 1.0 / ((HEIGHT * HEIGHT + WIDTH * WIDTH) as f64).sqrt();
    let ab_eq = get_line_eq(a, b);
    let bc_eq = get_line_eq(b, c);
    let ca_eq = get_line_eq(c, a);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let screen_coord = image_to_screen_coord(ImageCoord { x, y });
            let ab_sign = ab_eq(screen_coord);
            let bc_sign = bc_eq(screen_coord);
            let ca_sign = ca_eq(screen_coord);
            let triangle_sign = ab_sign.min(bc_sign).min(ca_sign);
            if triangle_sign < -pixel_width {
                image.lines[y][x] = [255, 255, 255];
            } else if triangle_sign < 0.0 + pixel_width {
                let intensity = 128 + (triangle_sign * 128.0) as u8;
                image.lines[y][x] = [intensity, intensity, intensity];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_to_image_coord() {
        assert_eq!(
            screen_to_image_coord(ScreenCoord { x: 1.0, y: 1.0 }),
            ImageCoord {
                x: WIDTH - 1,
                y: HEIGHT - 1
            }
        );
        assert_eq!(
            screen_to_image_coord(ScreenCoord { x: -1.0, y: -1.0 }),
            ImageCoord { x: 0, y: 0 }
        );
        assert_eq!(
            screen_to_image_coord(ScreenCoord { x: 0.0, y: 0.0 }),
            ImageCoord {
                x: (WIDTH - 1) / 2,
                y: (HEIGHT - 1) / 2
            }
        );
    }

    #[test]
    fn test_image_to_screen_coord() {
        assert_eq!(
            image_to_screen_coord(ImageCoord {
                x: WIDTH - 1,
                y: HEIGHT - 1
            }),
            ScreenCoord { x: 1.0, y: 1.0 }
        );
        assert_eq!(
            image_to_screen_coord(ImageCoord { x: 0, y: 0 }),
            ScreenCoord { x: -1.0, y: -1.0 }
        );
    }
}
