
use engine::Key;
use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::f32::RADIX;
use std::f32::consts::PI;
// use std::os::windows::fs::FileTypeExt;
use std::path::Path;
use std::fs::read_to_string;
use std::time::Duration;
use std::{thread, time};


use crate::{SpriteTile, Game, getSpriteFromSheet, getSpriteFromSheet_Demo, GameMode};

const W: f32 = 320.0;
const H: f32 = 240.0;

const TILE_SIZE: u16 = 256;
#[cfg(target_os = "macos")]
const SCALING_FACTOR: f32 = 5.0;
#[cfg(target_os = "windows")]
const SCALING_FACTOR: f32 = 2.5;
const PATTERN_DELAY: Duration = time::Duration::from_millis(500);

pub struct SimonSaysState {
    pub knobs: Vec<(SpriteTile, f32)>, //Vec of sprites and their rotation
    pub pattern: Vec<usize>,   //The pattern
    pub pattern_counter: usize,
    pub awaitInput: bool,
    pub completed: bool,
}


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

pub fn initialize() -> SimonSaysState{

    let mut knobs = Vec::default();
    knobs.push((newSpriteTile_Square(W/4.0, H/2.0,  H/4.0, 2, 0), 0.0));          // Left
    knobs.push((newSpriteTile_Square(3.0*W/4.0, H/2.0,  H/4.0, 2, 0), 0.0));      // Right
    knobs.push((newSpriteTile_Square(W/2.0, 3.0*H/4.0,  H/4.0, 2, 0), 0.0));      // Top
    knobs.push((newSpriteTile_Square(W/2.0, H/4.0,  H/4.0, 2, 0), 0.0));          // Bottom

    let mut pattern = Vec::default();
    pattern.push(rand::thread_rng().gen_range(0..=3));
    pattern.push(rand::thread_rng().gen_range(0..=3));

    SimonSaysState { 
        knobs, 
        pattern, 
        pattern_counter: 0,
        awaitInput: false,
        completed: false,
    }
}

pub fn update_simon_says(game: &mut Game, engine: &mut Engine){
    
    if game.simon_says.awaitInput {
        if engine.input.is_mouse_pressed(winit::event::MouseButton::Left) {

            let window_height = engine.renderer.gpu.config.height as f32;
    
            let mouse_pos = engine.input.mouse_pos();
            // Normalize mouse clicks to be 00 at bottom left corner
            let (x_norm, y_norm) = ((mouse_pos.x as f32 + game.camera.screen_pos[0])/SCALING_FACTOR,
                                    (((mouse_pos.y as f32 - window_height) * (-1.0 as f32)) + game.camera.screen_pos[1])/SCALING_FACTOR);
    
            println!("MOUSE: {}, {}\nNORM: {}, {}", mouse_pos.x, mouse_pos.y, x_norm,y_norm);
            let mut doRestart = false;
            let mut finishedPattern = false;

            println!("{}, {}, x{}, y{}", mouse_pos.x, mouse_pos.y, x_norm, y_norm);

            for (idx, ss_object) in game.simon_says.knobs.iter_mut().enumerate() {
                if(ss_object.0.collision.contains(x_norm, y_norm)){
                    // Clicked on a knob
                    if idx == game.simon_says.pattern[game.simon_says.pattern_counter] {
                        // Clicked on the correct knob, continue
                        ss_object.1 -= PI/4.0; // Rotate the thing by 45 degrees
                        game.simon_says.pattern_counter += 1;
                        if game.simon_says.pattern_counter >= game.simon_says.pattern.len() {
                            finishedPattern = true;
                        }
                        break;
                    }else{
                        // Clicked on the wrong knob, restart
                        doRestart = true;
                        break;

                    }

                }
            }

            if doRestart {
                
                for ss_object in game.simon_says.knobs.iter_mut() {
                    ss_object.1 = 0.0;
                }

                while game.simon_says.pattern.len() > 2 {
                    game.simon_says.pattern.pop();
                }

                game.simon_says.pattern_counter = 0;
                game.simon_says.awaitInput = false;

            }else if finishedPattern {
                if game.simon_says.pattern_counter > 5 {
                    game.simon_says.completed = true;
                    game.mode = GameMode::Platformer;
                    render_simon_says(game, engine);
                    return;
                }
                game.simon_says.pattern.push(rand::thread_rng().gen_range(0..=3));
                game.simon_says.pattern_counter = 0;
                game.simon_says.awaitInput = false;
            }
    
    
        }
    }else{
        //Perform the pattern

        game.simon_says.knobs[game.simon_says.pattern[game.simon_says.pattern_counter]].1 += PI/4.0;
        game.simon_says.pattern_counter += 1;
        thread::sleep(PATTERN_DELAY);
        
        if(game.simon_says.pattern_counter >= game.simon_says.pattern.len()) {
            game.simon_says.awaitInput = true;
            game.simon_says.pattern_counter = 0;
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

    for (ss_object, (trf, uv)) in game.simon_says.knobs.iter().zip(
        trfs1[WALL_START..]
            .iter_mut()
            .zip(uvs1[WALL_START..].iter_mut()),
    ) {
        *trf = ss_object.0.collision.to_transform_rot(ss_object.1);
        *uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &ss_object.0.tex_coord, 12, TILE_SIZE);
    }


    if !matches!(game.mode, GameMode::SimonSays) {
        trfs1.fill(Transform::zeroed());
    }

    
    engine
        .renderer
        .sprites
        .upload_sprites(&engine.renderer.gpu, SIMON_SAYS_SPRITE_GROUP, 0..WALL_START + game.simon_says.knobs.len() + 1);
    // engine
    //     .renderer
    //     .sprites
    //     .upload_sprites(&engine.renderer.gpu, SIMON_SAYS_SPRITE_GROUP, 0..0);
    engine
        .renderer
        .sprites
        .set_camera_all(&engine.renderer.gpu, game.camera);
}