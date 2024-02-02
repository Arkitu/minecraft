use bevy::prelude::*;

pub struct CracksPlugin;
impl Plugin for CracksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cracks>()
            .add_systems(Startup, setup_cracks);
    }
}

#[derive(Resource, Default)]
pub struct Cracks (pub [Handle<Image>; 5]);

pub fn setup_cracks(
    asset_server: Res<AssetServer>,
    mut cracks: ResMut<Cracks>
) {
    cracks.0 = [
        asset_server.load("cracks/crack_1.png"),
        asset_server.load("cracks/crack_2.png"),
        asset_server.load("cracks/crack_3.png"),
        asset_server.load("cracks/crack_4.png"),
        asset_server.load("cracks/crack_5.png")
    ];
}