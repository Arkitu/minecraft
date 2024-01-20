use bevy::{prelude::*, input::mouse::MouseMotion, window::{PrimaryWindow, CursorGrabMode}};

#[derive(Component)]
pub struct CameraMarker;

#[derive(Bundle)]
pub struct Camera {
    marker: CameraMarker,
    cam: Camera3dBundle,
    config: CameraConfig
}
impl Camera {
    pub fn spawn(parent: &mut ChildBuilder) {
        parent.spawn(Self {
            marker: CameraMarker,
            cam: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            },
            config: CameraConfig::default()
        });
    }
}

#[derive(Component)]
pub struct CameraConfig {
    sensi_x: f32,
    sensi_y: f32,
    yaw: f32,
    pitch: f32
}
impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig { sensi_x: 0.01, sensi_y: 0.01, yaw: 0.0, pitch: 0.0 }
    }
}

pub fn rotate_camera_from_vec2(mov: Vec2, cam_pos: &mut Mut<Transform>, config: &mut Mut<CameraConfig>) {
    config.yaw -= mov.x * config.sensi_x;
    config.pitch -= mov.y * config.sensi_y;
    config.pitch = config.pitch.clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);
    
    cam_pos.rotation = Quat::from_axis_angle(Vec3::Y, config.yaw) * Quat::from_axis_angle(Vec3::X, config.pitch);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn rotate_camera(
    mut motion_evr: EventReader<MouseMotion>,
    mut cam: Query<(&mut Transform, &mut CameraConfig), With<CameraMarker>>
) {
    let (mut cam_pos, mut config) = cam.single_mut();
    for ev in motion_evr.read() {
        let mov = ev.delta;
        rotate_camera_from_vec2(mov, &mut cam_pos, &mut config);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cursor_grab(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = q_windows.single_mut();

    // for a game that doesn't use the cursor (like a shooter):
    // use `Locked` mode to keep the cursor in one place
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;

    // also hide the cursor
    primary_window.cursor.visible = false;
}


// Wasm support

#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, atomic::{AtomicI32, Ordering::SeqCst}};
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
fn get_body() -> web_sys::HtmlElement {
    web_sys::window().unwrap().document().unwrap().body().unwrap()
}

#[cfg(target_arch = "wasm32")]
#[derive(Resource)]
pub struct WasmMouseTracker {
    delta_x: Arc<AtomicI32>,
    delta_y: Arc<AtomicI32>,
}

#[cfg(target_arch = "wasm32")]
impl WasmMouseTracker {
    pub fn new() -> Self {
        let delta_x = Arc::new(AtomicI32::new(0));
        let delta_y = Arc::new(AtomicI32::new(0));

        let dx = Arc::clone(&delta_x);
        let dy = Arc::clone(&delta_y);

        // From https://www.webassemblyman.com/rustwasm/how_to_add_mouse_events_in_rust_webassembly.html
        let on_move = gloo::events::EventListener::new(&get_body(), "mousemove", move |e| {
            let mouse_event = e.clone().dyn_into::<web_sys::MouseEvent>().unwrap();
            dx.store(mouse_event.movement_x(), SeqCst);
            dy.store(mouse_event.movement_y(), SeqCst);
        });
        on_move.forget();
        Self { delta_x, delta_y }
    }

    pub fn get_delta_and_reset(&self) -> Vec2 {
        let delta = Vec2::new(
            self.delta_x.load(SeqCst) as f32,
            self.delta_y.load(SeqCst) as f32,
        );
        self.delta_x.store(0, SeqCst);
        self.delta_y.store(0, SeqCst);
        delta
    }
}

#[cfg(target_arch = "wasm32")]
pub fn cursor_grab_wasm() {
    get_body().request_pointer_lock()
}

#[cfg(target_arch = "wasm32")]
pub fn rotate_camera(
    wasm_mouse_tracker: Res<WasmMouseTracker>,
    mut cam: Query<(&mut Transform, &mut CameraConfig), With<CameraMarker>>
) {
    let (mut cam_pos, mut config) = cam.single_mut();
    let mov = wasm_mouse_tracker.get_delta_and_reset();
    if mov != Vec2::ZERO {
        rotate_camera_from_vec2(mov, &mut cam_pos, &mut config)
    }
}