
use engine::Key;
use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::f32::RADIX;
// use std::os::windows::fs::FileTypeExt;
use std::path::Path;
use std::fs::read_to_string;
use std::{thread, time};

use crate::{SpriteTile, Game, getSpriteFromSheet, getSpriteFromSheet_Demo, GameMode};

const W: f32 = 320.0;
const H: f32 = 240.0;

const TILE_SIZE: u16 = 256;




fn newSpriteTile_Square(pos_x: f32, pos_y: f32, size: f32, tex_x: u16, tex_y: u16) -> SpriteTile {
    SpriteTile {
        collision: AABB::new(pos_x, pos_y, size, size),
        tex_coord: (tex_x, tex_y),
    }
}


fn newSpriteTile_Rect(pos_x: f32, pos_y: f32, width: f32, height: f32, tex_x: u16, tex_y: u16) -> SpriteTile {
    SpriteTile {
        collision: AABB::new(pos_x, pos_y, width, height),
        tex_coord: (tex_x, tex_y),
    }
}

pub fn initialize(simon_says_objects: &mut Vec<SpriteTile>){

    simon_says_objects.push(newSpriteTile_Rect(W/4.0, H/2.0, W/8.0, H/4.0, 1, 0));          // Left
    simon_says_objects.push(newSpriteTile_Rect(3.0*W/4.0, H/2.0, W/8.0, H/4.0, 2, 2));      // Right
    simon_says_objects.push(newSpriteTile_Rect(W/2.0, 3.0*H/4.0, W/8.0, H/4.0, 0, 2));      // Top
    simon_says_objects.push(newSpriteTile_Rect(W/2.0, H/4.0, W/8.0, H/4.0, 2, 0));      // Bottom

}

pub fn update_simon_says(game: &mut Game, engine: &mut Engine){
    
    if engine.input.is_mouse_pressed(winit::event::MouseButton::Left) {
        // TODO screen -> multicord needed

        let scaling_factor: f32 = 5.0;
            
        let window_height = engine.renderer.gpu.config.height as f32;
        let window_witdh = engine.renderer.gpu.config.width as f32;

        let mouse_pos = engine.input.mouse_pos();
        // Normalize mouse clicks to be 00 at bottom left corner
        let (x_norm, y_norm) = ((mouse_pos.x as f32 + game.camera.screen_pos[0])/scaling_factor,
                                (((mouse_pos.y as f32 - window_height) * (-1.0 as f32)) + game.camera.screen_pos[1])/scaling_factor);


        for ss_object in game.simon_says_objects.iter() {
            if(ss_object.collision.contains(x_norm, y_norm)){
                println!{"({},{})", ss_object.tex_coord.0, ss_object.tex_coord.1};
            }else{
                // println!("no selection");
            }
        }


    }

    if engine.input.is_key_pressed(engine::Key::S) {
        game.mode = GameMode::Platformer;
        if !matches!(game.mode, GameMode::SimonSays) {
            println!("here");
            render_simon_says(game, engine)
        }
    }


}

pub fn render_simon_says(game: &mut Game, engine: &mut Engine) {

    let DEMO_SPRITE_GROUP = 0;
    let TILE_SPRITE_GROUP = 1;
    let SIMON_SAYS_SPRITE_GROUP = 2;


    // [idx 0, idx 1..guy_idx, guy_idx, apple_start..)]
    // [Background, walls..., guy, apples...]


    // set bg image
    let (trfs1, uvs1) = engine.renderer.sprites.get_sprites_mut(SIMON_SAYS_SPRITE_GROUP);
    trfs1[0] = AABB::new(W /2.0, H /2.0, W, H).into();  // Create a non-collision AABB for use in the background
    uvs1[0] = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &(3,0), 16, TILE_SIZE);

    
    
    // let (trfs2, uvs2) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);

    // set walls
    const WALL_START: usize = 1;

    for (ss_object, (trf, uv)) in game.simon_says_objects.iter().zip(
        trfs1[WALL_START..]
            .iter_mut()
            .zip(uvs1[WALL_START..].iter_mut()),
    ) {
        *trf = (ss_object.collision).into();
        *uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &ss_object.tex_coord, 12, TILE_SIZE);
    }


    if !matches!(game.mode, GameMode::SimonSays) {
        trfs1.fill(Transform::zeroed());
    }

    
    engine
        .renderer
        .sprites
        .upload_sprites(&engine.renderer.gpu, SIMON_SAYS_SPRITE_GROUP, 0..WALL_START + game.simon_says_objects.len() + 1);
    // engine
    //     .renderer
    //     .sprites
    //     .upload_sprites(&engine.renderer.gpu, SIMON_SAYS_SPRITE_GROUP, 0..0);
    engine
        .renderer
        .sprites
        .set_camera_all(&engine.renderer.gpu, game.camera);
}