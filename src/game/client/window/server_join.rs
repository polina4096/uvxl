use egui::Align2;
use log::error;
use winit::window::CursorGrabMode;

use crate::{app::{App, UVxlEvent}, network::connection::Connection};

use super::{Window, WindowId};

pub struct ServerJoinWindow {
  pub address: String,
  pub name: String,
}

impl Default for ServerJoinWindow {
    fn default() -> Self {
        #[allow(unreachable_code, unused_labels)]
        Self { name: String::new(), address: 'a: {
          #[cfg(target_arch = "wasm32")] {
            break 'a String::from("127.0.0.1:2489");
          }; String::from("127.0.0.1:2488")
        } }
    }
}

impl Window for ServerJoinWindow {
  fn draw(&mut self, app: &mut App) {
    egui::Window::new("Join server")
      .collapsible(false)
      .fixed_size((192.0, 0.0))
      .anchor(Align2::CENTER_TOP, (0.0, 192.0))
      .show(&app.egui_ctx.context, |ui|
    {
      ui.add_enabled_ui(app.connection.is_none(), |ui| {
        ui.label("Name:");
        let edit = ui.text_edit_singleline(&mut self.name);

        ui.label("Address:");
        let edit = ui.text_edit_singleline(&mut self.address);
        let button = ui.button("Join");

        if button.clicked() || edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
          if let Err(err) = app.event_proxy.send_event(UVxlEvent::SetClientName(self.name.clone())) {
            error!("Failed to send UVxl event: {}", err);
          }

          let Ok(address) = self.address.parse() else { todo!() };
          let Ok(connection) = Connection::new(address, app.event_proxy.clone()) else { todo!() };
          app.connection = Some(connection);
          app.window.set_cursor_grab(CursorGrabMode::Locked)
            .unwrap_or_else(|err| error!("Failed to confine mouse cursor: {}", err));
        }
      });
    });
  }

  fn id(&self) -> WindowId { WindowId::ServerJoin }
}