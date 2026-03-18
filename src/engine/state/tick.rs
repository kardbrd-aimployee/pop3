use std::time::Instant;

use super::constants::*;
use super::flags::GameFlags;
use super::rng::GameRng;
use super::state_machine::GameState;
use super::traits::*;
use super::tribe::TribeArray;
use super::victory;

/// Time source abstraction.
/// Original uses GetTickCount() (milliseconds since boot).
pub trait TimeSource {
    /// Returns current time in milliseconds.
    fn now_ms(&self) -> u64;
}

/// std::time::Instant-based time source for native builds.
pub struct StdTimeSource {
    epoch: Instant,
}

impl StdTimeSource {
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl TimeSource for StdTimeSource {
    fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }
}

/// Central game world state. Owns all simulation data.
///
/// This struct is the single owner of the game's simulation state.
/// It drives the tick loop that updates all subsystems.
pub struct GameWorld {
    /// Current top-level game state.
    pub state: GameState,

    /// Game flags bitfield.
    /// Original: g_GameFlags at 0x00884bf9
    pub flags: GameFlags,

    /// Random number generator (deterministic for lockstep).
    /// Original: g_RandomSeed at 0x00885710
    pub rng: GameRng,

    /// Per-tribe data (4 tribes).
    /// Original: g_TribeArray at 0x00885760
    pub tribes: TribeArray,

    /// Ticks per second.
    /// Original: g_GameSpeed at 0x008856f9
    pub game_speed: u32,

    /// Current game tick counter.
    /// Original: g_GameTick at 0x0088571c
    pub game_tick: u32,

    /// Total ticks this session (used for victory check timing).
    /// Original: g_TickCounter at 0x00885720
    pub tick_counter: u32,

    /// AI update loop multiplier. Inner loop runs (ai_update_mult + 1) times.
    /// Original: g_AIUpdateMult (signed byte) at 0x0087e344
    pub ai_update_mult: i8,

    /// Local player tribe index (0-3).
    /// Original: g_PlayerTribe at 0x00884c88
    pub player_tribe: u8,

    /// Tutorial mode flag.
    /// Original: at 0x00884119
    /// Values 2 or 3 trigger tutorial tick instead of single-player tick.
    pub tutorial_mode: u8,

    /// Milliseconds per tick = 1000 / game_speed.
    /// Original: g_TickIntervalMs at 0x0059ac70
    tick_interval_ms: u64,

    /// Last tick timestamp (ms).
    /// Original: g_LastTickTime at 0x0059ac6c
    last_tick_time: u64,
}

impl GameWorld {
    pub fn new(game_speed: u32) -> Self {
        let speed = game_speed.max(1);
        Self {
            state: GameState::Frontend,
            flags: GameFlags::new(),
            rng: GameRng::new(0),
            tribes: TribeArray::new(),
            game_speed: speed,
            game_tick: 0,
            tick_counter: 0,
            ai_update_mult: 0,
            player_tribe: 0,
            tutorial_mode: 0,
            tick_interval_ms: (TICK_BASE_MS as u64) / (speed as u64),
            last_tick_time: 0,
        }
    }

    /// Set game speed and recompute tick interval.
    /// Original: g_TickIntervalMs = 1000 / g_GameSpeed (0x004bb5a0 preamble)
    pub fn set_game_speed(&mut self, speed: u32) {
        self.game_speed = speed.max(1);
        self.tick_interval_ms = (TICK_BASE_MS as u64) / (self.game_speed as u64);
    }

