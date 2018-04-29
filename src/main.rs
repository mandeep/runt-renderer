#![allow(dead_code)]
extern crate image;
extern crate nalgebra;

use std::fs::File;
use std::mem::swap;
use nalgebra::geometry::Point2;
use nalgebra::core::Vector3;

mod wavefront;


/// Bresenham's algorithm: Draw a line in the given color from (x0, y0) to (x1, y1)
///
/// # Examples
///
/// ```
/// let mut buffer = image::ImageBuffer::new(1921, 1081);
///
/// line(0, 0, 1920, 1080, &mut buffer, image::Rgb([255, 255, 255]))
/// ```
///
/// ```
/// let mut buffer = image::ImageBuffer::new(512, 512);
///
/// line(0, 0, 511, 511, &mut buffer, image::Rgb([128, 0, 255]))
/// ```
fn draw_line(mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, buffer: &mut image::RgbImage, color: image::Rgb<u8>) {
    let steep = (x0 - x1).abs() < (y0 - y1).abs();
    
    if steep {
        swap(&mut x0, &mut y0);
        swap(&mut x1, &mut y1);
    }
    
    if x0 > x1 {
        swap(&mut x0, &mut x1);
        swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut y = y0;
    let derror = (dy * 2).abs();
    let mut error = 0;

    for x in x0 ..= x1 {
        if steep {
            buffer.put_pixel(y as u32, x as u32, color);
        } else {
            buffer.put_pixel(x as u32, y as u32, color);
        }
        error += derror;
        if error > dx {
            if y1 > y0 {
                y += 1;
            } else {
                y -= 1;
            }
            error -= dx * 2;
        }
    }
}

fn find_barycentric(points: &Vec<Point2<f64>>, point: Point2<f64>) -> Vector3<f64> {
    let u = Vector3::new(points[2].x - points[0].x, points[1].x - points[0].x, points[0].x - point.x);
    let v = Vector3::new(points[2].y - points[0].y, points[1].y - points[0].y, points[0].y - point.y);

    let w = u.cross(&v);

    if (w.z).abs() < 1.0 {
        return Vector3::new(-1.0, 1.0, 1.0);
    } else {
        return Vector3::new(1.0 - (w.x + w.y) as f64 / w.z as f64, w.y as f64 / w.z as f64, w.x as f64 / w.z as f64);
    }

}

fn draw_triangle(points: &Vec<Point2<f64>>, buffer: &mut image::RgbImage, color: image::Rgb<u8>) {
    let mut bounding_box_minimum: Point2<f64> = Point2::new(buffer.width() as f64 - 1.0, buffer.height() as f64 - 1.0);
    let mut bounding_box_maximum: Point2<f64> = Point2::new(0.0, 0.0);

    for point in points {
        bounding_box_minimum.x = bounding_box_minimum.x.min(point.x);
        bounding_box_minimum.y = bounding_box_minimum.y.min(point.y);
        bounding_box_maximum.x = bounding_box_maximum.x.max(point.x);
        bounding_box_maximum.y = bounding_box_maximum.y.max(point.y);
    }

    for x in bounding_box_minimum.x as i32 ..= bounding_box_maximum.x as i32 {
        for y in bounding_box_minimum.y as i32 ..= bounding_box_maximum.y as i32 {
            let point = Point2::new(x as f64, y as f64);
            let barycentric_coordinates: Vector3<f64> = find_barycentric(points, point);
            if barycentric_coordinates.x >= 0.0 && barycentric_coordinates.y >= 0.0 && barycentric_coordinates.z >= 0.0 {
                buffer.put_pixel(point.x as u32, point.y as u32, color);
            }
        }
    }
}

/// Draw a wire mesh on the given ImageBuffer with the coordinates from the given file
///
/// # Examples
///
/// ```
/// let width = 512;
/// let height = 512;
/// let mut buffer = image::ImageBuffer::new(width, height);
///
/// draw_wire_mesh("coordinates.obj", &mut buffer, width, height); 
/// ```
fn draw_wire_mesh(filename: &str, buffer: &mut image::RgbImage, width: u32, height: u32) {
    let coordinates = wavefront::Object::new(filename);

    for face in coordinates.faces {
        for i in 0..3 {
            let v0 = coordinates.vertices[(face[i] - 1) as usize];
            let v1 = coordinates.vertices[(face[(i+1) % 3] - 1) as usize];
            
            let x0 = ((v0.x + 1.0) * width as f64 / 2.0).min(width as f64 - 1.0);
            let y0 = ((v0.y + 1.0) * height as f64 / 2.0).min(height as f64 - 1.0);
            
            let x1 = ((v1.x + 1.0) * width as f64 / 2.0).min(width as f64 - 1.0);
            let y1 = ((v1.y + 1.0) * height as f64 / 2.0).min(height as f64 - 1.0);
            
            draw_line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, buffer, image::Rgb([255, 255, 255]));
        }
    }
}

fn draw_triangle_mesh(filename: &str, buffer: &mut image::RgbImage, light_vector: Vector3<f64>) {
    let coordinates = wavefront::Object::new(filename);

    for face in coordinates.faces {
        let mut screen_coordinates: Vec<Point2<f64>> = Vec::new();
        let mut world_coordinates: Vec<Vector3<f64>> = Vec::new();
        for i in 0..3 {
            let world_coordinate: Vector3<f64> = coordinates.vertices[(face[i] - 1) as usize];

            let pointx = ((world_coordinate.x + 1.0) * buffer.width() as f64 / 2.0).min(buffer.width() as f64 - 1.0);
            let pointy = ((world_coordinate.y + 1.0) * buffer.height() as f64 / 2.0).min(buffer.height() as f64 - 1.0);
            
            screen_coordinates.push(Point2::new(pointx, pointy));
            world_coordinates.push(world_coordinate);
        }
        let normal: Vector3<f64> = (world_coordinates[2] - world_coordinates[0]).cross(&(world_coordinates[1] - world_coordinates[0])).normalize();
        let intensity = normal.dot(&light_vector);

        if intensity > 0.0 {
            draw_triangle(&screen_coordinates, buffer, image::Rgb([(255.0 * intensity) as u8, (255.0 * intensity) as u8, (255.0 * intensity) as u8]));
        }
    }
}

fn main() {
    let (width, height) = (1600, 1600);
    let mut buffer = image::ImageBuffer::new(width, height);

    let light_vector = Vector3::new(0.0, 0.0, -1.0).normalize();
    draw_triangle_mesh("../porsche.obj", &mut buffer, light_vector);

    let ref mut fout = File::create("../triangle_mesh.png").unwrap();
    image::ImageRgb8(buffer).flipv()
                            .save(fout, image::PNG)
                            .unwrap();
}
