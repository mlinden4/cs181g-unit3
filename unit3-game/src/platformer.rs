
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
// use bytemuck::{Pod, Zeroable};

use crate::{SpriteTile, Game, getSpriteFromSheet, getSpriteFromSheet_Demo, GameMode, newSpriteTile_Rect, newSpriteTile_Square};

const W: f32 = 320.0;
const H: f32 = 240.0;
const GUY_HORZ_SPEED: f32 = 2.5; //4.0
const COLLISION_STEPS: usize = 3;
const GRAVITY: f32 = 1.0;
const NO_COLLISION: u16 = 9;

const TOP_HALF_COLLISION: [(u16, u16); 4] = [(0, 3), (1, 3), (2, 3), (3, 3)];
const BOT_HALF_COLLISION: [(u16, u16); 2] = [(0,0), (2,2)];
const DEATH_COLLISION: [(u16, u16); 2] = [(0,0), (2,2)];
const DOOR_COLLISION: [(u16, u16); 6] = [(6,0), (6,1), (6,2), (6,3), (5,3), (5,4)];

// Each sprite is 128.5 wide and 130 tall but lots of blank space around character.
// index 0-7 walking to the right
// index 8-15 walking to the left
// index 16-20 is idle
const GUY_FRAMES: [(u16, u16); 26]= [(0, 390), (128, 390), (256, 390), (384, 390), (512, 390), (640, 390), (769, 390), (898, 390), 
                                    (0, 520), (128, 520), (256, 520), (385, 520), (513, 520), (641, 520), (769, 520), (898, 520),
                                    (0, 260), (0, 260), (128, 260), (128, 260), (256, 260), (256, 260), (385, 260), (385, 260), (0, 260), (0, 260)];

// const LEFT: &'static [&'static str] = &["Hello", "World", "!"];

const TILE_SIZE: u16 = 256;


pub struct Guy {
    pub pos: Vec2,
    pub vel: Vec2,
    pub grounded: bool,
    pub frame: usize,
}


impl Guy {

    fn doGravity(&mut self, ) {
        if self.vel.y >= -10.0 {            
            self.vel.y -= GRAVITY;
        }
        
    }

    fn setHorzVel(&mut self, direction:f32) {
        self.vel.x = direction * GUY_HORZ_SPEED;
    }

    fn handle_jump(&mut self, vert_dir: f32) {
        if vert_dir > 0.0 && self.grounded {
            self.vel.y = 10.0;
            self.grounded = false;
        }
        
    }

    fn moveGuy(&mut self, horz_dir: f32, vert_dir: f32) {
        
        //Handle velocities
        self.setHorzVel(horz_dir);
        self.handle_jump(vert_dir);
        self.doGravity();
        
        // Update positon
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;

        //Handle animation
        if horz_dir == 0.0 || vert_dir != 0.0{
            // use idle frames when still or jumping
            // delay frames are frames to make the idle look better after 20
            if self.frame == 25 || self.frame < 16{
                // restart animation
                self.frame = 16;
            }else{
                self.frame += 1;
            }
        }else if horz_dir > 0.0 {
            // if walking to the right
            if self.frame == 7 || self.frame > 8 {
                // if frames recycle OR was not previously walking right
                self.frame = 0;
            }else{
                self.frame += 1;
            }
        }else if horz_dir < 0.0{
            // if walking to the left
            if self.frame == 15 || self.frame > 15 || self.frame < 7{
                // if frames recycle OR was not previously walking left
                self.frame = 8;
            }else{
                self.frame += 1;
            }
        }
    }

    fn die(&mut self){
        self.pos = Vec2 {
            x: W / 2.0,
            y: H / 4.0,
        }
        
    }

}





pub fn loadLevel(collision_objects: &mut Vec<SpriteTile>, doors: &mut Vec<u16>, num: u16){

    let mut x_pos: f32 = 16.0;
    let mut y_pos: f32 = 16.0;
    let size: f32 = 32.0;
    // let incr: f32 = 32.0;

    let binding = read_to_string(format!("content/Levels/Level{}.txt", num))
        .unwrap();
    // let str_tex_coords = binding
    //     .split(|c| c == ' ')
    //     .filter(|&s| !s.is_empty());

    let segments: Vec<&str> = binding.split(|c| c == ' ' || c == '\n' || c == '(' || c == ')')
        .filter(|s| !s.is_empty())
        .collect();

    for segment in segments.iter() {
        println!("{}", segment);
    }

    // let tex_coords: Vec<(u16,u16)> = segments.iter()
    //     .for_each(|tex_coord| tex_coord.split(",") );

    let mut tex_coords: Vec<(u16,u16)> = Vec::default();

    for str_tex_coord in segments {
        let s: String = String::from(str_tex_coord);
        if s.chars().nth(0).unwrap().is_ascii_digit(){
            println!("tex_coords: {}, {}", s.chars().nth(0).unwrap(), s.chars().nth(2).unwrap());
            tex_coords.push((s.chars().nth(0).unwrap() as u16 - 48, s.chars().nth(2).unwrap() as u16 - 48));
        }
        
    }
    println!("{:?}", tex_coords);


    for tex_coord in tex_coords {


        if TOP_HALF_COLLISION.contains(&tex_coord) {
            collision_objects.push(newSpriteTile_Rect(x_pos, y_pos + (size/4.0), size, size/2.0, tex_coord.0, tex_coord.1));
        }else if BOT_HALF_COLLISION.contains(&tex_coord) {
            collision_objects.push(newSpriteTile_Rect(x_pos, y_pos - (size/4.0), size, size/2.0, tex_coord.0, tex_coord.1));
        }else if DOOR_COLLISION.contains(&tex_coord) {
            doors.push(collision_objects.len() as u16);
            collision_objects.push(newSpriteTile_Square(x_pos, y_pos, size, tex_coord.0, tex_coord.1));
        }else{
            collision_objects.push(newSpriteTile_Square(x_pos, y_pos, size, tex_coord.0, tex_coord.1));
        }
        
        if x_pos == 16.0 + size * 9.0 {
            x_pos = 16.0;
            y_pos += size;
        }else{
            x_pos += size;
        }
    }
    // collision_objects.reverse();

}

