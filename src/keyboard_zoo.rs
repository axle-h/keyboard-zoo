use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use rand::{Rng, thread_rng};
use crate::build_info::nice_app_name;
use crate::config::{Config, VideoMode};
use crate::frame_rate::FrameRate;
use crate::icon::app_icon;
use sdl2::image::{InitFlag as ImageInitFlag, Sdl2ImageContext};
use sdl2::mixer::{InitFlag as MixerInitFlag, DEFAULT_CHANNELS, DEFAULT_FORMAT};
use sdl2::pixels::Color;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::sys::mixer::MIX_CHANNELS;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::{AudioSubsystem, EventPump, Sdl};
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use crate::animate::Animations;
use crate::animate::event::AnimationEvent;
use crate::characters::render::CharacterRender;
use crate::assets::sound::Sound;
use crate::assets::sprites::Sprites;
use crate::game::action::Direction;
use crate::game::event::GameEvent;
use crate::game::{Game, game};
use crate::game::physics::Body;
use crate::game::scale::PhysicsScale;
use crate::game_input::{GameInputContext, GameInputKey};
use crate::{characters, particles};
use crate::particles::Particles;
use crate::particles::prescribed::{fireworks, orbit, sprite_lattice_source, sprite_triangle_source};
use crate::particles::render::ParticleRender;
use crate::particles::source::ParticleSource;

const MAX_FOREGROUND_PARTICLES: usize = 200000;
const MAX_BACKGROUND_PARTICLES: usize = 200000;

pub struct KeyboardZoo {
    config: Config,
    _sdl: Sdl,
    ttf: Sdl2TtfContext,
    _image: Sdl2ImageContext,
    canvas: Rc<RefCell<WindowCanvas>>,
    texture_creator: TextureCreator<WindowContext>,
    event_pump: EventPump,
    _audio: AudioSubsystem,
    particle_scale: particles::scale::Scale,
}

impl KeyboardZoo {
    pub fn new() -> Result<Self, String> {
        let config = Config::load()?;
        let sdl = sdl2::init()?;
        let image = sdl2::image::init(ImageInitFlag::PNG)?;
        let video = sdl.video()?;
        let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;

        if config.video.disable_screensaver && video.is_screen_saver_enabled() {
            video.disable_screen_saver();
        }

        let (width, height) = match config.video.mode {
            VideoMode::Window { width, height } => (width, height),
            VideoMode::FullScreen { width, height } => (width, height),
            _ => (1, 1),
        };

        let mut window_builder = video.window(&nice_app_name(), width, height);
        match config.video.mode {
            VideoMode::FullScreen { .. } => {
                window_builder.fullscreen();
            }
            VideoMode::FullScreenDesktop => {
                window_builder.fullscreen_desktop();
            }
            _ => {}
        };

        let mut window = window_builder
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        window.set_icon(app_icon()?);

        let canvas_builder = window.into_canvas().target_texture().accelerated();

        let mut canvas = if config.video.vsync {
            canvas_builder.present_vsync()
        } else {
            canvas_builder
        }
        .build()
        .map_err(|e| e.to_string())?;

        let event_pump = sdl.event_pump()?;

        let audio = sdl.audio()?;
        sdl2::mixer::open_audio(44_100, DEFAULT_FORMAT, DEFAULT_CHANNELS, 512)?;
        let _mixer_context = sdl2::mixer::init(MixerInitFlag::OGG)?;
        sdl2::mixer::allocate_channels(512);
        sdl2::mixer::Music::set_volume(config.audio.music_volume());

        let texture_creator = canvas.texture_creator();

        Ok(Self {
            config,
            _sdl: sdl,
            ttf,
            _image: image,
            canvas: Rc::new(RefCell::new(canvas)),
            texture_creator,
            event_pump,
            _audio: audio,
            particle_scale: particles::scale::Scale::new((width, height)),
        })
    }

    pub fn run_sandbox(&self) -> bool {
        self.config.input.run_toddler_sandbox
    }

