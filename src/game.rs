use web_view::{WVResult, Handle};
use std::f64::consts::PI;
use crate::ctx;
pub static GAME_WIDTH:i32 = 500;
pub static GAME_HEIGHT:i32 = 512;
pub static IMG_BACKGROUND:&str = "img-background";
pub static IMG_BACKGROUND_WIDTH:f64 = 288.;
pub static IMG_BIRD:&str = "img-bird";
pub static IMG_PIPEBOTTOM:&str = "img-pipebottom";
pub static IMG_PIPETOP:&str = "img-pipetop";
pub static IMG_PIPETOP_HEIGHT:f64 = 512.;
use crate::neat::ga::GA;
use crate::neat::phenotype::RunType;

pub struct Bird{
    x: f64,
    y: f64,
    width: f64,
    height: f64,

    alive: bool,
    gravity: f64,
    velocity: f64,
    jump: f64,
}

impl Bird{
    pub fn flap(&mut self){
        self.gravity = self.jump;
    }
    pub fn update(&mut self){
        self.gravity += self.velocity;
        self.y += self.gravity;
    }

    fn is_dead(&self, height: f64, pipes:&Vec<Pipe>) -> bool{
        if self.y >= height || self.y + self.height <= 0.0{
            return true;
        }
        for i in 0..pipes.len(){
            if !(
                self.x > pipes[i].x + pipes[i].width ||
                self.x + self.width < pipes[i].x || 
                self.y > pipes[i].y + pipes[i].height ||
                self.y + self.height < pipes[i].y
                ){
                return true;
            }
        }
        false
    }
}

impl Default for Bird{
    fn default() -> Self{
        Bird{
            x: 80.0,
            y: 250.0,
            width: 40.0,
            height: 30.0,

            alive: true,
            gravity: 0.0,
            velocity: 0.3,
            jump: -6.0,
        }
    }
}

struct Pipe{
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    speed: f64,
}
impl Pipe{
    pub fn update(&mut self){
        self.x -= self.speed;
    }
    pub fn is_out(&self) -> bool{
        self.x + self.width < 0.0
    }
}
impl Default for Pipe{
    fn default() -> Self{
        Pipe{
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 40.0,
            speed: 3.0
        }
    }
}

pub struct Game{
    pipes: Vec<Pipe>,
	birds: Vec<Bird>,
	score: i32,
	width: f64,
	height: f64,
	spawn_interval: i32,
	interval: i32,
	ga: GA,
	alives: usize,
	generation: i32,
	background_speed: f64,
	backgroundx: f64,
	max_score: i32,
    handle: Option<Handle<()>>,
}
impl Default for Game{
    fn default() -> Self{
        Game{
            pipes: vec![],
            birds: vec![],
            score: 0,
            width: GAME_WIDTH as f64,
            height: GAME_HEIGHT as f64,
            spawn_interval: 90,
            interval: 0,
            ga: GA::new(50, 2, 1),
            alives: 0,
            generation: 0,
            background_speed: 0.5,
            backgroundx: 0.0,
            max_score: 0,
            handle:None,
        }
    }
}

impl Game{
    pub fn start(&mut self){
        self.interval = 0;
        self.score = 0;
        self.pipes = vec![];
        self.birds = vec![];

        //下一代
        self.ga.epoch();
        for _ in 0..self.ga.size(){
            self.birds.push(Bird::default());
        }
        self.generation+=1;
        self.alives = self.birds.len();
        // self.set_fps(60);
        if let Some(handle) = self.handle.as_ref(){
            handle.dispatch(move |webview| { webview.eval(&format!("startAnimationFrame();")) }).unwrap();
        }
    }
    
    pub fn set_handle(&mut self, handle:Handle<()>){
        self.handle = Some(handle);
    }

    pub fn has_handle(&self) -> bool{
        self.handle.is_some()
    }

