import { setLanguage, t } from "./i18n.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const card = document.getElementById("overlay-card");
const titleEl = document.getElementById("overlay-title");
const countdownEl = document.getElementById("overlay-countdown");
const hintEl = document.getElementById("overlay-hint");
const snoozeBtn = document.getElementById("btn-snooze");
const breakNowBtn = document.getElementById("btn-break-now");
const skipBtn = document.getElementById("btn-skip");

let lastStatus = null;

function formatTime(totalSecs) {
  const m = Math.floor(totalSecs / 60);
  const s = totalSecs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function render(status) {
  lastStatus = status;
  countdownEl.textContent = formatTime(status.remainingSecs);

  if (status.phase === "break") {
    card.classList.add("break");
    card.classList.remove("warn-soft", "warn-hard");
    titleEl.textContent = t("overlayBreak");
    hintEl.textContent = t("overlayHintBreak");
    snoozeBtn.classList.add("hidden");
    breakNowBtn.classList.add("hidden");

    // Überspringen: je nach Schwierigkeitsgrad sofort, nach Wartezeit oder nie
    if (status.skipAllowedInSecs === null) {
      skipBtn.classList.add("hidden");
    } else {
      skipBtn.classList.remove("hidden");
      if (status.skipAllowedInSecs > 0) {
        skipBtn.disabled = true;
        skipBtn.textContent = t("skipIn", { n: status.skipAllowedInSecs });
      } else {
        skipBtn.disabled = false;
        skipBtn.textContent = t("skip");
      }
    }
  } else if (status.phase === "active") {
    card.classList.remove("break");
    titleEl.textContent =
      status.context === "game" ? t("overlaySoonGame") : t("overlaySoon");
    hintEl.textContent =
      status.context === "game" ? t("overlayHintGame") : t("overlayHintSoon");
    skipBtn.classList.add("hidden");
    breakNowBtn.classList.remove("hidden");
    breakNowBtn.disabled = false;
    breakNowBtn.textContent = t("breakNow");

    // Verlängern nur zeigen, wenn noch Snoozes übrig sind (null = unbegrenzt)
    if (status.snoozesLeft !== null && status.snoozesLeft === 0) {
      snoozeBtn.classList.add("hidden");
    } else {
      snoozeBtn.classList.remove("hidden");
      snoozeBtn.disabled = false;
      snoozeBtn.textContent =
        status.snoozesLeft === null
          ? t("snooze")
          : t("snoozeLeft", { n: status.snoozesLeft });
    }

    // Warnstufen: sanftes Pulsieren ab 5 Min., schnelles ab 1:30
    const remaining = status.remainingSecs;
    card.classList.toggle("warn-hard", remaining <= 90);
    card.classList.toggle("warn-soft", remaining > 90 && remaining <= 300);
  }

  if (card.classList.contains("hidden")) {
    card.classList.remove("hidden");
    card.classList.add("appear");
    card.addEventListener(
      "animationend",
      () => card.classList.remove("appear"),
      { once: true }
    );
  }
}

function applySettings(settings) {
  setLanguage(settings.language);
  if (lastStatus) render(lastStatus);
}

listen("timer-status", (event) => render(event.payload));

listen("settings-changed", (event) => applySettings(event.payload));

listen("overlay-interactive", (event) => {
  document.body.classList.toggle("interactive", event.payload);
});

snoozeBtn.addEventListener("click", async () => {
  try {
    await invoke("snooze");
  } catch (error) {
    hintEl.textContent = String(error);
  }
});

breakNowBtn.addEventListener("click", () => invoke("start_break_now"));

skipBtn.addEventListener("click", async () => {
  try {
    await invoke("skip_break");
  } catch (error) {
    hintEl.textContent = String(error);
  }
});

invoke("get_settings").then((settings) => {
  applySettings(settings);
  invoke("get_timer_status").then(render);
});
