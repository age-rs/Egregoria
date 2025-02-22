use common::FastMap;

use imgui::TextureId;
use legion::Entity;
use serde::{Deserialize, Serialize};

use crate::input::{KeyCode, KeyboardInfo};
use crate::uiworld::UiWorld;
use egregoria::Egregoria;
use geom::{Camera, AABB};
use roadbuild::RoadBuildResource;
use wgpu_engine::GfxContext;

mod bulldozer;
mod follow;
mod inspect;
mod inspected_aura;
mod lotbrush;
mod roadbuild;
mod roadeditor;
mod selectable;
mod specialbuilding;
mod topgui;

pub mod windows;

pub use follow::FollowEntity;
pub use inspect::*;
pub use topgui::*;

pub fn run_ui_systems(goria: &Egregoria, uiworld: &mut UiWorld) {
    bulldozer::bulldozer(goria, uiworld);
    inspected_aura::inspected_aura(goria, uiworld);
    lotbrush::lotbrush(goria, uiworld);
    roadbuild::roadbuild(goria, uiworld);
    roadeditor::roadeditor(goria, uiworld);
    selectable::selectable(goria, uiworld);
    specialbuilding::specialbuilding(goria, uiworld);
    hand_reset(uiworld);

    let eye = uiworld.read::<Camera>().pos;
    let cam = AABB::new(eye, eye).expand(2000.0);
    if goria.map().trees.check_non_generated_chunks(cam) {
        uiworld.commands().map_generate_trees(cam);
    }
}

register_resource_noserialize!(InspectedEntity);
#[derive(Copy, Clone, Debug)]
pub struct InspectedEntity {
    pub e: Option<Entity>,
    pub dist2: f32,
}

impl Default for InspectedEntity {
    fn default() -> Self {
        Self {
            e: None,
            dist2: 0.0,
        }
    }
}

pub fn hand_reset(uiworld: &mut UiWorld) {
    let info = uiworld.read::<KeyboardInfo>();
    if info.just_pressed.contains(&KeyCode::Escape) {
        *uiworld.write::<Tool>() = Tool::Hand;
    }
}

register_resource_noserialize!(Tool);
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Tool {
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush,
    SpecialBuilding,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum UiTex {
    Road,
    Curved,
    RoadEdit,
    Bulldozer,
    Buildings,
    LotBrush,
}

const UI_TEXTURES: &[(UiTex, &str)] = &[
    (UiTex::Road, "assets/ui/road.png"),
    (UiTex::Curved, "assets/ui/curved.png"),
    (UiTex::RoadEdit, "assets/ui/road_edit.png"),
    (UiTex::Bulldozer, "assets/ui/bulldozer.png"),
    (UiTex::Buildings, "assets/ui/buildings.png"),
    (UiTex::LotBrush, "assets/ui/lotbrush.png"),
];

pub struct UiTextures {
    textures: FastMap<UiTex, TextureId>,
}

impl UiTextures {
    pub fn new(gfx: &GfxContext, renderer: &mut imgui_wgpu::Renderer) -> Self {
        let mut textures = common::fastmap_with_capacity(UI_TEXTURES.len());
        for &(name, path) in UI_TEXTURES {
            let (img, width, height) = wgpu_engine::Texture::read_image(path)
                .expect(&*format!("Couldn't load gui texture {}", path));

            let mut config = imgui_wgpu::TextureConfig::default();
            config.size.width = width;
            config.size.height = height;

            let imgui_tex = imgui_wgpu::Texture::new(&gfx.device, renderer, config);
            imgui_tex.write(&gfx.queue, &img, width, height);

            textures.insert(name, renderer.textures.insert(imgui_tex));
        }
        Self { textures }
    }

    pub fn get(&self, name: UiTex) -> TextureId {
        *self.textures.get(&name).unwrap()
    }
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Hand
    }
}