    /// Drive the simulation tick loop. Called once per frame.
    /// Original: Game_SimulationTick at 0x004bb5a0
    ///
    /// Computes how many ticks to run based on elapsed time since the last
    /// tick, capped at MAX_CATCHUP_TICKS to prevent spiral of death.
    ///
    /// Returns the number of ticks actually executed this frame.
    pub fn simulation_tick(
        &mut self,
        time: &dyn TimeSource,
        subs: &mut TickSubsystems,
    ) -> u32 {
        // Only tick during active gameplay
        if self.state != GameState::InGame {
            return 0;
        }

        // Recompute tick interval each frame (matches original preamble)
        // Original: EAX = 0x3E8; CDQ; IDIV ECX (game_speed)
        self.tick_interval_ms = (TICK_BASE_MS as u64) / (self.game_speed.max(1) as u64);

        let now = time.now_ms();

        // First tick initialization
        // Original: if g_GameTick == 0: call ESI (GetTickCount); g_LastTickTime = EAX - 1
        if self.game_tick == 0 {
            self.last_tick_time = now.saturating_sub(1);
        }

        // Don't tick if paused
        if self.flags.is_paused() {
            return 0;
        }

        // Time hasn't advanced enough for a tick yet
        if now <= self.last_tick_time {
            return 0;
        }

        // Single-player tick loop
        // Original: 0x004bb941 branch (when not multiplayer)
        let mut ticks_run: u32 = 0;

        while now > self.last_tick_time && ticks_run < MAX_CATCHUP_TICKS as u32 {
            self.run_one_tick(subs);
            self.last_tick_time += self.tick_interval_ms;
            ticks_run += 1;
        }

        ticks_run
    }

    /// Execute a single simulation tick.
    /// Faithfully reproduces the call order from Game_SimulationTick (0x004bb5a0).
    ///
    /// Original order (from disassembly at 0x004bb956-0x004bb9c9):
    /// 1. Tick_ProcessNetworkMessages (0x004a76b0)
    /// 2. Tick_ProcessPendingActions (0x004a6f60) — conditional on flags
    /// 3. Tick_UpdateGameTime (0x004a7ac0)
    /// 4. Tick_UpdateTerrain (0x0048bda0)
    /// 5. Tick_UpdateObjects (0x004a7550)
    /// 6. Tick_UpdateWater (0x0048bf10)
    /// 7. Inner loop (ai_update_mult + 1 times):
    ///    a. Tick_UpdateSinglePlayer/Tutorial
    ///    b. AI_UpdateAllTribes (skip if FLAG_VICTORY_DEFEAT)
    ///    c. Tick_UpdatePopulation
    ///    d. Tick_UpdateMana
    fn run_one_tick(&mut self, subs: &mut TickSubsystems) {
        // 1. Tick_ProcessNetworkMessages (0x004a76b0)
        let proceed = subs.network.tick_process_network();
        if !proceed {
            return;
        }

        // 2. Tick_ProcessPendingActions (0x004a6f60)
        subs.actions.tick_process_actions();

        // 3. Tick_UpdateGameTime (0x004a7ac0)
        subs.game_time.tick_update_game_time();

        // 4. Tick_UpdateTerrain (0x0048bda0)
        subs.terrain.tick_update_terrain();

        // 5. Tick_UpdateObjects (0x004a7550) -- persons, buildings, projectiles via UnitCoordinator
        subs.objects.tick_update_objects();

        // 6. Tick_UpdateWater (0x0048bf10)
        subs.water.tick_update_water();

        // 7. Inner loop: (g_AIUpdateMult + 1) iterations
        // Original: MOVSX ESI, byte ptr [0x0087e344]; INC ESI; JZ skip
        let iterations = (self.ai_update_mult as i32 + 1).max(0);

        for _ in 0..iterations {
            // 7a. Single-player or tutorial tick
            // Original: CMP byte ptr [0x00884119], 0x02; JZ tutorial
            if self.tutorial_mode == TUTORIAL_MODE_2
                || self.tutorial_mode == TUTORIAL_MODE_3
            {
                // Tick_UpdateTutorial (0x00469320)
                subs.tutorial.tick_update_tutorial();
            } else {
                // Tick_UpdateSinglePlayer (0x00456500)
                subs.single_player.tick_update_single_player();

                // 7b. AI_UpdateAllTribes (0x0041a7d0) — skip if victory/defeat
                // Original: TEST dword ptr [0x00884bf9], 0x800000; JNZ skip
                if !self.flags.is_victory_defeat() {
                    subs.ai.tick_update_ai();
                }
            }

            // 7c. Tick_UpdatePopulation (0x004198f0) -- building spawn processing, housing capacity
            subs.population.tick_update_population();

            // 7d. Tick_UpdateMana (0x004aeac0) -- mana generation from persons + housing
            //     via mana_rate_for_person/add_mana per person per tick
            subs.mana.tick_update_mana();
        }

        // Increment tick counters (done inside Tick_UpdateMana in the original,
        // but logically belongs here for clarity)
        self.game_tick = self.game_tick.wrapping_add(1);
        self.tick_counter = self.tick_counter.wrapping_add(1);

        // Victory/defeat check
        victory::check_victory_conditions(
            self.tick_counter,
            &mut self.flags,
            &mut self.tribes,
            self.player_tribe,
        );
    }
}