pub fn update_platformer(game: &mut Game, engine: &mut Engine){

        // Character movement ------------------------------------------------------------------------
        let dir_x = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
        let dir_y = engine.input.key_axis(engine::Key::Down, engine::Key::Up);
        println!("dirx: {}, diry: {}", dir_x, dir_y);
        game.guy.moveGuy(dir_x, dir_y);
        // Character movement ------------------------------------------------------------------------

        if engine.input.is_key_pressed(engine::Key::R) {
            game.guy.die();
        }

        if engine.input.is_key_pressed(engine::Key::L) {
            game.level = 0;
            loadLevel(&mut game.collision_objects, &mut game.doors, 0);
            game.guy.die();
        }

        if engine.input.is_key_pressed(engine::Key::S) {
            game.mode = GameMode::SimonSays;
            if !matches!(game.mode, GameMode::Platformer) {
                // game.camera.screen_pos = [500.0, 500.0];
                println!("here");
                render_platformer(game, engine);
            }
        }


        // Collision ------------------------------------------------------------------------
        let mut contacts = Vec::with_capacity(game.collision_objects.len());
        // TODO: for multiple guys this might be better as flags on the guy for what side he's currently colliding with stuff on
        for _iter in 0..COLLISION_STEPS {
            let guy_aabb = AABB {
                center: game.guy.pos,
                size: Vec2 { x: 16.0, y: 16.0 },
            };
            contacts.clear();

            // TODO: to generalize to multiple guys, need to iterate over guys first and have guy_index, rect_index, displacement in a contact tuple
            
            contacts.extend(
                    game.collision_objects
                    .iter()
                    .enumerate()
                    .filter_map(|(ri, w)| w.collision.displacement(guy_aabb).map(|d| (ri, d)))
            );

            if contacts.is_empty() {
                break;
            }
            // This part stays mostly the same for multiple guys, except the shape of contacts is different
            contacts.sort_by(|(_r1i, d1), (_r2i, d2)| {
                d2.length_squared()
                    .partial_cmp(&d1.length_squared())
                    .unwrap()
            });
            let x_tex_region = (guy_aabb.center.x);//TILE_SIZE as f32) as u16;
            println!("{}, {}", x_tex_region, guy_aabb.center.y);
            // check if guy collides with doors
            if guy_aabb.center.x > 250.0 && guy_aabb.center.x <= 300.0  && game.level == 0{
                if guy_aabb.center.y > 70.0 && guy_aabb.center.y < 150.0{
                    //bottom door collision
                    game.level = 1;
                    game.collision_objects.clear();
                    loadLevel(&mut game.collision_objects, &mut game.doors, 1);
                } else if guy_aabb.center.y < 215.0 && guy_aabb.center.y > 165.0{
                    //top door collision
                    game.level = 3;
                    game.collision_objects.clear();
                    loadLevel(&mut game.collision_objects, &mut game.doors, 3);
                } 
            }
            
            if game.level == 1 {
                if engine.input.is_key_pressed(engine::Key::Space) {
                    game.mode = GameMode::SimonSays;
                    render_platformer(game, engine);
                    return;
                }else if guy_aabb.center.x < 250.0 {
                    //door close, guy left doorway
                    game.level = 0;
                    game.collision_objects.clear();
                    loadLevel(&mut game.collision_objects, &mut game.doors, 0);
                }
            }

            if game.level == 3 {
                // bottom door open
                if engine.input.is_key_pressed(engine::Key::Space) {
                    game.level = 2;
                    game.collision_objects.clear();
                    loadLevel(&mut game.collision_objects, &mut game.doors, 2);
                }else if guy_aabb.center.x < 250.0 {
                    //door close, guy left doorway
                    game.level = 0;
                    game.collision_objects.clear();
                    loadLevel(&mut game.collision_objects, &mut game.doors, 0);
                }
            }
            
            if game.level == 2{
                // mini game 1 
                if engine.input.is_key_pressed(engine::Key::Escape) {
                    game.level = 0;
                    game.guy = Guy {
                        pos: Vec2 {
                            x: W / 2.0,
                            y: H / 2.0,
                        },
                        vel: Vec2 {
                            x: 0.0,
                            y: 0.0,
                        },
                        grounded: false,
                        frame: 0,
                    };
                    game.collision_objects.clear();
                    loadLevel(&mut game.collision_objects, &mut game.doors, 0);
                }
            }
            
            


            for (wall_idx, _disp) in contacts.iter() {
                if !game.doors.contains(&(*wall_idx as u16)){

                    
                    if game.collision_objects[*wall_idx].tex_coord.0 == NO_COLLISION {
                        continue;
                    }

                    if DEATH_COLLISION.contains(&game.collision_objects[*wall_idx].tex_coord) {
                        game.guy.die();
                    }

                    

                    // TODO: for multiple guys should access game.guys[guy_idx].
                    let guy_aabb = AABB {
                        center: game.guy.pos,
                        size: Vec2 { x: 16.0, y: 16.0 },
                    };
                    let wall = game.collision_objects[*wall_idx].collision;


                    let mut disp = wall.displacement(guy_aabb).unwrap_or(Vec2::ZERO);
                    
                    // We got to a basically zero collision amount
                    if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                        break;
                    }
                    

                    // Guy is below wall, push down
                    if game.guy.pos.y < wall.center.y {
                        disp.y *= -1.0;
                    } else if game.guy.pos.y > wall.center.y {
                        disp.y *= 1.0;
                    }

                    // Guy is left of wall, push left
                    if game.guy.pos.x < wall.center.x {
                        disp.x *= -1.0;

                    // Guy is right of wall, push left
                    } else if game.guy.pos.x > wall.center.x {
                        disp.x *= 1.0;
                    }

                    
                    if disp.y.abs() <= disp.x.abs(){
                        
                        // Guy is above wall, push up

                        game.guy.pos.y += disp.y;
                        game.guy.vel.y = 0.0;

                        if game.guy.vel.y <= 0.0 && disp.y > 0.0 {
                            game.guy.grounded = true;
                        }

                        
                        // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                    }else if disp.x.abs() <= disp.y.abs() {
                        game.guy.pos.x += disp.x;
                        game.guy.vel.x = 0.0;
                        
                        // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                    }

                    // println!("x vel: {}, y vel: {}", game.guy.vel.x , game.guy.vel.y );
                }    

                
            }
        }
        // Collision ------------------------------------------------------------------------
}

