/// Demo on how to transform glyphs to vertices

#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
use glium::{glutin, Surface};

use lyon::path::Path;
use rusttype::{Font, PositionedGlyph};

fn main() {
    let text = "Hello world";
    // let font_bytes = include_bytes!("Archicoco.ttf");
    // let font_bytes = include_bytes!("Orkney Bold.ttf");
    // let font_bytes = include_bytes!("Carlito-Regular.ttf");
    let font_bytes = include_bytes!("KatamotzIkasi.ttf");

    let font = Font::from_bytes(font_bytes as &[u8]).expect("Font must be valid ttf data!");

    let geometry = text_to_vertices(text, &font);

    // Construct opengl window
    let mut event_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;

                in vec2 position;
                in vec3 color;

                out vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;

                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        },
    )
    .unwrap();

    let vertex_buffer = { glium::VertexBuffer::new(&display, &geometry.vertices).unwrap() };
    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &geometry.indices).unwrap();

    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = move || {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [0.05, 0.0, 0.0, -0.5],
                [0.0, 0.05, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    };

    // Draw the triangle to the screen.
    draw();

    // the main loop
    event_loop.run_forever(move |event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::WindowEvent::CloseRequested => glutin::ControlFlow::Break,
                // Redraw the triangle when the window is resized.
                glutin::WindowEvent::Resized(..) => {
                    draw();
                    glutin::ControlFlow::Continue
                }
                _ => glutin::ControlFlow::Continue,
            },
            _ => glutin::ControlFlow::Continue,
        }
    });
}

fn text_to_vertices(text: &str, font: &Font) -> lyon::tessellation::VertexBuffers<Vertex, u16> {
    let glyphs: Vec<PositionedGlyph> = font
        .layout(
            text,
            rusttype::Scale::uniform(8.0),
            rusttype::point(0.0, 0.0),
        )
        .map(|g| g.standalone())
        .collect();

    println!("Got {} glyphs", glyphs.len());

    // Paths to vertices!
    let mut geometry: lyon::tessellation::VertexBuffers<Vertex, u16> =
        lyon::tessellation::VertexBuffers::new();

    let mut tessellator = lyon::tessellation::FillTessellator::new();

    for glyph in glyphs {
        let s = glyph.shape();

        if let Some(shape) = s {
            println!("Glyph with {} contours in shape!", shape.len());
            let path = contours_to_path(shape);
            println!("Path: {:?}", path);

            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &lyon::tessellation::FillOptions::default(),
                    &mut lyon::tessellation::BuffersBuilder::new(
                        &mut geometry,
                        |vertex: lyon::tessellation::FillVertex| Vertex {
                            position: vertex.position.to_array(),
                            color: [0.0, 1.0, 0.0],
                        },
                    ),
                )
                .unwrap();
        } else {
            println!("No shape!");
        }
    }

    println!("Got {} vertices", geometry.vertices.len());
    geometry
}

/// Convert a rusttype contours into a lyon path
fn contours_to_path(contours: Vec<rusttype::Contour>) -> lyon::path::Path {
    // Start a path construction:
    let mut path_builder = Path::builder();

    for contour in contours {
        println!("Contour with {} segments", contour.segments.len());
        contour_to_path(&mut path_builder, contour);
    }

    path_builder.build()
}

fn contour_to_path(path_builder: &mut lyon::path::Builder, contour: rusttype::Contour) {
    let start_point = match contour.segments.first().unwrap() {
        rusttype::Segment::Line(line) => p2p(line.p[0]),
        rusttype::Segment::Curve(curve) => p2p(curve.p[0]),
    };

    path_builder.move_to(start_point);

    for segment in contour.segments {
        println!("Segment: {:?}", segment);

        // convert to tesselator!
        match segment {
            rusttype::Segment::Line(line) => {
                path_builder.line_to(p2p(line.p[1]));
            }
            rusttype::Segment::Curve(curve) => {
                path_builder.quadratic_bezier_to(p2p(curve.p[1]), p2p(curve.p[2]))
            }
        }
    }

    path_builder.close();
}

/// Convert rusttype point into lyon point
fn p2p(p: rusttype::Point<f32>) -> lyon::math::Point {
    lyon::math::point(p.x, p.y)
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);
