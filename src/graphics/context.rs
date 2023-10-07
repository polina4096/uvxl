#![allow(unused_imports)]
use cfg_if::cfg_if;
use log::info;
use wgpu::{Backends, PowerPreference};
use winit::window::Window;
use winit::dpi::PhysicalSize;

pub struct Graphics {
  pub device  : wgpu::Device,
  pub surface : wgpu::Surface,
  pub queue   : wgpu::Queue,
  pub format  : wgpu::TextureFormat,
  pub config  : wgpu::SurfaceConfiguration,

  pub size    : PhysicalSize<u32>,
  pub scale   : f64,
}

impl Graphics {
  pub async fn new(window: &Window) -> Graphics {
    let instance = wgpu::Instance::new(
      wgpu::InstanceDescriptor {
        backends             : Backends::all(),
        dx12_shader_compiler : Default::default(),
      }
    );

    // # Safety
    // The surface needs to live as long as the window that created it.
    // State owns the window so this should be safe.
    let surface = unsafe { instance.create_surface(&window) }
      .expect("Failed to create a surface");

    cfg_if! {
      if #[cfg(target_arch = "wasm32")] {
        // request_adapter is not available on WASM platform
        let adapter = instance.enumerate_adapters(wgpu::Backends::all())
          .find(|adapter| {
              // Check if this adapter supports our surface
              adapter.is_surface_supported(&surface)
          }).unwrap();
      } else {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
          power_preference       : PowerPreference::HighPerformance,
          compatible_surface     : Some(&surface),
          force_fallback_adapter : false,
        }).await.expect("Failed to retrive an adapter");
      }
    }

    let info = adapter.get_info();
    info!("Selected GPU: {} | ({:?})", info.name, info.device_type);
    info!("Selected backend: {:?}", info.backend);
    info!("Driver: {} | {}", info.driver, info.driver_info);

    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        label    : None,
        features : wgpu::Features::empty(),

        // WebGL doesn't support all of wgpu's features, so if
        // we're building for the web we'll have to disable some.
        limits: {
          if cfg!(target_arch = "wasm32")
               { wgpu::Limits::downlevel_webgl2_defaults() }
          else { wgpu::Limits::default()                   }
        },
      },
      None, // Trace path
    ).await.expect("Failed to retrieve a device");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter().copied()
      .find(|f| *f == wgpu::TextureFormat::Rgba8UnormSrgb) // f.is_srgb()
      .unwrap_or(surface_caps.formats[0]);
    info!("Surface format: {:?}", surface_format);

    let size = window.inner_size();
    assert_ne!(size.width, 0);
    assert_ne!(size.height, 0);

    let present_mode = wgpu::PresentMode::AutoVsync;
    info!("Present mode: {:?}", present_mode);

    let surface_config = wgpu::SurfaceConfiguration {
      usage        : wgpu::TextureUsages::RENDER_ATTACHMENT,
      format       : surface_format,
      width        : size.width,
      height       : size.height,
      present_mode : present_mode,
      alpha_mode   : surface_caps.alpha_modes[0],
      view_formats : vec![],
    };

    surface.configure(&device, &surface_config);

    let scale = window.scale_factor();

    return Graphics {
      device,
      surface,
      queue,
      format: surface_format,
      config: surface_config,

      size,
      scale,
    };
  }
}
