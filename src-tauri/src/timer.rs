use crate::config::{ConfigState, Difficulty};
use crate::context::{self, UsageContext};
use crate::flow;
use crate::idle;
use crate::lock;
use crate::overlay;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// Aktive Nutzung, Intervall zählt runter
    Active,
    /// Nutzer ist länger als der Idle-Puffer inaktiv — Timer eingefroren
    IdlePaused,
    /// Augenpause läuft
    Break,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerStatus {
    pub phase: Phase,
    pub remaining_secs: u64,
    pub snoozes_used: u32,
    /// Wie viele Snoozes noch erlaubt sind; None = unbegrenzt
    pub snoozes_left: Option<u32>,
    /// In wie vielen Sekunden die Pause übersprungen werden darf; None = nie
    pub skip_allowed_in_secs: Option<u64>,
    /// Erkannter Nutzungskontext (Spiel oder Arbeit)
    pub context: UsageContext,
}

/// Maximale Snooze-Anzahl je Schwierigkeitsgrad; None = unbegrenzt.
/// Im Spiel-Kontext gibt es einen Snooze extra — die laufende Runde darf zu Ende
/// gespielt werden (dadurch hat auch "Streng" im Spiel genau einen Aufschub).
fn snooze_limit(difficulty: Difficulty, context: UsageContext) -> Option<u32> {
    let base = match difficulty {
        Difficulty::Locker => return None,
        Difficulty::Normal => 2,
        Difficulty::Streng => 0,
    };
    Some(base + u32::from(context == UsageContext::Game))
}

/// Wartezeit in der Pause, bevor "Überspringen" erlaubt ist; None = nie erlaubt
fn skip_wait_secs(difficulty: Difficulty) -> Option<u64> {
    match difficulty {
        Difficulty::Locker => Some(0),
        Difficulty::Normal => Some(30),
        Difficulty::Streng => None,
    }
}

pub struct TimerCore {
    pub phase: Phase,
    pub remaining_secs: u64,
    pub snoozes_used: u32,
    /// Höchster beobachteter Idle-Wert während IdlePaused (für die Reset-Entscheidung)
    max_idle_secs: u64,
    /// Wie lange die laufende Pause schon dauert (für die Skip-Wartezeit)
    break_elapsed_secs: u64,
    /// Wie viel Flow-Aufschub in diesem Intervall schon verbraucht wurde
    flow_extended_secs: u64,
    /// Zuletzt erkannter Nutzungskontext (im Tick aktualisiert)
    context: UsageContext,
}

pub struct TimerState(pub Mutex<TimerCore>);

impl TimerCore {
    fn new(interval_minutes: u32) -> Self {
        Self {
            phase: Phase::Active,
            remaining_secs: u64::from(interval_minutes) * 60,
            snoozes_used: 0,
            max_idle_secs: 0,
            break_elapsed_secs: 0,
            flow_extended_secs: 0,
            context: UsageContext::Work,
        }
    }

    fn status(&self, difficulty: Difficulty) -> TimerStatus {
        let snoozes_left = snooze_limit(difficulty, self.context)
            .map(|limit| limit.saturating_sub(self.snoozes_used));
        let skip_allowed_in_secs = if self.phase == Phase::Break {
            skip_wait_secs(difficulty)
                .map(|wait| wait.saturating_sub(self.break_elapsed_secs))
        } else {
            None
        };
        TimerStatus {
            phase: self.phase,
            remaining_secs: self.remaining_secs,
            snoozes_used: self.snoozes_used,
            snoozes_left,
            skip_allowed_in_secs,
            context: self.context,
        }
    }
}

/// Setzt den Timer auf ein volles Intervall zurück (z.B. nach Settings-Änderung).
pub fn reset(app: &AppHandle) {
    let interval = app
        .state::<ConfigState>()
        .0
        .lock()
        .unwrap()
        .interval_minutes;
    let state = app.state::<TimerState>();
    let mut core = state.0.lock().unwrap();
    *core = TimerCore::new(interval);
}

pub fn spawn(app: AppHandle) {
    let interval = app
        .state::<ConfigState>()
        .0
        .lock()
        .unwrap()
        .interval_minutes;
    app.manage(TimerState(Mutex::new(TimerCore::new(interval))));

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        tick(&app);
    });
}

