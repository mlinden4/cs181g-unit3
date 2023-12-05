use engine::Key;
use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;
use std::{thread, time};

use crate::{getSpriteFromSheet, getSpriteFromSheet_Demo, Game, GameMode, SpriteTile};

const W: f32 = 320.0;
const H: f32 = 240.0;

const TILE_SIZE: u16 = 256;
#[cfg(target_os = "macos")]
const SCALING_FACTOR: f32 = 5.0;
#[cfg(target_os = "windows")]
const SCALING_FACTOR: f32 = 2.5;

pub struct MiningState {
    pub obstacles: Vec<(SpriteTile, f32)>, //Vec of sprites and their rotation
    pub completed: bool,
    pub prize: (f32, f32),
    pub grid: Vec<(f32, f32, i32)>,
    pub prize_hits: usize,
}

fn newSpriteTile_Square(pos_x: f32, pos_y: f32, size: f32, tex_x: u16, tex_y: u16) -> SpriteTile {
    SpriteTile {
        collision: AABB::new(pos_x, pos_y, size, size),
        tex_coord: (tex_x, tex_y),
    }
}

fn newSpriteTile_Rect(
    pos_x: f32,
    pos_y: f32,
    width: f32,
    height: f32,
    tex_x: u16,
    tex_y: u16,
) -> SpriteTile {
    SpriteTile {
        collision: AABB::new(pos_x, pos_y, width, height),
        tex_coord: (tex_x, tex_y),
    }
}

pub fn initialize() -> MiningState {
    let mut obstacles = Vec::default();

    let mut x_pos = 13.0;
    let mut y_pos = 215.0;
    let size = 25;

    let mut grid = Vec::default(); //x,y,hits taken (up to 3)
                                   // generate prize behind ice
    let rand_x = rand::thread_rng().gen_range(0..=12) as f32 * 25.0 + x_pos;
    let rand_y = y_pos - rand::thread_rng().gen_range(0..=8) as f32 * 25.0;
    let obj_tex_coords = vec![(12, 0), (13, 1), (14, 1), (14, 2), (14, 5), (13, 5)];
    let obj_rand = obj_tex_coords[rand::thread_rng().gen_range(0..5)];
    obstacles.push((
        newSpriteTile_Square(rand_x, rand_y, 23.0, obj_rand.0, obj_rand.1),
        0.0,
    ));

    for i in 0..9 {
        for j in 0..13 {
            obstacles.push((newSpriteTile_Square(x_pos, y_pos, size as f32, 6, 11), 0.0));
            grid.push((x_pos, y_pos, 0));
            x_pos += size as f32;
        }
        x_pos = 13.0;
        y_pos -= size as f32;
    }

    MiningState {
        obstacles,
        completed: false,
        prize: (rand_x, rand_y),
        grid,
        prize_hits: 0,
    }
}

pub fn update_mining(game: &mut Game, engine: &mut Engine) {
    if engine.input.is_key_pressed(engine::Key::S) {
        game.mode = GameMode::Platformer;
        if !matches!(game.mode, GameMode::Mining) {
            println!("here");
            render_mining(game, engine);
        }
    }

    println!("{}, {}", game.mining.prize.0, game.mining.prize.1);
    if engine
        .input
        .is_mouse_pressed(winit::event::MouseButton::Left)
    {
        let window_height = engine.renderer.gpu.config.height as f32;
        let window_witdh = engine.renderer.gpu.config.width as f32;
        let mouse_pos = engine.input.mouse_pos();
        // Normalize mouse clicks to be 00 at bottom left corner
        let (x_norm, y_norm) = (
            (mouse_pos.x as f32 + game.camera.screen_pos[0]) / SCALING_FACTOR,
            (((mouse_pos.y as f32 - window_height) * (-1.0 as f32)) + game.camera.screen_pos[1])
                / SCALING_FACTOR,
        );
        let mut overPrize = false;
        for (idx, ss_object) in game.mining.obstacles.iter_mut().enumerate() {
            if (ss_object.0.collision.contains(x_norm, y_norm)) {
                if idx == 0 {
                    if game.mining.prize_hits < 3 {
                        game.mining.prize_hits += 1;
                    } else if game.mining.prize_hits == 3 {
                        ss_object.0.collision.center = Vec2::new(140.0, 120.0);
                        ss_object.0.collision.size = Vec2::new(64.0, 64.0);
                        game.mining.prize_hits += 1;
                    } else {
                        game.mining.completed = true;
                        game.mining.prize_hits = 0;
                        game.mode = GameMode::Platformer;
                        return;
                    }
                }
                match ss_object.0.tex_coord {
                    (6, 11) => ss_object.0.tex_coord.1 = 12,
                    (6, 12) => ss_object.0.tex_coord.0 = 7,
                    (7, 12) => ss_object.0.collision.size = Vec2::new(0.0, 0.0),
                    _ => continue,
                }
                //freeze for a split second to make sure that clicks have to be separated
                sleep(time::Duration::from_millis(10));
            }
        }
    }
}

