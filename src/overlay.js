const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const card = document.getElementById("overlay-card");
const titleEl = document.getElementById("overlay-title");
const countdownEl = document.getElementById("overlay-countdown");
const hintEl = document.getElementById("overlay-hint");
const snoozeBtn = document.getElementById("btn-snooze");
const breakNowBtn = document.getElementById("btn-break-now");
const skipBtn = document.getElementById("btn-skip");

let blinkedOnceAt5 = false;

function formatTime(totalSecs) {
  const m = Math.floor(totalSecs / 60);
  const s = totalSecs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function render(status) {
  countdownEl.textContent = formatTime(status.remainingSecs);

  if (status.phase === "break") {
    card.classList.add("break");
    card.classList.remove("blink-once", "blink-loop");
    titleEl.textContent = "Augenpause!";
    hintEl.textContent = "Schau aus dem Fenster oder in die Ferne — nicht auf den Bildschirm.";
    snoozeBtn.classList.add("hidden");
    breakNowBtn.classList.add("hidden");
    blinkedOnceAt5 = false;

    // Überspringen: je nach Schwierigkeitsgrad sofort, nach Wartezeit oder nie
    if (status.skipAllowedInSecs === null) {
      skipBtn.classList.add("hidden");
    } else {
      skipBtn.classList.remove("hidden");
      if (status.skipAllowedInSecs > 0) {
        skipBtn.disabled = true;
        skipBtn.textContent = `Überspringen (in ${status.skipAllowedInSecs}s)`;
      } else {
        skipBtn.disabled = false;
        skipBtn.textContent = "Überspringen";
      }
    }
  } else if (status.phase === "active") {
    card.classList.remove("break");
    titleEl.textContent =
      status.context === "game" ? "Gleich Augenpause 🎮" : "Gleich Augenpause";
    hintEl.textContent =
      status.context === "game"
        ? "Spiel erkannt — bring die Runde zu Ende, dann Augen entspannen."
        : "Schau gleich 5 Minuten aus dem Fenster oder in die Ferne.";
    skipBtn.classList.add("hidden");
    breakNowBtn.classList.remove("hidden");
    breakNowBtn.disabled = false;

    // Verlängern nur zeigen, wenn noch Snoozes übrig sind (null = unbegrenzt)
    if (status.snoozesLeft !== null && status.snoozesLeft === 0) {
      snoozeBtn.classList.add("hidden");
    } else {
      snoozeBtn.classList.remove("hidden");
      snoozeBtn.disabled = false;
      snoozeBtn.textContent =
        status.snoozesLeft === null
          ? "+5 Min. verlängern"
          : `+5 Min. verlängern (${status.snoozesLeft}x übrig)`;
    }

    const remaining = status.remainingSecs;
    if (remaining <= 90) {
      card.classList.add("blink-loop");
      card.classList.remove("blink-once");
    } else if (remaining <= 300 && !blinkedOnceAt5) {
      blinkedOnceAt5 = true;
      card.classList.add("blink-once");
      card.addEventListener(
        "animationend",
        () => card.classList.remove("blink-once"),
        { once: true }
      );
    } else if (remaining > 300) {
      blinkedOnceAt5 = false;
      card.classList.remove("blink-loop", "blink-once");
    }
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

listen("timer-status", (event) => render(event.payload));

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

invoke("get_timer_status").then(render);
