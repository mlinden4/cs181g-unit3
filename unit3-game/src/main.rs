// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::Key;
use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::f32::RADIX;
use std::os::windows::fs::FileTypeExt;
use std::path::Path;
use std::fs::read_to_string;
use std::{thread, time};

const W: f32 = 320.0;
const H: f32 = 240.0;
const GUY_HORZ_SPEED: f32 = 4.0;
const SPRITE_MAX: usize = 128;
const CATCH_DISTANCE: f32 = 16.0;
const COLLISION_STEPS: usize = 3;
const GRAVITY: f32 = 1.0;
const NO_COLLISION: u16 = 9;

const TOP_HALF_COLLISION: [(u16, u16); 4] = [(0, 3), (1, 3), (2, 3), (3, 3)];
const BOT_HALF_COLLISION: [(u16, u16); 2] = [(0,0), (2,2)];
const DEATH_COLLISION: [(u16, u16); 2] = [(0,0), (2,2)];
const DOOR_COLLISION: [(u16, u16); 6] = [(6,0), (6,1), (6,2), (6,3), (5,3), (5,4)];


// const LEFT: &'static [&'static str] = &["Hello", "World", "!"];

const TILE_SIZE: u16 = 256;
const TILE_SHEET_W: u16 = 7 * TILE_SIZE;
const TILE_SHEET_H: u16 = 5 * TILE_SIZE;

struct Guy {
    pos: Vec2,
    vel: Vec2,
    grounded: bool,
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
    }

    fn die(&mut self){
        self.pos = Vec2 {
            x: W / 2.0,
            y: H / 4.0,
        }
        
    }

}

struct Apple {
    pos: Vec2,
    vel: Vec2,
}

struct SpriteTile {
    collision: AABB,
    tex_coord: (u16, u16),
}

struct Game {
    camera: engine::Camera,
    collision_objects: Vec<SpriteTile>,
    doors: Vec<u16>,
    guy: Guy,
    score: u32,
    font: engine_simple::BitFont,
    level:u32,
}

