use crate::*;
use futures::executor::block_on;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn reference_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("screenshots").join("reference")
}

fn output_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("screenshots").join("test_output");
    std::fs::create_dir_all(&dir).ok();
    dir
}

struct TestRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    vger: Vger,
}

impl TestRenderer {
    fn new() -> Self {
        let (device, queue) = block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, None)
                .await
                .expect("No suitable GPU adapters found!");
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::default(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .expect("Unable to find a suitable GPU adapter!")
        });
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let vger = Vger::new(device.clone(), queue.clone(), TEXTURE_FORMAT);
        Self { device, queue, vger }
    }

    /// Render a view to RGBA pixel data.
    fn render_view(&mut self, view: &impl View, width: u32, height: u32) -> Vec<u8> {
        let mut cx = Context::new();
        let sz: LocalSize = [width as f32, height as f32].into();

        self.vger.begin(width as f32, height as f32, 1.0);

        let mut path = vec![0];
        let view_sz = view.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |s, size, max_width| self.vger.text_bounds(s, size, max_width),
            },
        );

        // Center the view like Context::render does.
        let offset: LocalOffset = ((sz - view_sz) / 2.0).into();
        self.vger.translate(offset);

        view.draw(&mut path, &mut DrawArgs { cx: &mut cx, vger: &mut self.vger });

        // Create offscreen texture.
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture_desc = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            label: Some("test_render_texture"),
            view_formats: &[TEXTURE_FORMAT],
        };
        let render_texture = self.device.create_texture(&texture_desc);
        let texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let desc = wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..<_>::default()
        };

        self.vger.encode(&desc);

        // Read back pixels.
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let padded_bytes_per_row = {
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            (unpadded_bytes_per_row + align - 1) / align * align
        };

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            render_texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            texture_size,
        );
        self.queue.submit(Some(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);

        // Map and read.
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        // Remove row padding if needed.
        if padded_bytes_per_row == unpadded_bytes_per_row {
            data.to_vec()
        } else {
            let mut pixels = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                pixels.extend_from_slice(&data[start..end]);
            }
            pixels
        }
    }
}

fn save_png(path: &Path, pixels: &[u8], width: u32, height: u32) {
    let file = std::fs::File::create(path).unwrap();
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(pixels).unwrap();
}

fn load_png(path: &Path) -> (Vec<u8>, u32, u32) {
    let file = std::fs::File::open(path).unwrap();
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf.truncate(info.buffer_size());
    (buf, info.width, info.height)
}

/// Compare two RGBA pixel buffers, returning the fraction of pixels that differ
/// beyond a per-channel threshold.
fn pixel_diff_fraction(a: &[u8], b: &[u8], threshold: u8) -> f64 {
    assert_eq!(a.len(), b.len(), "pixel buffers must be same size");
    let pixel_count = a.len() / 4;
    let mut diff_count = 0usize;
    for i in 0..pixel_count {
        let base = i * 4;
        let differs = (0..4).any(|c| {
            let av = a[base + c] as i16;
            let bv = b[base + c] as i16;
            (av - bv).unsigned_abs() > threshold as u16
        });
        if differs {
            diff_count += 1;
        }
    }
    diff_count as f64 / pixel_count as f64
}

/// Render a view at custom dimensions, compare against reference PNG.
fn screenshot_test_sized(
    renderer: &mut TestRenderer,
    view: &impl View,
    name: &str,
    width: u32,
    height: u32,
) {
    let pixels = renderer.render_view(view, width, height);

    let reference_path = reference_dir().join(format!("{name}.png"));
    let output_path = output_dir().join(format!("{name}.png"));

    save_png(&output_path, &pixels, width, height);

    if !reference_path.exists() {
        std::fs::create_dir_all(reference_path.parent().unwrap()).ok();
        save_png(&reference_path, &pixels, width, height);
        eprintln!("Created reference screenshot: {}", reference_path.display());
        return;
    }

    let (ref_pixels, ref_w, ref_h) = load_png(&reference_path);
    assert_eq!(ref_w, width, "reference width mismatch for {name}");
    assert_eq!(ref_h, height, "reference height mismatch for {name}");

    let threshold = 2u8;
    let max_diff_fraction = 0.001;
    let diff = pixel_diff_fraction(&pixels, &ref_pixels, threshold);

    if diff > max_diff_fraction {
        let diff_path = output_dir().join(format!("{name}_diff.png"));
        let diff_pixels: Vec<u8> = pixels
            .chunks(4)
            .zip(ref_pixels.chunks(4))
            .flat_map(|(a, b)| {
                let differs = (0..4).any(|c| {
                    (a[c] as i16 - b[c] as i16).unsigned_abs() > threshold as u16
                });
                if differs {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 0, 255]
                }
            })
            .collect();
        save_png(&diff_path, &diff_pixels, width, height);

        panic!(
            "Screenshot regression for '{name}': {:.2}% pixels differ (threshold: {:.2}%).\n\
             Reference: {}\n\
             Output:    {}\n\
             Diff:      {}",
            diff * 100.0,
            max_diff_fraction * 100.0,
            reference_path.display(),
            output_path.display(),
            diff_path.display(),
        );
    }
}

