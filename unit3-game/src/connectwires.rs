
use engine::Key;
use engine_simple as engine;
use engine_simple::wgpu;
use engine_simple::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::f32::RADIX;
use std::ops::Index;
// use std::os::windows::fs::FileTypeExt;
use std::path::Path;
use std::fs::read_to_string;
use std::{thread, time};
use std::collections::HashMap;
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1


use crate::{SpriteTile, Game, getSpriteFromSheet, getSpriteFromSheet_Demo, GameMode};

const W: f32 = 320.0;
const H: f32 = 240.0;

const TILE_SIZE: u16 = 16;

#[cfg(target_os = "macos")]
const SCALING_FACTOR: f32 = 5.0;
#[cfg(target_os = "windows")]
const SCALING_FACTOR: f32 = 2.5;
#[derive(Debug, PartialEq, Eq, Copy, Clone, EnumIter)]
pub enum Color{
    Pink,
    Green,
    Blue,
    Orange,
    Purple
}
pub struct ConnectWiresState {
    //pub squares: Vec<SpriteTile>, //Vec of sprites 
    pub squares: Vec<(SpriteTile, f32)>, //Vec of squares
    pub palette: Vec<(SpriteTile, f32)>, //Vec of color palette circles
    pub correct: Vec<Vec<i16>>, // number of correctly clicked squares
    pub color: Color,
    pub grid: Vec<(f32, f32)>, // the vector contains the indicies of squares that are correctly placed
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
fn get_pos(start: f32, size: i32, index: i32) -> f32{
    return (index*size) as f32 + start;
}
fn sum_correct(correct: &Vec<Vec<i16>>) -> i16 {
    let mut sum:i16 = 0;
    for i in 0..correct.len(){
        for j in 0.. correct[i].len(){
            sum += correct[i][j];
            println!("SUM FUNCT: {}", sum);

        }
    }
    return sum;
}
fn index_match(sprite: &(SpriteTile, f32), state: &ConnectWiresState, index:i16) -> bool{
    //Check if the sprite tile sent has the index asked for using the grid of sprite locations from the game state
    if state.grid[index as usize].0 == sprite.0.collision.center.x &&
        sprite.0.collision.center.y == state.grid[index as usize].1 {
            return true;
        }
    return false;
}
fn palette_match(sprite: &(SpriteTile, f32)) -> bool{
    //palette is at y value of H-10.0
    if sprite.0.collision.center.y == H-10.0{
            return true;
    }
    return false;
}
pub fn initialize() -> ConnectWiresState{
    let mut x_pos = 90.0;
    let mut y_pos = 190.0;
    let size = 35;

    let mut palette = Vec::default();
    // push palette at top
    //pink
    palette.push((newSpriteTile_Square(W/2.0 -40.0, H-10.0,  16.0, 10, 4), 0.0));  
    // green
    palette.push((newSpriteTile_Square(W/2.0 -20.0, H-10.0,  16.0, 9, 5), 0.0));  
    //blue
    palette.push((newSpriteTile_Square(W/2.0, H-10.0,  16.0, 11, 4), 0.0));    
    //orange
    palette.push((newSpriteTile_Square(W/2.0 + 20.0, H-10.0,  16.0, 11, 5), 0.0));   
    //purple
    palette.push((newSpriteTile_Square(W/2.0 + 40.0, H-10.0,  16.0, 10, 5), 0.0));   

    // push end points
    //pink (1,1) (3,1)
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 1), get_pos(y_pos, -1*size, 1),  size as f32- 10.0, 10, 4), 0.0));
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 3), get_pos(y_pos, -1*size, 1),  size as f32- 10.0, 10, 4), 0.0));

    //green (1,4) (3,3)   
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 1), get_pos(y_pos, -1*size, 4),  size as f32- 10.0, 9, 5), 0.0));
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 3), get_pos(y_pos, -1*size, 3),  size as f32- 10.0, 9, 5), 0.0));
    // blue (1,3) (3,4)
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 1), get_pos(y_pos, -1*size, 3),  size as f32- 10.0, 11, 4), 0.0));
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 3), get_pos(y_pos, -1*size, 4),  size as f32- 10.0, 11, 4), 0.0));
    
    //orange (0,1) (4,1)
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 0), get_pos(y_pos, -1*size, 1),  size as f32- 10.0, 11, 5), 0.0));
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 4), get_pos(y_pos, -1*size, 1),  size as f32- 10.0, 11, 5), 0.0));
    
    //purple (0,2) (0,4)
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 0), get_pos(y_pos, -1*size, 2),  size as f32- 10.0, 10, 5), 0.0));
    palette.push((newSpriteTile_Square(get_pos(x_pos, size, 0), get_pos(y_pos, -1*size, 4),  size as f32- 10.0, 10, 5), 0.0));
    

    // make squares vector of background tiles
    let mut squares = Vec::default();
    let mut grid = Vec::default();


    let mut i = 0;
    let mut j = 0;
    for i in 0..5 {
        for j in 0..5 {
            squares.push((newSpriteTile_Square(x_pos, y_pos,  size as f32 - 5.0, 1, 1), 0.0)); 
            grid.push((x_pos, y_pos));
            x_pos += size as f32;
        }
        x_pos = 90.0;
        y_pos -= size as f32;
    }
    let mut correct:Vec<Vec<i16>> = vec![];
    for i in 0..5{
        correct.push(vec![]);
    }
    
    let color:Color = Color::Pink;
    ConnectWiresState { 
        squares,
        palette,
        correct,
        color,
        grid,
    }
}