fn tick(app: &AppHandle) {
    let settings = app.state::<ConfigState>().0.lock().unwrap().clone();
    let timer_state = app.state::<TimerState>();
    let mut core = timer_state.0.lock().unwrap();
    let idle_secs = idle::idle_seconds();
    core.context = context::current_context();

    match core.phase {
        Phase::Active => {
            if idle_secs >= u64::from(settings.idle_buffer_minutes) * 60 {
                core.phase = Phase::IdlePaused;
                core.max_idle_secs = idle_secs;
            } else {
                core.remaining_secs = core.remaining_secs.saturating_sub(1);
                if core.remaining_secs == 0 {
                    // Wer gerade im Schreibfluss ist, bekommt die Pause minutenweise
                    // aufgeschoben — bis zum harten Cap
                    if flow::is_in_flow() && core.flow_extended_secs < flow::FLOW_CAP_SECS {
                        core.remaining_secs = 60;
                        core.flow_extended_secs += 60;
                    } else {
                        start_break(&mut core, app, &settings);
                    }
                }
            }
        }
        Phase::IdlePaused => {
            if idle_secs < 2 {
                // Nutzer ist zurück. War die Abwesenheit mindestens so lang wie eine
                // Pause, zählt sie als Pause → volles Intervall beginnt neu.
                if core.max_idle_secs >= u64::from(settings.break_minutes) * 60 {
                    core.remaining_secs = u64::from(settings.interval_minutes) * 60;
                    core.snoozes_used = 0;
                }
                core.phase = Phase::Active;
                core.max_idle_secs = 0;
            } else {
                core.max_idle_secs = core.max_idle_secs.max(idle_secs);
            }
        }
        Phase::Break => {
            core.remaining_secs = core.remaining_secs.saturating_sub(1);
            core.break_elapsed_secs += 1;
            if core.remaining_secs == 0 {
                end_break(&mut core, &settings);
                let _ = app.emit("break-over", core.status(settings.difficulty));
                overlay::set_blocking(app, false);
            }
        }
    }

    let _ = app.emit("timer-status", core.status(settings.difficulty));

    // Overlay in den letzten 10 Minuten des Intervalls und während der Pause zeigen
    let show_overlay = match core.phase {
        Phase::Active => core.remaining_secs <= OVERLAY_LEAD_SECS,
        Phase::Break => true,
        Phase::IdlePaused => false,
    };
    drop(core);
    overlay::set_visible(app, show_overlay);
}

/// Übergang in die Pause — bei "Streng" wird zusätzlich blockiert und gesperrt.
fn start_break(core: &mut TimerCore, app: &AppHandle, settings: &crate::config::Settings) {
    core.phase = Phase::Break;
    core.remaining_secs = u64::from(settings.break_minutes) * 60;
    core.break_elapsed_secs = 0;
    let _ = app.emit("break-due", core.status(settings.difficulty));
    if settings.difficulty == Difficulty::Streng {
        overlay::set_blocking(app, true);
        lock::lock_session();
    }
}

fn end_break(core: &mut TimerCore, settings: &crate::config::Settings) {
    core.phase = Phase::Active;
    core.remaining_secs = u64::from(settings.interval_minutes) * 60;
    core.snoozes_used = 0;
    core.break_elapsed_secs = 0;
    core.flow_extended_secs = 0;
}

/// Ab so vielen Restsekunden wird das Overlay eingeblendet (10 Minuten)
pub const OVERLAY_LEAD_SECS: u64 = 600;
/// Um so viele Minuten verschiebt ein Snooze die Pause
const SNOOZE_MINUTES: u64 = 5;

#[tauri::command]
pub fn snooze(app: AppHandle, state: State<TimerState>) -> Result<TimerStatus, String> {
    let settings = app.state::<ConfigState>().0.lock().unwrap().clone();
    let mut core = state.0.lock().unwrap();
    if core.phase != Phase::Active {
        return Err("Verlängern ist nur vor der Pause möglich".into());
    }
    if let Some(limit) = snooze_limit(settings.difficulty, core.context) {
        if core.snoozes_used >= limit {
            return Err("Kein Verlängern mehr möglich".into());
        }
    }
    core.snoozes_used += 1;
    core.remaining_secs += SNOOZE_MINUTES * 60;
    let status = core.status(settings.difficulty);
    drop(core);
    let _ = app.emit("timer-status", status.clone());
    Ok(status)
}

#[tauri::command]
pub fn start_break_now(app: AppHandle, state: State<TimerState>) -> TimerStatus {
    let settings = app.state::<ConfigState>().0.lock().unwrap().clone();
    let mut core = state.0.lock().unwrap();
    if core.phase == Phase::Active {
        start_break(&mut core, &app, &settings);
    }
    let status = core.status(settings.difficulty);
    drop(core);
    let _ = app.emit("timer-status", status.clone());
    status
}

/// Pause überspringen — je nach Schwierigkeitsgrad sofort, verzögert oder nie erlaubt.
#[tauri::command]
pub fn skip_break(app: AppHandle, state: State<TimerState>) -> Result<TimerStatus, String> {
    let settings = app.state::<ConfigState>().0.lock().unwrap().clone();
    let mut core = state.0.lock().unwrap();
    if core.phase != Phase::Break {
        return Err("Es läuft gerade keine Pause".into());
    }
    match skip_wait_secs(settings.difficulty) {
        None => return Err("Überspringen ist bei diesem Schwierigkeitsgrad nicht möglich".into()),
        Some(wait) if core.break_elapsed_secs < wait => {
            return Err("Überspringen ist noch nicht freigeschaltet".into())
        }
        Some(_) => {}
    }
    end_break(&mut core, &settings);
    let status = core.status(settings.difficulty);
    drop(core);
    overlay::set_blocking(&app, false);
    let _ = app.emit("timer-status", status.clone());
    Ok(status)
}

#[tauri::command]
pub fn get_timer_status(app: AppHandle, state: State<TimerState>) -> TimerStatus {
    let difficulty = app.state::<ConfigState>().0.lock().unwrap().difficulty;
    state.0.lock().unwrap().status(difficulty)
}