fn loadLevel(collision_objects: &mut Vec<SpriteTile>, doors: &mut Vec<u16>, num: u16){

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


        if(TOP_HALF_COLLISION.contains(&tex_coord)){
            collision_objects.push(newSpriteTile_Rect(x_pos, y_pos + (size/4.0), size, size/2.0, tex_coord.0, tex_coord.1));
        }else if(BOT_HALF_COLLISION.contains(&tex_coord)){
            collision_objects.push(newSpriteTile_Rect(x_pos, y_pos - (size/4.0), size, size/2.0, tex_coord.0, tex_coord.1));
        }else if(DOOR_COLLISION.contains(&tex_coord)){
            doors.push(collision_objects.len() as u16);
            collision_objects.push(newSpriteTile_Square(x_pos, y_pos, size, tex_coord.0, tex_coord.1));
        }else{
            collision_objects.push(newSpriteTile_Square(x_pos, y_pos, size, tex_coord.0, tex_coord.1));
        }
        
        if(x_pos == 16.0 + size * 9.0){
            x_pos = 16.0;
            y_pos += size;
        }else{
            x_pos += size;
        }
    }
    // collision_objects.reverse();

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

 fn addSpriteTileRow(col_objs: &mut Vec<SpriteTile>, sprite_idxs: Vec<(u16, u16)>, y_pos: f32, size: f32) {
    
    let mut x_pos: f32 = 16.0;
    let x_incr: f32 = 32.0;

    for s_idx in sprite_idxs.iter() {


        if(TOP_HALF_COLLISION.contains(s_idx)){
            col_objs.push(newSpriteTile_Rect(x_pos, y_pos + (size/4.0), size, size/2.0, s_idx.0, s_idx.1));
        }else if(BOT_HALF_COLLISION.contains(s_idx)){
            col_objs.push(newSpriteTile_Rect(x_pos, y_pos - (size/4.0), size, size/2.0, s_idx.0, s_idx.1));
        }else{
            col_objs.push(newSpriteTile_Square(x_pos, y_pos, size, s_idx.0, s_idx.1));
        }
        
        x_pos += x_incr;
    }
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

        let guy = Guy {
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
        loadLevel(&mut collision_objects, &mut doors, 0);


        //              size_x
        //            --------------
        //   size_y   | c_xy x     |
        //            --------------

        
        
        // let row4_sprite_idxs: Vec<(u16,u16)> = vec![(9,9),(9,9),(0,3),(1,3),(2,3),(9,9),(9,9),(3,4),(9,9),(9,9),];
        // let row3_sprite_idxs: Vec<(u16,u16)> = vec![(1,4),(1,1),(9,9),(9,9),(9,9),(9,9),(9,9),(9,9),(9,9),(9,9),];
        // let row2_sprite_idxs: Vec<(u16,u16)> = vec![(3,4),(0,4),(0,4),(0,4),(0,4),(0,4),(0,4),(0,0),(0,0),(0,4),];
        // let row1_sprite_idxs: Vec<(u16,u16)> = vec![(3,4),(3,4),(3,4),(3,4),(3,4),(3,4),(3,4),(1,0),(1,0),(3,4),];

        // addSpriteTileRow(&mut collision_objects, row1_sprite_idxs, 16.0, 32.0);
        // addSpriteTileRow(&mut collision_objects, row2_sprite_idxs, 48.0, 32.0);
        // addSpriteTileRow(&mut collision_objects, row3_sprite_idxs, 80.0, 32.0);
        // addSpriteTileRow(&mut collision_objects, row4_sprite_idxs, 112.0, 32.0);

 
        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 512, 0, 80, 8),
            10,
        );
        
        Game {
            camera,
            guy,
            collision_objects,
            doors,
            // apples: Vec::with_capacity(16),
            // apple_timer: 0,
            score: 0,
            font,
            level: 0,
        }
    }



    fn update(&mut self, engine: &mut Engine) {

        // Character movement ------------------------------------------------------------------------
        let dir_x = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
        let dir_y = engine.input.key_axis(engine::Key::Down, engine::Key::Up);

        self.guy.moveGuy(dir_x, dir_y);
        // Character movement ------------------------------------------------------------------------

        if engine.input.is_key_pressed(engine::Key::R) {
            self.guy.die();
        }

        if engine.input.is_key_pressed(engine::Key::L) {
            self.level = 0;
            loadLevel(&mut self.collision_objects, &mut self.doors, 0);
            self.guy.die();
        }






        // Collision ------------------------------------------------------------------------
        let mut contacts = Vec::with_capacity(self.collision_objects.len());
        // TODO: for multiple guys this might be better as flags on the guy for what side he's currently colliding with stuff on
        for _iter in 0..COLLISION_STEPS {
            let guy_aabb = AABB {
                center: self.guy.pos,
                size: Vec2 { x: 16.0, y: 16.0 },
            };
            contacts.clear();

            // TODO: to generalize to multiple guys, need to iterate over guys first and have guy_index, rect_index, displacement in a contact tuple
            
            contacts.extend(
                    self.collision_objects
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
            if guy_aabb.center.x > 250.0 && guy_aabb.center.x <= 300.0  && self.level == 0{
                if guy_aabb.center.y > 70.0 && guy_aabb.center.y < 150.0{
                    //bottom door collision
                    self.level = 1;
                    self.collision_objects.clear();
                    loadLevel(&mut self.collision_objects, &mut self.doors, 1);
                } else if guy_aabb.center.y < 215.0 && guy_aabb.center.y > 165.0{
                    //top door collision
                    self.level = 3;
                    self.collision_objects.clear();
                    loadLevel(&mut self.collision_objects, &mut self.doors, 3);
                }
            }
            
            if self.level == 1 || self.level == 3{
                // bottom door open
                if engine.input.is_key_pressed(engine::Key::Space) {
                    self.level = 2;
                    self.collision_objects.clear();
                    loadLevel(&mut self.collision_objects, &mut self.doors, 2);
                }
            }
            
            if self.level == 2{
                // mini game 1 
                if engine.input.is_key_pressed(engine::Key::Escape) {
                    self.level = 0;
                    self.guy = Guy {
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
                    self.collision_objects.clear();
                    loadLevel(&mut self.collision_objects, &mut self.doors, 0);
                }
            }
            
            


            for (wall_idx, _disp) in contacts.iter() {
                if !self.doors.contains(&(*wall_idx as u16)){

                    
                    if(self.collision_objects[*wall_idx].tex_coord.0 == NO_COLLISION){
                        continue;
                    }

                    if(DEATH_COLLISION.contains(&self.collision_objects[*wall_idx].tex_coord)){
                        self.guy.die();
                    }

                    

                    // TODO: for multiple guys should access self.guys[guy_idx].
                    let guy_aabb = AABB {
                        center: self.guy.pos,
                        size: Vec2 { x: 16.0, y: 16.0 },
                    };
                    let wall = self.collision_objects[*wall_idx].collision;


                    let mut disp = wall.displacement(guy_aabb).unwrap_or(Vec2::ZERO);
                    
                    // We got to a basically zero collision amount
                    if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                        break;
                    }
                    

                    // Guy is below wall, push down
                    if self.guy.pos.y < wall.center.y {
                        disp.y *= -1.0;
                    } else if self.guy.pos.y > wall.center.y {
                        disp.y *= 1.0;
                    }

                    // Guy is left of wall, push left
                    if self.guy.pos.x < wall.center.x {
                        disp.x *= -1.0;

                    // Guy is right of wall, push left
                    } else if self.guy.pos.x > wall.center.x {
                        disp.x *= 1.0;
                    }

                    
                    if disp.y.abs() <= disp.x.abs(){
                        
                        // Guy is above wall, push up

                        self.guy.pos.y += disp.y;
                        self.guy.vel.y = 0.0;

                        if(self.guy.vel.y <= 0.0 && disp.y > 0.0) {
                            self.guy.grounded = true;
                        }

                        
                        // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                    }else if  disp.x.abs() <= disp.y.abs() {
                        self.guy.pos.x += disp.x;
                        self.guy.vel.x = 0.0;
                        
                        // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                    }

                    // println!("x vel: {}, y vel: {}", self.guy.vel.x , self.guy.vel.y );
                }    

                
            }
        }
        // Collision ------------------------------------------------------------------------





        
        
    }
    
    
    
    fn render(&mut self, engine: &mut Engine) {
        
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
        let guy_idx = WALL_START + self.collision_objects.len();
        for (wall, (trf, uv)) in self.collision_objects.iter().zip(
            trfs1[WALL_START..guy_idx]
                .iter_mut()
                .zip(uvs1[WALL_START..guy_idx].iter_mut()),
        ) {
            *trf = (wall.collision).into();
            *uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &wall.tex_coord, 12, TILE_SIZE);
        }

        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(DEMO_SPRITE_GROUP);

        // set guy
        trfs[guy_idx] = AABB {
            center: self.guy.pos,
            size: Vec2 { x: 16.0, y: 16.0 },
        }
        .into();
        // TODO animation frame
        uvs[guy_idx] = getSpriteFromSheet_Demo(DEMO_SPRITE_GROUP as u16, 16, 64, 8, 70, 70);
        // SheetRegion::new(0, 16, 480, 8, 16, 16);
        
        

        
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, DEMO_SPRITE_GROUP, 0..WALL_START + self.collision_objects.len() + 1);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, TILE_SPRITE_GROUP, 0..self.collision_objects.len() + 1);
    
        engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
    }
}
fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}