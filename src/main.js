import { setLanguage, t, applyTranslations } from "./i18n.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const el = {
  form: document.getElementById("settings-form"),
  interval: document.getElementById("interval"),
  break: document.getElementById("break"),
  idle: document.getElementById("idle"),
  theme: document.getElementById("theme"),
  language: document.getElementById("language"),
  autostart: document.getElementById("autostart"),
  overlayAlways: document.getElementById("overlay-always"),
  status: document.getElementById("save-status"),
};

let lastTimerStatus = null;

function formatTime(totalSecs) {
  const m = Math.floor(totalSecs / 60);
  const s = totalSecs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function renderTimerStatus(status) {
  lastTimerStatus = status;
  const phaseKeys = {
    active: "phaseActive",
    idlepaused: "phaseIdle",
    break: "phaseBreak",
  };
  document.getElementById("timer-phase").textContent = t(
    phaseKeys[status.phase] ?? status.phase
  );
  document.getElementById("timer-remaining").textContent = formatTime(
    status.remainingSecs
  );
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
}

function applyLanguage(lang) {
  setLanguage(lang);
  applyTranslations();
  if (lastTimerStatus) renderTimerStatus(lastTimerStatus);
}

function fillForm(settings) {
  el.interval.value = settings.intervalMinutes;
  el.break.value = settings.breakMinutes;
  el.idle.value = settings.idleBufferMinutes;
  el.theme.value = settings.theme;
  el.language.value = settings.language;
  el.autostart.checked = settings.autostart;
  el.overlayAlways.checked = settings.overlayAlwaysVisible;
  const radio = document.querySelector(
    `input[name="difficulty"][value="${settings.difficulty}"]`
  );
  if (radio) radio.checked = true;
  applyTheme(settings.theme);
  applyLanguage(settings.language);
}

function readForm() {
  return {
    intervalMinutes: Number(el.interval.value),
    breakMinutes: Number(el.break.value),
    idleBufferMinutes: Number(el.idle.value),
    difficulty: document.querySelector('input[name="difficulty"]:checked')
      .value,
    theme: el.theme.value,
    language: el.language.value,
    autostart: el.autostart.checked,
    overlayAlwaysVisible: el.overlayAlways.checked,
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
el.language.addEventListener("change", () => applyLanguage(el.language.value));

el.form.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await invoke("set_settings", { settings: readForm() });
    showStatus(t("saved"), "ok");
  } catch (error) {
    showStatus(`${t("saveError")}: ${error}`, "error");
  }
});

listen("timer-status", (event) => renderTimerStatus(event.payload));

invoke("get_settings").then(fillForm);
invoke("get_timer_status").then(renderTimerStatus);