pub fn render_platformer(game: &mut Game, engine: &mut Engine) {

    let DEMO_SPRITE_GROUP = 0;
    let TILE_SPRITE_GROUP = 1;


    // [idx 0, idx 1..guy_idx, guy_idx, apple_start..)]
    // [Background, walls..., guy, apples...]


    // set bg image
    let (trfs1, uvs1) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);
    trfs1[0] = AABB::new(W /2.0, H /2.0, W, H).into();  // Create a non-collision AABB for use in the background
    uvs1[0] = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &(0,1), 16, TILE_SIZE);

    
    
    // let (trfs2, uvs2) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);

    // set walls
    const WALL_START: usize = 1;
    let guy_idx = WALL_START + game.collision_objects.len();
    for (wall, (trf, uv)) in game.collision_objects.iter().zip(
        trfs1[WALL_START..guy_idx]
            .iter_mut()
            .zip(uvs1[WALL_START..guy_idx].iter_mut()),
    ) {
        *trf = (wall.collision).into();
        *uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &wall.tex_coord, 12, TILE_SIZE);
    }

    if !matches!(game.mode, GameMode::Platformer) {
        println!("here render 1");
        trfs1.fill(Transform::zeroed());
    }

    let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(DEMO_SPRITE_GROUP);

    // set guy
    trfs[guy_idx] = AABB {
        center: game.guy.pos + 3.0,
        size: Vec2 { x: 32.0, y: 32.0 },
    }
    .into();
    // TODO animation frame
    // let mut x_off: u16 = 24;
    // if game.guy.frame < 16{
    //     // guy is walking, size is 30 x 120
    //     x_off = 44;
    // }
    uvs[guy_idx] = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, GUY_FRAMES[game.guy.frame].0 + 24, GUY_FRAMES[game.guy.frame].1 + 44, 8, 100, 100);

    //uvs[guy_idx] = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, GUY_FRAMES[game.guy.frame].0, GUY_FRAMES[game.guy.frame].0, 8, 128, 130);
    // SheetRegion::new(0, 16, 480, 8, 16, 16);
    

    if !matches!(game.mode, GameMode::Platformer) {
        trfs.fill(Transform::zeroed());
    }

    
    engine
        .renderer
        .sprites
        .upload_sprites(&engine.renderer.gpu, DEMO_SPRITE_GROUP, 0..WALL_START + game.collision_objects.len() + 1);
    engine
        .renderer
        .sprites
        .upload_sprites(&engine.renderer.gpu, TILE_SPRITE_GROUP, 0..game.collision_objects.len() + 1);
    engine
        .renderer
        .sprites
        .set_camera_all(&engine.renderer.gpu, game.camera);
}