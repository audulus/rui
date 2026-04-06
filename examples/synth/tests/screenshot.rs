use futures::executor::block_on;
use rui::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use synth::{synth_view_no_audio, SynthUI};

const WIDTH: u32 = 768;
const HEIGHT: u32 = 512;
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn reference_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("screenshots")
}

fn output_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("screenshots").join("output");
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

        let offset: LocalOffset = ((sz - view_sz) / 2.0).into();
        self.vger.translate(offset);
        view.draw(&mut path, &mut DrawArgs { cx: &mut cx, vger: &mut self.vger });

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

        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

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

fn pixel_diff_fraction(a: &[u8], b: &[u8], threshold: u8) -> f64 {
    assert_eq!(a.len(), b.len());
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

#[test]
fn test_synth_screenshot() {
    let mut renderer = TestRenderer::new();

    let ui = state(SynthUI::default, move |ui_handle, _cx| {
        synth_view_no_audio(ui_handle, |_| {})
    });

    let pixels = renderer.render_view(&ui, WIDTH, HEIGHT);

    let reference_path = reference_dir().join("synth_ui.png");
    let output_path = output_dir().join("synth_ui.png");

    save_png(&output_path, &pixels, WIDTH, HEIGHT);

    if !reference_path.exists() {
        std::fs::create_dir_all(reference_path.parent().unwrap()).ok();
        save_png(&reference_path, &pixels, WIDTH, HEIGHT);
        eprintln!("Created reference screenshot: {}", reference_path.display());
        return;
    }

    let (ref_pixels, ref_w, ref_h) = load_png(&reference_path);
    assert_eq!(ref_w, WIDTH, "reference width mismatch");
    assert_eq!(ref_h, HEIGHT, "reference height mismatch");

    let threshold = 2u8;
    let max_diff_fraction = 0.001;
    let diff = pixel_diff_fraction(&pixels, &ref_pixels, threshold);

    if diff > max_diff_fraction {
        let diff_path = output_dir().join("synth_ui_diff.png");
        let diff_pixels: Vec<u8> = pixels
            .chunks(4)
            .zip(ref_pixels.chunks(4))
            .flat_map(|(a, b)| {
                let differs = (0..4).any(|c| {
                    (a[c] as i16 - b[c] as i16).unsigned_abs() > threshold as u16
                });
                if differs { [255, 0, 0, 255] } else { [0, 0, 0, 255] }
            })
            .collect();
        save_png(&diff_path, &diff_pixels, WIDTH, HEIGHT);

        panic!(
            "Screenshot regression: {:.2}% pixels differ (threshold: {:.2}%).\n\
             Reference: {}\nOutput: {}\nDiff: {}",
            diff * 100.0, max_diff_fraction * 100.0,
            reference_path.display(), output_path.display(), diff_path.display(),
        );
    }
}