    pub fn update(&mut self){
        self.backgroundx += self.background_speed;
        let mut next_holl = 0.0;
        if self.birds.len() > 0{
            for i in (0..self.pipes.len()).step_by(2){
                if self.pipes[i].x + self.pipes[i].width > self.birds[0].x{
                    next_holl = self.pipes[i].height/self.height;
                    break;
                }
            }
        }

        for i in 0..self.birds.len(){
            if self.birds[i].alive {

                let inputs = [
                    self.birds[i].y / self.height,
                    next_holl
                ];
                
                //网络处理
                let phenotype = self.ga.get_phenotype(i).unwrap();
                let output = phenotype.borrow_mut().update(&inputs, RunType::Active);
                if output[0] > 0.5{
                    self.birds[i].flap();
                }

                self.birds[i].update();
                if self.birds[i].is_dead(self.height, &self.pipes) {
                    self.birds[i].alive = false;
                    self.alives -= 1;
                    //console.log(self.alives);
                    //设置网络得分
                    self.ga.fitness_scores()[i] = self.score as f64;
                    if self.is_it_end(){
                        self.start();
                    }
                }
            }
        }

        for pipe in &mut self.pipes{
            pipe.update();
        }
        self.pipes.retain(|pipe|{
            !pipe.is_out()
        });

        if self.interval == 0{
            let delta_bord = 50.0;
            let pipe_holl = 120.0;
            let holl_position = (rand::random::<f64>() * (self.height - delta_bord * 2.0 - pipe_holl)).round() +  delta_bord;
            self.pipes.push(Pipe{x:self.width, y:0.0, height:holl_position, ..Default::default()});
            self.pipes.push(Pipe{x:self.width, y:holl_position+pipe_holl, height:self.height, ..Default::default()});
        }

        self.interval += 1;
        if self.interval == self.spawn_interval{
            self.interval = 0;
        }

        self.score += 1;
        self.max_score = std::cmp::max(self.score, self.max_score);
    }

    pub fn draw(&self) -> WVResult{
        if self.handle.is_none(){
            return Ok(());
        }
        let handle = self.handle.as_ref().unwrap();
        ctx::clear_rect(handle, 0, 0, self.width as i32, self.height as i32)?;
        for i in 0..((self.width / IMG_BACKGROUND_WIDTH).ceil() + 1.0) as usize{
            ctx::draw_image(handle, IMG_BACKGROUND, (i as f64 * IMG_BACKGROUND_WIDTH - (self.backgroundx%IMG_BACKGROUND_WIDTH).floor()) as i32, 0, None)?;
        }
        
        for i in 0..self.pipes.len(){
            if i%2 == 0{
                ctx::draw_image(handle, IMG_PIPETOP, self.pipes[i].x as i32, (self.pipes[i].y + self.pipes[i].height - IMG_PIPETOP_HEIGHT) as i32, Some((self.pipes[i].width as i32, IMG_PIPETOP_HEIGHT as i32)))?;
            }else{
                ctx::draw_image(handle, IMG_PIPEBOTTOM, self.pipes[i].x as i32, self.pipes[i].y as i32, Some((self.pipes[i].width as i32, IMG_PIPETOP_HEIGHT as i32)))?;
            }
        }

        ctx::fill_style(handle, "#FFC600")?;
        ctx::stroke_style(handle, "#CE9E00")?;
        for i in 0..self.birds.len(){
            if self.birds[i].alive{
                ctx::save(handle)?; 
                ctx::translate(handle, (self.birds[i].x + self.birds[i].width/2.0) as i32, (self.birds[i].y + self.birds[i].height/2.0) as i32)?;
                ctx::rotate(handle, PI/2.0 * self.birds[i].gravity/20.0)?;
                ctx::draw_image(handle, IMG_BIRD, (-self.birds[i].width/2.0) as i32, (-self.birds[i].height/2.0) as i32, Some((self.birds[i].width as i32, self.birds[i].height as i32)))?;
                ctx::restore(handle)?;
            }
        }

        ctx::fill_style(handle, "white")?;
        ctx::font(handle, "20px Oswald, sans-serif")?;
        ctx::fill_text(handle, &format!("Score : {}", self.score), 10, 25)?;
        ctx::fill_text(handle, &format!("Max Score : {}", self.max_score), 10, 50)?;
        ctx::fill_text(handle, &format!("Generation : {}", self.generation), 10, 75)?;
        //生存数量
        //ctx::fill_text(handle, &format!("Alive : {} / {}", self.alives, Neuvol.options.population), 10, 100);
        Ok(())
    }

    pub fn set_fps(&mut self, fps:i32){
        if let Some(handle) = self.handle.as_ref(){
            let script = format!("setFps({});", fps);
            handle.dispatch(move |webview| { webview.eval(&script) }).unwrap();
            // if(FPS == 0){
            //     setZeroTimeout(function(){
            //         self.update();
            //     });
            // }else{
            //     setTimeout(function(){
            //         self.update();
            //     }, 1000/FPS);
            // }
        }
    }

    pub fn is_it_end(&self) -> bool{
        for i in 0..self.birds.len(){
            if self.birds[i].alive{
                return false;
            }
        }
        true
    }
}