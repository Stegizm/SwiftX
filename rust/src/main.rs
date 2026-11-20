use swiftx::app::App;
use swiftx::core::constants;
use swiftx::ipc::UserEvent;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

fn main() {
    // Teshis: yalnizca guncelleme kontrolu yapip cik.
    if std::env::args().any(|a| a == "--check-update") {
        match swiftx::core::updater::check() {
            Ok(Some(i)) => println!("Yeni surum mevcut: v{} ({})", i.version, i.page_url),
            Ok(None) => println!("Guncelleme yok (mevcut surum v{}).", constants::VERSION),
            Err(e) => println!("Guncelleme kontrolu basarisiz: {e}"),
        }
        return;
    }

    // wry'nin Linux arka ucu (webkit2gtk) alt-web gorunumleri icin X11 gerektirir.
    // Wayland oturumlarinda XWayland uzerinden calismasi icin X11 arka ucunu sec.
    if std::env::var_os("GDK_BACKEND").is_none() && std::env::var_os("DISPLAY").is_some() {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(format!("SwiftX {}", constants::VERSION))
        .with_inner_size(tao::dpi::LogicalSize::new(1360.0, 860.0))
        .build(&event_loop)
        .expect("pencere olusturulamadi");

    let mut app = App::new(window, proxy);

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    app.save_session();
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(_) => app.resized(),
                WindowEvent::Destroyed => {
                    app.save_session();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::UserEvent(user_event) => app.handle(user_event),
            _ => {}
        }
    });
}
