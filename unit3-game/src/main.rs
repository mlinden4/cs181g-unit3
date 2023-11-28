// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::Key;
use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use winit::platform;
use std::f32::RADIX;
// use std::os::windows::fs::FileTypeExt;
use std::path::Path;
use std::fs::read_to_string;
use std::{thread, time};

mod platformer;

const W: f32 = 320.0;
const H: f32 = 240.0;
const SPRITE_MAX: usize = 128;


const TOP_HALF_COLLISION: [(u16, u16); 4] = [(0, 3), (1, 3), (2, 3), (3, 3)];
// const BOT_HALF_COLLISION: [(u16, u16); 2] = [(0,0), (2,2)];
// const DEATH_COLLISION: [(u16, u16); 2] = [(0,0), (2,2)];
// const DOOR_COLLISION: [(u16, u16); 6] = [(6,0), (6,1), (6,2), (6,3), (5,3), (5,4)];


// const LEFT: &'static [&'static str] = &["Hello", "World", "!"];

// const TILE_SIZE: u16 = 256;
// const TILE_SHEET_W: u16 = 7 * TILE_SIZE;
// const TILE_SHEET_H: u16 = 5 * TILE_SIZE;


pub struct SpriteTile {
    collision: AABB,
    tex_coord: (u16, u16),
}

pub struct Game {
    camera: engine::Camera,
    collision_objects: Vec<SpriteTile>,
    doors: Vec<u16>,
    guy: platformer::Guy,
    level:u32,
    mode: GameMode,

}

enum GameMode {
    Platformer,
    SimonSays,
    ConnectWires,
    // Other modes...
}

fn newSpriteGroup(sprite_path: &str, engine: &mut Engine, camera_ref: &Camera) {
    
    let camera = camera_ref.clone();

    let sprite_img = image::open(sprite_path).unwrap().into_rgba8();
    
    let sprite_tex = engine.renderer.gpu.create_texture(
        &sprite_img,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        sprite_img.dimensions(),
        Some(sprite_path),  // Some string or something
    );

    engine.renderer.sprites.add_sprite_group(
        &engine.renderer.gpu,
        &sprite_tex,
        vec![Transform::zeroed(); SPRITE_MAX], //bg, three walls, guy, a few apples
        vec![SheetRegion::zeroed(); SPRITE_MAX],
        camera,
    );

}

fn getSpriteFromSheet(sheet_num: u16, tex_coord: &(u16,u16), depth: u16, sprite_size: u16) -> SheetRegion {
    if TOP_HALF_COLLISION.contains(tex_coord) {
        SheetRegion::new(sheet_num, tex_coord.0*sprite_size, tex_coord.1*sprite_size, depth, sprite_size, sprite_size/2)
    }else{
        // *trf = (wall.collision).into();
        SheetRegion::new(sheet_num, tex_coord.0*sprite_size, tex_coord.1*sprite_size, depth, sprite_size, sprite_size)
    }
}

// Meant to just get it directly based on data
fn getSpriteFromSheet_Demo(sheet_num: u16, x: u16, y: u16, depth: u16, w: u16, h: u16) -> SheetRegion {
    SheetRegion::new(sheet_num, x, y, depth, w, h)
}

impl engine::Game for Game {

    fn new(engine: &mut Engine) -> Self {
        let camera = Camera {
            screen_pos: [0.0, 0.0],
            screen_size: [W, H],
        };
        #[cfg(target_arch = "wasm32")]
        let sprite_img = {
            let img_bytes = include_bytes!("content/demo.png");
            image::load_from_memory_with_format(&img_bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())
                .unwrap()
                .into_rgba8()
        };
        #[cfg(not(target_arch = "wasm32"))]
        newSpriteGroup("content/Swordsman/Idle.png", engine, &camera); // 0
        newSpriteGroup("content/new_spritesheet.png", engine, &camera); // 1
        //newSpriteGroup("content/Objects/DoorUnlocked.png", engine, &camera); // 2

        let guy = platformer::Guy {
            pos: Vec2 {
                x: W / 2.0,
                y: H / 2.0,
            },
            vel: Vec2 {
                x: 0.0,
                y: 0.0,
            },
            grounded: false,
        };

        
        
        let mut collision_objects: Vec<SpriteTile> = Vec::default(); 
        let mut doors: Vec<u16> = Vec::default(); 
        platformer::loadLevel(&mut collision_objects, &mut doors, 0);


        //              size_x
        //            --------------
        //   size_y   | c_xy x     |
        //            --------------

 
        // let font = engine::BitFont::with_sheet_region(
        //     '0'..='9',
        //     SheetRegion::new(0, 0, 512, 0, 80, 8),
        //     10,
        // );
        
        Game {
            camera,
            guy,
            collision_objects,
            doors,
            level: 0,
            mode: GameMode::Platformer
        }
    }



    fn update(&mut self, engine: &mut Engine) {

        match self.mode {
            GameMode::Platformer => platformer::update_platformer(self, engine),
            GameMode::ConnectWires => (),
            GameMode::SimonSays => (),
        }
        

    }
    
    
    
    fn render(&mut self, engine: &mut Engine) {

        match self.mode {
            GameMode::Platformer => platformer::render_platformer(self, engine),
            GameMode::ConnectWires => (),
            GameMode::SimonSays => (),
        }
        
    }


}
fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}