/// Render a view, compare against reference PNG. If no reference exists, create it.
fn screenshot_test(renderer: &mut TestRenderer, view: &impl View, name: &str) {
    let pixels = renderer.render_view(view, WIDTH, HEIGHT);

    let reference_path = reference_dir().join(format!("{name}.png"));
    let output_path = output_dir().join(format!("{name}.png"));

    // Always save the current output for inspection.
    save_png(&output_path, &pixels, WIDTH, HEIGHT);

    if !reference_path.exists() {
        // First run: create the reference.
        std::fs::create_dir_all(reference_path.parent().unwrap()).ok();
        save_png(&reference_path, &pixels, WIDTH, HEIGHT);
        eprintln!("Created reference screenshot: {}", reference_path.display());
        return;
    }

    let (ref_pixels, ref_w, ref_h) = load_png(&reference_path);
    assert_eq!(ref_w, WIDTH, "reference width mismatch for {name}");
    assert_eq!(ref_h, HEIGHT, "reference height mismatch for {name}");

    // Allow a small per-channel threshold for GPU rounding differences.
    let threshold = 2u8;
    let max_diff_fraction = 0.001; // 0.1% of pixels
    let diff = pixel_diff_fraction(&pixels, &ref_pixels, threshold);

    if diff > max_diff_fraction {
        // Save a diff image for debugging.
        let diff_path = output_dir().join(format!("{name}_diff.png"));
        let diff_pixels: Vec<u8> = pixels
            .chunks(4)
            .zip(ref_pixels.chunks(4))
            .flat_map(|(a, b)| {
                let differs = (0..4).any(|c| {
                    (a[c] as i16 - b[c] as i16).unsigned_abs() > threshold as u16
                });
                if differs {
                    [255, 0, 0, 255] // Red for differing pixels
                } else {
                    [0, 0, 0, 255] // Black for matching pixels
                }
            })
            .collect();
        save_png(&diff_path, &diff_pixels, WIDTH, HEIGHT);

        panic!(
            "Screenshot regression for '{name}': {:.2}% pixels differ (threshold: {:.2}%).\n\
             Reference: {}\n\
             Output:    {}\n\
             Diff:      {}",
            diff * 100.0,
            max_diff_fraction * 100.0,
            reference_path.display(),
            output_path.display(),
            diff_path.display(),
        );
    }
}

// --- Tests ---

#[test]
fn test_screenshot_rectangle() {
    let mut renderer = TestRenderer::new();
    let ui = rectangle().color(Color::new(0.2, 0.5, 0.8, 1.0)).size([200.0, 100.0]);
    screenshot_test(&mut renderer, &ui, "rectangle");
}

#[test]
fn test_screenshot_circle() {
    let mut renderer = TestRenderer::new();
    let ui = circle().color(Color::new(1.0, 0.3, 0.3, 1.0)).size([200.0, 200.0]);
    screenshot_test(&mut renderer, &ui, "circle");
}

#[test]
fn test_screenshot_rounded_rect() {
    let mut renderer = TestRenderer::new();
    let ui = rectangle()
        .corner_radius(20.0)
        .color(Color::new(0.3, 0.8, 0.3, 1.0))
        .size([250.0, 150.0]);
    screenshot_test(&mut renderer, &ui, "rounded_rect");
}

#[test]
fn test_screenshot_padded_circle() {
    let mut renderer = TestRenderer::new();
    let ui = circle()
        .color(Color::new(0.8, 0.8, 0.2, 1.0))
        .size([200.0, 200.0])
        .padding(30.0);
    screenshot_test(&mut renderer, &ui, "padded_circle");
}

#[test]
fn test_screenshot_vlist() {
    let mut renderer = TestRenderer::new();
    let colors = [
        Color::new(1.0, 0.2, 0.2, 1.0),
        Color::new(0.2, 1.0, 0.2, 1.0),
        Color::new(0.2, 0.2, 1.0, 1.0),
    ];
    let ui = list(vec![0, 1, 2], move |id| {
        rectangle()
            .color(colors[*id as usize])
            .size([300.0, 80.0])
    });
    screenshot_test(&mut renderer, &ui, "vlist");
}

#[test]
fn test_screenshot_hlist() {
    let mut renderer = TestRenderer::new();
    let colors = [
        Color::new(1.0, 0.5, 0.0, 1.0),
        Color::new(0.0, 0.5, 1.0, 1.0),
        Color::new(0.5, 0.0, 1.0, 1.0),
    ];
    let ui = hlist(vec![0, 1, 2], move |id| {
        rectangle()
            .color(colors[*id as usize])
            .size([100.0, 200.0])
    });
    screenshot_test(&mut renderer, &ui, "hlist");
}

