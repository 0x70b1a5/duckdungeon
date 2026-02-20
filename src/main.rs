#[warn(clippy::all, clippy::pedantic)]
mod camera;
mod components;
mod map;
mod map_builder;
mod spawner;
mod systems;
mod turn_state;

enum GameMode {
    Menu,
    Playing,
    End,
}

mod prelude {
    pub use bracket_lib::prelude::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;
}

use prelude::*;

struct State {
    mode: GameMode,
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut rng = RandomNumberGenerator::new();
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let map_builder = MapBuilder::new(&mut rng);

        // SPAWN HIM...
        spawn_player(&mut ecs, map_builder.player_start);

        // SPAWN THEM...
        map_builder
            .rooms
            .iter()
            .skip(1)
            .map(|r| r.center())
            .for_each(|pos| spawn_monster(&mut ecs, &mut rng, pos));

        resources.insert(map_builder.map);
        resources.insert(Camera::new(map_builder.player_start));
        resources.insert(TurnState::AwaitingInput);
        Self {
            ecs,
            resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler(),
            mode: GameMode::Menu,
        }
    }
    fn await_death_by_keystroke(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Q => self.mode = GameMode::End,
                _ => {}
            }
        }
    }
    fn restart(&mut self) {
        self.mode = GameMode::Playing;
    }
    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "WELCOME TO DUCK DUNGEON...");
        ctx.print_centered(8, "[P] PLAY");
        ctx.print_centered(9, "[Q] QUIT");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "YOU'RE DINNER, BABY!");
        ctx.print_centered(8, "[P] PLAY AGAIN");
        ctx.print_centered(9, "[Q] QUIT");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.print_color_centered(2, RED, BLACK, "Your Duck was Dungeoned.");
        ctx.print_color_centered(4, WHITE, BLACK, "Slain by a monster, your hero's quest has come to a premature end..");
        ctx.print_color_centered(5, WHITE, BLACK, "The Duckington Amulet was never recovered.");
        ctx.print_color_centered(6, GREEN, BLACK, "Press 1 to play again.");

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.ecs = World::default();
            self.resources = Resources::default();
            let mut rng = RandomNumberGenerator::new();
            let map_builder = MapBuilder::new(&mut rng);
            spawn_player(&mut self.ecs, map_builder.player_start);
            map_builder.rooms.iter()
            .skip(1)
            .map(|r| r.center())
            .for_each(|pos| spawn_monster(&mut self.ecs, &mut rng, pos));

            self.resources.insert(map_builder.map);
            self.resources.insert(Camera::new(map_builder.player_start));
            self.resources.insert(TurnState::AwaitingInput);
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();
        self.resources.insert(ctx.key);
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => {
                let current_state =
                    self.resources.get::<TurnState>().unwrap().clone();
                match current_state {
                    TurnState::AwaitingInput => {
                        self.input_systems
                            .execute(&mut self.ecs, &mut self.resources);
                        self.await_death_by_keystroke(ctx);
                    }
                    TurnState::PlayerTurn => {
                        self.player_systems
                            .execute(&mut self.ecs, &mut self.resources);
                    }
                    TurnState::MonsterTurn => {
                        self.monster_systems
                            .execute(&mut self.ecs, &mut self.resources);
                    }
                    TurnState::GameOver => {
                        self.game_over(ctx);
                    }
                }
            }
        }
        render_draw_buffer(ctx).expect("Render error");
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Duck Dungeon")
        .with_fps_cap(30.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            "dungeonfont.png",
        )
        .with_simple_console_no_bg(
            SCREEN_WIDTH * 2,
            SCREEN_HEIGHT * 2,
            "terminal8x8.png",
        )
        .build()?;

    main_loop(context, State::new())
}
