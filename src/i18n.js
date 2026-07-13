// Alle UI-Texte in Deutsch und Englisch.
// Verwendung: t("key") — Sprache vorher mit setLanguage() setzen.

const STRINGS = {
  de: {
    // Settings-Fenster
    subtitle: "Stündliche Augenpausen, automatisch.",
    sectionTimer: "Timer",
    intervalLabel: "Aktives Intervall (Minuten)",
    intervalHint: "Nach dieser Zeit aktiver Nutzung wird eine Pause fällig.",
    breakLabel: "Pausenlänge (Minuten)",
    idleLabel: "Inaktivitäts-Puffer (Minuten)",
    idleHint: "Bist du länger als das inaktiv, pausiert der Timer automatisch.",
    sectionDifficulty: "Schwierigkeitsgrad",
    diffEasy: "Locker",
    diffEasyDesc: "Hinweis ist sofort wegklickbar, unbegrenzt verlängerbar.",
    diffNormal: "Normal",
    diffNormalDesc: "Überspringen erst nach 30 Sekunden, max. 2× verlängern.",
    diffStrict: "Streng",
    diffStrictDesc: "Kein Verlängern — nach Ablauf wird der PC gesperrt.",
    sectionGeneral: "Allgemein",
    themeLabel: "Design",
    themeSystem: "System",
    themeLight: "Hell",
    themeDark: "Dunkel",
    languageLabel: "Sprache",
    autostartLabel: "Mit dem System starten",
    overlayAlwaysLabel: "Countdown dauerhaft anzeigen",
    overlayAlwaysHint:
      "Sonst erscheint das Overlay erst in den letzten 10 Minuten.",
    save: "Speichern",
    saved: "Gespeichert ✓",
    saveError: "Fehler",
    phaseActive: "Aktiv — nächste Pause in",
    phaseIdle: "Pausiert (inaktiv)",
    phaseBreak: "Augenpause! Noch",

    // Overlay
    overlaySoon: "Gleich Augenpause",
    overlaySoonGame: "Gleich Augenpause 🎮",
    overlayBreak: "Augenpause!",
    overlayHintSoon: "Schau gleich 5 Minuten aus dem Fenster oder in die Ferne.",
    overlayHintGame: "Spiel erkannt — bring die Runde zu Ende, dann Augen entspannen.",
    overlayHintBreak: "Schau aus dem Fenster oder in die Ferne — nicht auf den Bildschirm.",
    snooze: "+5 Min. verlängern",
    snoozeLeft: "+5 Min. verlängern ({n}× übrig)",
    breakNow: "Pause jetzt starten",
    skip: "Überspringen",
    skipIn: "Überspringen (in {n}s)",
  },
  en: {
    subtitle: "Hourly eye breaks, automatically.",
    sectionTimer: "Timer",
    intervalLabel: "Active interval (minutes)",
    intervalHint: "After this much active use, a break is due.",
    breakLabel: "Break length (minutes)",
    idleLabel: "Idle buffer (minutes)",
    idleHint: "If you are inactive longer than this, the timer pauses automatically.",
    sectionDifficulty: "Difficulty",
    diffEasy: "Easy",
    diffEasyDesc: "Reminder can be dismissed instantly, unlimited postponing.",
    diffNormal: "Normal",
    diffNormalDesc: "Skipping unlocks after 30 seconds, postpone up to 2×.",
    diffStrict: "Strict",
    diffStrictDesc: "No postponing — when time is up, the PC gets locked.",
    sectionGeneral: "General",
    themeLabel: "Theme",
    themeSystem: "System",
    themeLight: "Light",
    themeDark: "Dark",
    languageLabel: "Language",
    autostartLabel: "Start with the system",
    overlayAlwaysLabel: "Always show countdown",
    overlayAlwaysHint: "Otherwise the overlay appears only in the last 10 minutes.",
    save: "Save",
    saved: "Saved ✓",
    saveError: "Error",
    phaseActive: "Active — next break in",
    phaseIdle: "Paused (idle)",
    phaseBreak: "Eye break! Remaining",

    overlaySoon: "Eye break soon",
    overlaySoonGame: "Eye break soon 🎮",
    overlayBreak: "Eye break!",
    overlayHintSoon: "Get ready to look out the window or into the distance for 5 minutes.",
    overlayHintGame: "Game detected — finish the round, then rest your eyes.",
    overlayHintBreak: "Look out the window or into the distance — not at the screen.",
    snooze: "Postpone +5 min",
    snoozeLeft: "Postpone +5 min ({n}× left)",
    breakNow: "Start break now",
    skip: "Skip",
    skipIn: "Skip (in {n}s)",
  },
};

let currentLanguage = "de";

export function setLanguage(lang) {
  currentLanguage = STRINGS[lang] ? lang : "de";
  document.documentElement.lang = currentLanguage;
}

export function t(key, params = {}) {
  let text = STRINGS[currentLanguage][key] ?? STRINGS.de[key] ?? key;
  for (const [name, value] of Object.entries(params)) {
    text = text.replace(`{${name}}`, value);
  }
  return text;
}

/// Übersetzt alle Elemente mit data-i18n="key" (Textinhalt).
export function applyTranslations() {
  for (const el of document.querySelectorAll("[data-i18n]")) {
    el.textContent = t(el.dataset.i18n);
  }
}