/// Bundle of subsystem trait objects passed to the tick loop.
/// This avoids passing 11 separate `&mut dyn Trait` parameters.
pub struct TickSubsystems<'a> {
    pub terrain: &'a mut dyn TerrainTick,
    pub objects: &'a mut dyn ObjectTick,
    pub water: &'a mut dyn WaterTick,
    pub network: &'a mut dyn NetworkTick,
    pub actions: &'a mut dyn ActionTick,
    pub game_time: &'a mut dyn GameTimeTick,
    pub single_player: &'a mut dyn SinglePlayerTick,
    pub tutorial: &'a mut dyn TutorialTick,
    pub ai: &'a mut dyn AiTick,
    pub population: &'a mut dyn PopulationTick,
    pub mana: &'a mut dyn ManaTick,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Mock time source that returns a fixed value.
    struct MockTime {
        ms: u64,
    }
    impl TimeSource for MockTime {
        fn now_ms(&self) -> u64 {
            self.ms
        }
    }

    /// Shared call log for recording subsystem call order.
    type CallLog = Rc<RefCell<Vec<&'static str>>>;

    /// Create a NoOp subsystems bundle for tests that don't care about call order.
    fn noop_subs() -> (NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp) {
        (NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp)
    }

    macro_rules! make_subs {
        ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr, $j:expr, $k:expr) => {
            TickSubsystems {
                terrain: $a, objects: $b, water: $c,
                network: $d, actions: $e, game_time: $f,
                single_player: $g, tutorial: $h, ai: $i,
                population: $j, mana: $k,
            }
        };
    }

    // Individual recorder structs that share a call log.
    struct RecTerrain(CallLog);
    struct RecObjects(CallLog);
    struct RecWater(CallLog);
    struct RecNetwork(CallLog);
    struct RecActions(CallLog);
    struct RecGameTime(CallLog);
    struct RecSinglePlayer(CallLog);
    struct RecTutorial(CallLog);
    struct RecAi(CallLog);
    struct RecPopulation(CallLog);
    struct RecMana(CallLog);

    impl TerrainTick for RecTerrain { fn tick_update_terrain(&mut self) { self.0.borrow_mut().push("terrain"); } }
    impl ObjectTick for RecObjects { fn tick_update_objects(&mut self) { self.0.borrow_mut().push("objects"); } }
    impl WaterTick for RecWater { fn tick_update_water(&mut self) { self.0.borrow_mut().push("water"); } }
    impl NetworkTick for RecNetwork { fn tick_process_network(&mut self) -> bool { self.0.borrow_mut().push("network"); true } }
    impl ActionTick for RecActions { fn tick_process_actions(&mut self) { self.0.borrow_mut().push("actions"); } }
    impl GameTimeTick for RecGameTime { fn tick_update_game_time(&mut self) { self.0.borrow_mut().push("game_time"); } }
    impl SinglePlayerTick for RecSinglePlayer { fn tick_update_single_player(&mut self) { self.0.borrow_mut().push("single_player"); } }
    impl TutorialTick for RecTutorial { fn tick_update_tutorial(&mut self) { self.0.borrow_mut().push("tutorial"); } }
    impl AiTick for RecAi { fn tick_update_ai(&mut self) { self.0.borrow_mut().push("ai"); } }
    impl PopulationTick for RecPopulation { fn tick_update_population(&mut self) { self.0.borrow_mut().push("population"); } }
    impl ManaTick for RecMana { fn tick_update_mana(&mut self) { self.0.borrow_mut().push("mana"); } }

    #[test]
    fn test_no_tick_when_not_in_game() {
        let mut world = GameWorld::new(10);
        world.state = GameState::Frontend;
        let time = MockTime { ms: 1000 };
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h, mut i, mut j, mut k) = noop_subs();
        let mut subs = make_subs!(&mut a, &mut b, &mut c, &mut d, &mut e, &mut f, &mut g, &mut h, &mut i, &mut j, &mut k);
        assert_eq!(world.simulation_tick(&time, &mut subs), 0);
    }

    #[test]
    fn test_tick_order() {
        let mut world = GameWorld::new(10); // 10 ticks/sec = 100ms interval
        world.state = GameState::InGame;
        world.ai_update_mult = 0; // 1 inner iteration

        let log: CallLog = Rc::new(RefCell::new(Vec::new()));
        let mut r_terrain = RecTerrain(log.clone());
        let mut r_objects = RecObjects(log.clone());
        let mut r_water = RecWater(log.clone());
        let mut r_network = RecNetwork(log.clone());
        let mut r_actions = RecActions(log.clone());
        let mut r_game_time = RecGameTime(log.clone());
        let mut r_single_player = RecSinglePlayer(log.clone());
        let mut r_tutorial = RecTutorial(log.clone());
        let mut r_ai = RecAi(log.clone());
        let mut r_population = RecPopulation(log.clone());
        let mut r_mana = RecMana(log.clone());

        // First call initializes last_tick_time
        let time = MockTime { ms: 0 };
        let mut subs = make_subs!(
            &mut r_terrain, &mut r_objects, &mut r_water,
            &mut r_network, &mut r_actions, &mut r_game_time,
            &mut r_single_player, &mut r_tutorial, &mut r_ai,
            &mut r_population, &mut r_mana
        );
        world.simulation_tick(&time, &mut subs);
        log.borrow_mut().clear();

        // Advance time past one tick interval (100ms)
        let time = MockTime { ms: 150 };
        world.simulation_tick(&time, &mut subs);

        // Verify call order matches the original
        assert_eq!(
            *log.borrow(),
            vec![
                "network", "actions", "game_time", "terrain", "objects", "water",
                "single_player", "ai", "population", "mana",
            ]
        );
    }

    #[test]
    fn test_catchup_capped() {
        let mut world = GameWorld::new(10); // 100ms per tick
        world.state = GameState::InGame;

        // Initialize (ms=1 so first tick actually runs; ms=0 would saturate to 0)
        let time = MockTime { ms: 1 };
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h, mut i, mut j, mut k) = noop_subs();
        let mut subs = make_subs!(&mut a, &mut b, &mut c, &mut d, &mut e, &mut f, &mut g, &mut h, &mut i, &mut j, &mut k);
        world.simulation_tick(&time, &mut subs);

        // Jump far ahead (should be capped at MAX_CATCHUP_TICKS)
        let time = MockTime { ms: 10_000 };
        let ticks = world.simulation_tick(&time, &mut subs);
        assert_eq!(ticks, MAX_CATCHUP_TICKS as u32);
    }

    #[test]
    fn test_tutorial_mode_uses_tutorial_tick() {
        let mut world = GameWorld::new(10);
        world.state = GameState::InGame;
        world.tutorial_mode = TUTORIAL_MODE_2;
        world.ai_update_mult = 0;

        let log: CallLog = Rc::new(RefCell::new(Vec::new()));
        let mut r_terrain = RecTerrain(log.clone());
        let mut r_objects = RecObjects(log.clone());
        let mut r_water = RecWater(log.clone());
        let mut r_network = RecNetwork(log.clone());
        let mut r_actions = RecActions(log.clone());
        let mut r_game_time = RecGameTime(log.clone());
        let mut r_single_player = RecSinglePlayer(log.clone());
        let mut r_tutorial = RecTutorial(log.clone());
        let mut r_ai = RecAi(log.clone());
        let mut r_population = RecPopulation(log.clone());
        let mut r_mana = RecMana(log.clone());

        let time = MockTime { ms: 0 };
        let mut subs = make_subs!(
            &mut r_terrain, &mut r_objects, &mut r_water,
            &mut r_network, &mut r_actions, &mut r_game_time,
            &mut r_single_player, &mut r_tutorial, &mut r_ai,
            &mut r_population, &mut r_mana
        );
        world.simulation_tick(&time, &mut subs);
        log.borrow_mut().clear();

        let time = MockTime { ms: 150 };
        world.simulation_tick(&time, &mut subs);

        let calls = log.borrow();
        // Should use "tutorial" instead of "single_player" + "ai"
        assert!(calls.contains(&"tutorial"));
        assert!(!calls.contains(&"single_player"));
        assert!(!calls.contains(&"ai"));
    }
}
