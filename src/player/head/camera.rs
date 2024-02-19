use bevy::{prelude::*, input::mouse::MouseMotion, window::{PrimaryWindow, CursorGrabMode}};
use crate::{PlayerMarker, HeadMarker};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rotate_camera);

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, cursor_grab)
            .insert_resource(WasmMouseTracker::new());

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, cursor_grab)
            .add_systems(Update, cursor_release);
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

pub fn rotate_camera_from_vec2(mov: Vec2, player_pos: &mut Mut<Transform>, cam_pos: &mut Mut<Transform>, config: &mut Mut<CameraConfig>) {
    config.yaw -= mov.x * config.sensi_x;
    config.pitch -= mov.y * config.sensi_y;
    config.pitch = config.pitch.clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);
    
    player_pos.rotation = Quat::from_axis_angle(Vec3::Y, config.yaw); 
    cam_pos.rotation = Quat::from_axis_angle(Vec3::X, config.pitch);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn rotate_camera(
    mut motion_evr: EventReader<MouseMotion>,
    mut cam: Query<(&mut CameraConfig, &mut Transform), (With<HeadMarker>, Without<PlayerMarker>)>,
    mut player_pos: Query<&mut Transform, (With<PlayerMarker>, Without<HeadMarker>)>
) {
    let (mut config, mut cam_pos) = cam.single_mut();
    let mut player_pos = player_pos.single_mut();
    for ev in motion_evr.read() {
        let mov = ev.delta;
        rotate_camera_from_vec2(mov, &mut player_pos, &mut cam_pos, &mut config);
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

#[cfg(not(target_arch = "wasm32"))]
pub fn cursor_release(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return
    }
    let mut primary_window = q_windows.single_mut();

    primary_window.cursor.grab_mode = CursorGrabMode::None;

    primary_window.cursor.visible = true;
}






// Wasm support

#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, atomic::{AtomicI32, AtomicBool, Ordering::SeqCst}};
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
    mouse_down: Arc<AtomicBool>
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

        let mouse_down = Arc::new(AtomicBool::new(false));

        let md = Arc::clone(&mouse_down);
        let on_mouse_down = gloo::events::EventListener::new(&get_body(), "mousedown", move |_| {
            md.store(true, SeqCst);
        });
        on_mouse_down.forget();

        let md = Arc::clone(&mouse_down);
        let on_mouse_up = gloo::events::EventListener::new(&get_body(), "mouseup", move |_| {
            md.store(false, SeqCst);
        });
        on_mouse_up.forget();

        Self { delta_x, delta_y, mouse_down }
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
    pub fn is_mouse_down(&self) -> bool {
        self.mouse_down.load(SeqCst)
    }
}

#[cfg(target_arch = "wasm32")]
pub fn cursor_grab() {
    let body = get_body();
    body.request_pointer_lock();
    body.request_fullscreen();
}

#[cfg(target_arch = "wasm32")]
pub fn rotate_camera(
    wasm_mouse_tracker: Res<WasmMouseTracker>,
    mut cam: Query<(&mut CameraConfig, &mut Transform), (With<HeadMarker>, Without<PlayerMarker>)>,
    mut player_pos: Query<&mut Transform, (With<PlayerMarker>, Without<HeadMarker>)>
) {
    let (mut config, mut cam_pos) = cam.single_mut();
    let mut player_pos = player_pos.single_mut();
    let mov = wasm_mouse_tracker.get_delta_and_reset();
    if mov != Vec2::ZERO {
        rotate_camera_from_vec2(mov, &mut player_pos, &mut cam_pos, &mut config)
    }
}