pub fn update_connect_wires(game: &mut Game, engine: &mut Engine){
    if sum_correct(&game.connect_wires.correct) == 164 {
        // perform game won logic
        println!("GAME WON!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        game.mode = GameMode::Platformer;
        render_connect_wires(game, engine);
        return;
    }else{
        println!("num correct: {}",sum_correct(&game.connect_wires.correct));
    }
    if  engine.input.is_key_pressed(engine::Key::Escape) {
        // game intentionally abandoned
        game.mode = GameMode::Platformer;
        render_connect_wires(game, engine);
        return;
    }
    if engine.input.is_mouse_pressed(winit::event::MouseButton::Left) {
        let mut remove: Vec<usize> = Vec::default();
        //let mut remove: Vec<&(SpriteTile, f32)> = Vec::default();

        let window_height = engine.renderer.gpu.config.height as f32;
        let window_witdh = engine.renderer.gpu.config.width as f32;

        let mouse_pos = engine.input.mouse_pos();
        // Normalize mouse clicks to be 00 at bottom left corner
        let (x_norm, y_norm) = ((mouse_pos.x as f32 + game.camera.screen_pos[0])/SCALING_FACTOR,
                                (((mouse_pos.y as f32 - window_height) * (-1.0 as f32)) + game.camera.screen_pos[1])/SCALING_FACTOR);
        let mut i = 0;
        let indicies: Vec<Vec<i16>> = vec![vec![7], vec![17,22], vec![11,12, 13, 14, 19, 24], vec![0,1,2,3,4], vec![15]];
        let color_locs: Vec<(u16,u16)> = vec![(10,2), (9,3), (11,2), (11,3), (10,3)];

        for ss_object in game.connect_wires.squares.iter() {
            if(ss_object.0.collision.contains(x_norm, y_norm)){
                if ss_object.0.tex_coord.0 == 1 && ss_object.0.tex_coord.1 == 1{
                    for color in Color::iter(){
                        if game.connect_wires.color == color {
                            game.connect_wires.palette.push((newSpriteTile_Square(
                                ss_object.0.collision.center.x, 
                                ss_object.0.collision.center.y, 
                                30.0, 
                                color_locs[i].0, 
                                color_locs[i].1), 0.0));
                            for j in 0..indicies[i].len(){
                                if !game.connect_wires.correct[i].contains(&indicies[i][j]) && index_match(ss_object, &game.connect_wires, indicies[i][j]){
                                    game.connect_wires.correct[i].push(indicies[i][j]);
                                    println!("CORRECT AT INDEX {}", indicies[i][j])
                                }
                            }
                            // println!("checking color {:?}", color);
                        }
                        i += 1;
                    }
                }
                println!{"({},{})", ss_object.0.tex_coord.0, ss_object.0.tex_coord.1};
            }else{
                // println!("no selection");
            }
            let palette_locs: Vec<(u16,u16)> = vec![(10,4), (9,5), (11,4), (11,5), (10,5)];
            let colors = vec![Color::Pink, Color::Green, Color::Blue, Color::Orange, Color::Purple];
            let mut ind:usize = 0;
            for ss_object in game.connect_wires.palette.iter() {
                if(ss_object.0.collision.contains(x_norm, y_norm)){
                    // check for color changes or overlap with endpoints
                    for i in 0..palette_locs.len(){
                        if ss_object.0.tex_coord.0 == palette_locs[i].0 &&  // check if sprite is a circle tex
                        ss_object.0.tex_coord.1 == palette_locs[i].1 && // check if sprite y is a circle tex
                        palette_match(ss_object) // check sprite loc on screen to see if in pallete
                        {
                            game.connect_wires.color = colors[i];
                        }else if !palette_match(ss_object) &&
                        ss_object.0.tex_coord.1 < 4 && // not a circle (above circle on sprite sheet)
                        ss_object.0.tex_coord.0 > 8 // not a white square
                        { 
                            // clicked on an existing square, need to hide previous square so we will remove it after loop
                            // if remove.is_empty(){
                            //     remove.push(ind);
                            //     println!("Removing square")
                            // }
                        }else{
                            // TO DO determin behavior when endpoint clicked

                        }
                    }
                
                }
                ind += 1;
            }
            
        }
        for to_remove in remove.iter(){
            game.connect_wires.palette[*to_remove].0.collision.size.x = 0.0;
            game.connect_wires.palette[*to_remove].0.collision.size.y = 0.0;

            //game.connect_wires.palette.remove(remove[*to_remove]);
        }

    }

    if engine.input.is_key_pressed(engine::Key::S) {

    }


}

pub fn render_connect_wires(game: &mut Game, engine: &mut Engine) {
    let DEMO_SPRITE_GROUP = 0;
    let TILE_SPRITE_GROUP = 1;
    let SIMON_SAYS_SPRITE_GROUP = 2;
    let CONNECT_WIRES_SPRITE_GROUP = 3;

    // [idx 0, idx 1..guy_idx, guy_idx, apple_start..)]
    // [Background, walls..., guy, apples...]


    // set bg image
    let (trfs1, uvs1) = engine.renderer.sprites.get_sprites_mut(CONNECT_WIRES_SPRITE_GROUP);
    trfs1[0] = AABB::new(W /2.0, H /2.0, W, H).into();  // Create a non-collision AABB for use in the background
    uvs1[0] = getSpriteFromSheet(CONNECT_WIRES_SPRITE_GROUP as u16, &(131,62), 8, 1);

    
    
    // let (trfs2, uvs2) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);
    // set palette
    const PALETTE_START: usize = 1;
    game.connect_wires.palette.reverse();
    for (ss_object, (trf, uv)) in game.connect_wires.palette.iter().zip(
        trfs1[PALETTE_START..]
            .iter_mut()
            .zip(uvs1[PALETTE_START..].iter_mut()),
    ) {
        *trf = ss_object.0.collision.to_transform_rot(ss_object.1);
        *uv = getSpriteFromSheet(CONNECT_WIRES_SPRITE_GROUP as u16, &ss_object.0.tex_coord, 1, 17);
    }

    // set bkgd squares
    let SQUARE_START: usize = game.connect_wires.palette.len()+1;

    for (ss_object, (trf, uv)) in game.connect_wires.squares.iter().zip(
        trfs1[SQUARE_START..]
            .iter_mut()
            .zip(uvs1[SQUARE_START..].iter_mut()),
    ) {
        *trf = ss_object.0.collision.to_transform_rot(ss_object.1);
        *uv = getSpriteFromSheet(CONNECT_WIRES_SPRITE_GROUP as u16, &ss_object.0.tex_coord, 2, 17);
    }

    if !matches!(game.mode, GameMode::ConnectWires) {
        trfs1.fill(Transform::zeroed());
    }


    engine
        .renderer
        .sprites
        .upload_sprites(&engine.renderer.gpu, 
            CONNECT_WIRES_SPRITE_GROUP, 
            0..PALETTE_START + game.connect_wires.squares.len() + game.connect_wires.palette.len() + 1);
    // engine
    //     .renderer
    //     .sprites
    //     .upload_sprites(&engine.renderer.gpu, SIMON_SAYS_SPRITE_GROUP, 0..0);
    engine
        .renderer
        .sprites
        .set_camera_all(&engine.renderer.gpu, game.camera);
    // let DEMO_SPRITE_GROUP = 0;
    // let TILE_SPRITE_GROUP = 1;
    // let OTHER_SPRITE_GROUP = 2;
    // let CONNECT_WIRES_SPRITE_GROUP = 3;


    // // [idx 0, idx 1..guy_idx, guy_idx, apple_start..)]
    // // [Background, walls..., guy, apples...]


    // // set bg image
    // let (trfs1, uvs1) = engine.renderer.sprites.get_sprites_mut(OTHER_SPRITE_GROUP);
    // trfs1[0] = AABB::new(W /2.0, H /2.0, W, H).into();  // Create a non-collision AABB for use in the background
    // uvs1[0] = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &(3,0), 16, TILE_SIZE);

    
    
    // // let (trfs2, uvs2) = engine.renderer.sprites.get_sprites_mut(TILE_SPRITE_GROUP);

    // // set walls
    // // const WALL_START: usize = 1;

    // // for (ss_object, (trf, uv)) in game.connect_wires_objects.iter().zip(
    // //     trfs1[WALL_START..]
    // //         .iter_mut()
    // //         .zip(uvs1[WALL_START..].iter_mut()),
    // // ) {
    // //     *trf = (ss_object.collision).into();
    // //     *uv = getSpriteFromSheet(TILE_SPRITE_GROUP as u16, &ss_object.tex_coord, 12, TILE_SIZE);
    // // }


    // // if !matches!(game.mode, GameMode::SimonSays) {
    // //     trfs1.fill(Transform::zeroed());
    // // }

    
    // engine
    //     .renderer
    //     .sprites
    //     .upload_sprites(&engine.renderer.gpu, CONNECT_WIRES_SPRITE_GROUP, 0..81 + 1);
    // // engine
    // //     .renderer
    // //     .sprites
    // //     .upload_sprites(&engine.renderer.gpu, CONNECT_WIRES_SPRITE_GROUP, 0..0);
    // engine
    //     .renderer
    //     .sprites
    //     .set_camera_all(&engine.renderer.gpu, game.camera);
}