pub fn render_mining(game: &mut Game, engine: &mut Engine) {
    let DEMO_SPRITE_GROUP = 0;
    let TILE_SPRITE_GROUP = 1;
    let MINING_SPRITE_GROUP = 4;

    // set bg image
    let (trfs1, uvs1) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);
    trfs1[0] = AABB::new(W / 2.0, H / 2.0, W, H).into(); // Create a non-collision AABB for use in the background
    uvs1[0] = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &(2, 4), 16, TILE_SIZE);

    if !matches!(game.mode, GameMode::Mining) {
        trfs1.fill(Transform::zeroed());
    }

    // set walls
    const WALL_START: usize = 1;
    // set up variables for walls
    let (trfs0, uvs0) = engine.renderer.sprites.get_sprites_mut(MINING_SPRITE_GROUP);
    // prize placement
    trfs0[WALL_START] = game.mining.obstacles[0].0.collision.into();
    if game.mining.prize_hits >= 3 {
        uvs0[WALL_START] = getSpriteFromSheet(
            MINING_SPRITE_GROUP as u16,
            &game.mining.obstacles[0].0.tex_coord,
            13,
            17,
        );
    } else {
        uvs0[WALL_START] = getSpriteFromSheet(
            MINING_SPRITE_GROUP as u16,
            &game.mining.obstacles[0].0.tex_coord,
            15,
            17,
        );
    }

    // ice placement
    let ice_idx = WALL_START + 2;
    for (ss_object, (trf, uv)) in game.mining.obstacles[1..game.mining.obstacles.len()]
        .iter()
        .zip(trfs0[ice_idx..].iter_mut().zip(uvs0[ice_idx..].iter_mut()))
    {
        *trf = ss_object.0.collision.to_transform_rot(ss_object.1);
        *uv = getSpriteFromSheet(MINING_SPRITE_GROUP as u16, &ss_object.0.tex_coord, 14, 17);

        //*uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &ss_object.0.tex_coord, 12, TILE_SIZE);
    }

    // set guy
    // let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(DEMO_SPRITE_GROUP);
    // let guy_idx = WALL_START + game.collision_objects.len();

    // trfs[guy_idx] = AABB {
    //     center: game.guy.pos + 3.0,
    //     size: Vec2 { x: 32.0, y: 32.0 },
    // }
    // .into();
    // // animate guy
    // uvs[guy_idx] = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, GUY_FRAMES[game.guy.frame].0 + 24, GUY_FRAMES[game.guy.frame].1 + 44, 8, 100, 100);

    // engine
    //     .renderer
    //     .sprites
    //     .upload_sprites(&engine.renderer.gpu, DEMO_SPRITE_GROUP, 0..WALL_START + game.collision_objects.len() + 1);
    engine.renderer.sprites.upload_sprites(
        &engine.renderer.gpu,
        MINING_SPRITE_GROUP,
        0..game.mining.obstacles.len() + WALL_START + 2,
    );
    engine.renderer.sprites.upload_sprites(
        &engine.renderer.gpu,
        TILE_SPRITE_GROUP,
        0..game.collision_objects.len() + 1,
    );
    engine
        .renderer
        .sprites
        .set_camera_all(&engine.renderer.gpu, game.camera);
}