    fn orbit_particle_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.borrow().window().size();
        orbit(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
        )
    }

    fn fireworks_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.borrow().window().size();
        fireworks(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
        )
    }

    pub fn game(&mut self) -> Result<(), String> {
        let mut fg_particles = ParticleRender::new(
            Particles::new(MAX_FOREGROUND_PARTICLES),
            &self.texture_creator,
            self.particle_scale
        )?;

        let mut bg_particles = ParticleRender::new(
            Particles::new(MAX_BACKGROUND_PARTICLES),
            &self.texture_creator,
            self.particle_scale
        )?;

        let mut character_render = CharacterRender::new(&self.texture_creator)?;
        let mut sprites = Sprites::new(&self.texture_creator)?;
        let mut sound = Sound::new(sprites.names(), self.config.audio)?;
        let mut character_sound  = characters::sound(self.config.audio)?;

        let mut inputs = GameInputContext::new(self.config.input);
        let (width, height) = self.canvas.borrow().window().size();
        let scale = PhysicsScale::new(width, height, self.config.physics);
        let mut game = game(scale, self.config.physics, self.canvas.clone());

        let mut animations = Animations::new();

        let mut frame_rate = FrameRate::new();

        fg_particles.clear();
        bg_particles.clear();
        bg_particles.add_source(self.orbit_particle_source());
        //bg_particles.add_source(self.fireworks_source());

        let mut rng = thread_rng();
        sound.play_music()?;
        'game: loop {
            let delta = frame_rate.update()?;

            for key in inputs.update(delta, self.event_pump.poll_iter()) {
                match key {
                    GameInputKey::Up => game.push(Direction::Up),
                    GameInputKey::Down => game.push(Direction::Down),
                    GameInputKey::Left => game.push(Direction::Left),
                    GameInputKey::Right => game.push(Direction::Right),
                    GameInputKey::SpawnAsset(ch) => {
                        sprites.pick_sprite_by_char(ch).map(|sprite| game.spawn_asset(sprite));
                    },
                    GameInputKey::SpawnRandomAsset => game.spawn_asset(sprites.pick_random_sprite()),
                    GameInputKey::SpawnCharacter(character) => game.spawn_character(character),
                    GameInputKey::SpawnRandomCharacter => game.spawn_character(rng.gen()),
                    GameInputKey::Nuke => animations.nuke(game.bodies().into_iter().map(|b| b.id()).collect()),
                    GameInputKey::Explosion => game.explosion(),
                    GameInputKey::Quit => break 'game,
                }
            }

            for event in animations.update(delta) {
                match event {
                    AnimationEvent::DestroyAsset { id } => game.destroy(id)
                }
            }

            let physics_delta = delta.min(Duration::from_millis(32)); // if simulation cannot maintain 30fps then slow it down
            for event in game.update(physics_delta).into_iter() {
                match event {
                    GameEvent::Spawned(body) => {
                        match body {
                            Body::Asset(asset_body) => {
                                sound.play_create(asset_body.asset_name());
                                fg_particles.add_source(sprite_lattice_source(asset_body, &self.particle_scale));
                            }
                            Body::Character(character_body) => {
                                character_sound.play_create(character_body.character().character_type())?;
                            }
                        }
                    }
                    GameEvent::Destroy(body) => {
                        match body {
                            Body::Asset(asset_body) => {
                                for triangle in asset_body.polygons().iter() {
                                    fg_particles.add_source(sprite_triangle_source(*triangle, &self.particle_scale));
                                }
                                sound.play_destroy();
                            }
                            Body::Character(character_body) => {
                                character_sound.play_destroy(character_body.character().character_type())?;
                                // TODO particles?
                            }
                        }

                    }
                    GameEvent::CharacterAttack(character) => {
                        character_sound.play_attack(character)?;
                    }
                    GameEvent::Explosion { x, y } => {
                        fg_particles.add_source(
                            particles::prescribed::explosion((x, y), &self.particle_scale)
                        );
                        sound.play_explosion();
                    }
                }
            }

            // update particles
            fg_particles.update(delta);
            bg_particles.update(delta);

            self.clear();

            if self.config.physics.debug_draw {
                game.debug_draw();
            } else {
                // draw bg particles
                bg_particles.draw(&mut self.canvas.borrow_mut())?;

                self.draw_bodies(&mut sprites, &mut character_render, game.bodies())?;

                // draw fg particles
                fg_particles.draw(&mut self.canvas.borrow_mut())?;
            }

            self.canvas.borrow_mut().present();
        }

        sound.halt_music();
        Ok(())
    }

    fn draw_bodies(&self, sprites: &mut Sprites, character_render: &mut CharacterRender, bodies: Vec<Body>) -> Result<(), String> {
        let mut canvas = self.canvas.borrow_mut();
        for body in bodies.into_iter() {
            match body {
                Body::Asset(asset_body) => {
                    sprites.draw_sprite(&mut canvas, asset_body)?;
                }
                Body::Character(character_body) => {
                    character_render.draw_character(&mut canvas, character_body)?;
                }
            }

        }
        Ok(())
    }

    fn clear(&self) {
        let mut canvas = self.canvas.borrow_mut();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
    }
}