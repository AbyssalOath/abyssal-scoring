const { invoke } = window.__TAURI__.core;

const STORAGE_KEY = "abyssal:activeCompetitionId";
const POLL_MS = 500;

let lastThemeId = null;

function applyTheme(theme) {
  if (theme.id === lastThemeId) return;
  lastThemeId = theme.id;

  const root = document.documentElement;
  root.style.setProperty("--abyssal-bg", theme.background_color);
  root.style.setProperty("--abyssal-primary", theme.primary_color);
  root.style.setProperty("--abyssal-accent", theme.accent_color);
  root.style.setProperty("--abyssal-font", theme.font_family);
  root.style.setProperty(
    "--abyssal-bg-image",
    theme.background_image_path ? `url("${toAssetUrl(theme.background_image_path)}")` : "none"
  );
  document.body.classList.toggle("grid-overlay", !!theme.show_grid_overlay);

  const logo = document.getElementById("logo-corner");
  if (theme.logo_path) {
    logo.src = toAssetUrl(theme.logo_path);
    logo.classList.remove("hidden");
  } else {
    logo.classList.add("hidden");
  }
}

// Local file paths need the Tauri asset protocol to be loadable from
// an <img src>. Swap this out if you instead copy logos into
// frontend/assets and serve them as regular relative URLs.
function toAssetUrl(path) {
  return window.__TAURI__.core.convertFileSrc(path);
}

function showIdle() {
  document.getElementById("idle-message").classList.remove("hidden");
  document.getElementById("event-name").classList.add("hidden");
  document.getElementById("participant-name").classList.add("hidden");
  document.getElementById("score").classList.add("hidden");
}

function showLive(eventName, participantName, scoreText) {
  document.getElementById("idle-message").classList.add("hidden");
  const eventEl = document.getElementById("event-name");
  const nameEl = document.getElementById("participant-name");
  const scoreEl = document.getElementById("score");
  eventEl.textContent = eventName;
  nameEl.textContent = participantName;
  scoreEl.textContent = scoreText;
  eventEl.classList.remove("hidden");
  nameEl.classList.remove("hidden");
  scoreEl.classList.remove("hidden");
}

async function tick() {
  const competitionId = localStorage.getItem(STORAGE_KEY);
  if (!competitionId) {
    showIdle();
    return;
  }

  try {
    const [state, theme] = await Promise.all([
      invoke("get_scoreboard_state", { competitionId }),
      invoke("resolve_theme", { competitionId }),
    ]);
    applyTheme(theme);

    if (state.display_mode === "idle" || !state.current_participant_id) {
      showIdle();
      return;
    }

    const [participant, event] = await Promise.all([
      invoke("get_participant", { id: state.current_participant_id }),
      state.current_event_id ? invoke("get_event", { id: state.current_event_id }) : Promise.resolve(null),
    ]);

    const scoreText =
      state.display_mode === "showing_result" && state.live_value && state.live_value.final_score !== undefined
        ? Number(state.live_value.final_score).toFixed(2)
        : "--.--";

    showLive(event ? event.name : "", `${participant.first_name} ${participant.last_name}`, scoreText);
  } catch (err) {
    console.error("scoreboard poll failed", err);
  }
}

setInterval(tick, POLL_MS);
tick();