#[test]
fn test_screenshot_nested() {
    let mut renderer = TestRenderer::new();
    let ui = list(vec![0, 1], |id| {
        let color = if *id == 0 {
            Color::new(0.8, 0.2, 0.5, 1.0)
        } else {
            Color::new(0.2, 0.5, 0.8, 1.0)
        };
        rectangle()
            .color(color)
            .size([200.0, 80.0])
            .padding(10.0)
    });
    screenshot_test(&mut renderer, &ui, "nested");
}

#[test]
fn test_screenshot_overlapping_zlist() {
    let mut renderer = TestRenderer::new();
    let colors = [
        Color::new(0.0, 0.0, 1.0, 1.0),
        Color::new(1.0, 1.0, 0.0, 0.8),
    ];
    let sizes: [f32; 2] = [300.0, 200.0];
    let ui = zlist(vec![0, 1], move |id| {
        let i = *id as usize;
        rectangle()
            .corner_radius(if i == 1 { 100.0 } else { 0.0 })
            .color(colors[i])
            .size([sizes[i], sizes[i]])
    });
    screenshot_test(&mut renderer, &ui, "zlist_overlap");
}

#[test]
fn test_screenshot_cond() {
    let mut renderer = TestRenderer::new();
    let ui = cond(
        true,
        circle().color(Color::new(0.0, 1.0, 0.5, 1.0)).size([250.0, 250.0]),
        rectangle().color(Color::new(1.0, 0.0, 0.0, 1.0)).size([250.0, 250.0]),
    );
    screenshot_test(&mut renderer, &ui, "cond_true");
}

/// Synth-style control panel layout.
/// The real synth UI is tested in examples/synth/tests/screenshot.rs
/// using the actual synth view tree.
#[test]
fn test_screenshot_synth_controls() {
    let mut renderer = TestRenderer::new();

    let dark_bg = Color::new(0.15, 0.15, 0.15, 1.0);
    let control_bg = Color::new(0.3, 0.3, 0.3, 1.0);
    let highlight = Color::new(0.2, 0.5, 1.0, 1.0);

    let wave_buttons = hstack((
        text("Wave").font_size(10).padding(Auto),
        rectangle().corner_radius(4.0).color(control_bg).size([40.0, 25.0]).padding(2.0),
        rectangle().corner_radius(4.0).color(control_bg).size([40.0, 25.0]).padding(2.0),
        rectangle().corner_radius(4.0).color(highlight).size([40.0, 25.0]).padding(2.0),
        rectangle().corner_radius(4.0).color(control_bg).size([40.0, 25.0]).padding(2.0),
    )).padding(5.0);

    let octave = hstack((
        text("Oct").font_size(10).padding(Auto),
        rectangle().corner_radius(4.0).color(control_bg).size([25.0, 25.0]).padding(2.0),
        text("4").font_size(14).padding(Auto),
        rectangle().corner_radius(4.0).color(control_bg).size([25.0, 25.0]).padding(2.0),
    )).padding(5.0);

    let gain = hstack((
        text("Gain").font_size(10).size([35.0, 18.0]),
        rectangle().corner_radius(2.0).color(control_bg).size([120.0, 8.0]).padding(Auto),
    )).padding(5.0);

    let slider_row = |label: &str| {
        hstack((
            text(label).font_size(10).size([55.0, 18.0]),
            rectangle().corner_radius(2.0).color(control_bg).size([140.0, 8.0]).padding(Auto),
        )).padding(2.0)
    };

    let ui = vstack((
        hstack((wave_buttons, octave, gain)).size([700.0, 40.0]),
        hstack((
            vstack((
                text("Envelope").font_size(10).padding(Auto),
                slider_row("Attack"), slider_row("Decay"),
                slider_row("Sustain"), slider_row("Release"),
            )).padding(5.0),
            vstack((
                text("Filter").font_size(10).padding(Auto),
                slider_row("Cutoff"), slider_row("Reso"),
            )).padding(5.0),
            vstack((
                text("Unison").font_size(10).padding(Auto),
                slider_row("Detune"),
                hstack((
                    text("Voices").font_size(10).size([55.0, 18.0]),
                    rectangle().corner_radius(4.0).color(control_bg).size([25.0, 20.0]).padding(2.0),
                    text("1").font_size(12).padding(Auto),
                    rectangle().corner_radius(4.0).color(control_bg).size([25.0, 20.0]).padding(2.0),
                )).padding(2.0),
            )).padding(5.0),
        )).size([700.0, 130.0]),
        hlist(vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13], |id| {
            let is_black = matches!(id, 1 | 3 | 6 | 8 | 10);
            let color = if is_black { Color::new(0.1, 0.1, 0.1, 1.0) }
                else { Color::new(0.95, 0.95, 0.95, 1.0) };
            rectangle().color(color).size([40.0, 120.0]).padding(1.0)
        }),
    )).background(rectangle().color(dark_bg));

    screenshot_test_sized(&mut renderer, &ui, "synth_controls", 768, 512);
}
