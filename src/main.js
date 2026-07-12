const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const PHASE_LABELS = {
  active: "Aktiv — nächste Pause in",
  idlepaused: "Pausiert (inaktiv)",
  break: "Augenpause! Noch",
};

function formatTime(totalSecs) {
  const m = Math.floor(totalSecs / 60);
  const s = totalSecs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function renderTimerStatus(status) {
  document.getElementById("timer-phase").textContent =
    PHASE_LABELS[status.phase] ?? status.phase;
  document.getElementById("timer-remaining").textContent = formatTime(
    status.remainingSecs
  );
}

listen("timer-status", (event) => renderTimerStatus(event.payload));
invoke("get_timer_status").then(renderTimerStatus);

const el = {
  form: document.getElementById("settings-form"),
  interval: document.getElementById("interval"),
  break: document.getElementById("break"),
  idle: document.getElementById("idle"),
  theme: document.getElementById("theme"),
  autostart: document.getElementById("autostart"),
  status: document.getElementById("save-status"),
};

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
}

function fillForm(settings) {
  el.interval.value = settings.intervalMinutes;
  el.break.value = settings.breakMinutes;
  el.idle.value = settings.idleBufferMinutes;
  el.theme.value = settings.theme;
  el.autostart.checked = settings.autostart;
  const radio = document.querySelector(
    `input[name="difficulty"][value="${settings.difficulty}"]`
  );
  if (radio) radio.checked = true;
  applyTheme(settings.theme);
}

function readForm() {
  return {
    intervalMinutes: Number(el.interval.value),
    breakMinutes: Number(el.break.value),
    idleBufferMinutes: Number(el.idle.value),
    difficulty: document.querySelector('input[name="difficulty"]:checked')
      .value,
    theme: el.theme.value,
    autostart: el.autostart.checked,
  };
}

function showStatus(message, kind) {
  el.status.textContent = message;
  el.status.className = kind;
  setTimeout(() => {
    el.status.textContent = "";
    el.status.className = "";
  }, 3000);
}

el.theme.addEventListener("change", () => applyTheme(el.theme.value));

el.form.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await invoke("set_settings", { settings: readForm() });
    showStatus("Gespeichert ✓", "ok");
  } catch (error) {
    showStatus(`Fehler: ${error}`, "error");
  }
});

invoke("get_settings").then(fillForm);
