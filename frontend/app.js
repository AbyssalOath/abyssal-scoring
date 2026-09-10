const { invoke } = window.__TAURI__.core;
const { save, open } = window.__TAURI__.dialog;
const STORAGE_KEY = "abyssal:activeCompetitionId";

let activeCompetitionId = localStorage.getItem(STORAGE_KEY) || null;
let activeCompetitionSport = null;
let teamsCache = [];
let participantsCache = [];
let judgesCache = [];
let eventsCache = [];

// Every backend call goes through this so a validation error (or any
// other failure) always surfaces to the person instead of vanishing
// into an unhandled promise rejection in the console.
async function safeInvoke(command, args) {
  try {
    return await invoke(command, args);
  } catch (err) {
    alert(`Something went wrong: ${err}`);
    throw err;
  }
}

// ---- Nav ----
document.querySelectorAll("nav button").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll("nav button").forEach((b) => b.classList.remove("active"));
    document.querySelectorAll("section.panel").forEach((p) => p.classList.remove("active"));
    btn.classList.add("active");
    document.getElementById(btn.dataset.panel).classList.add("active");
  });
});

function setActiveCompetition(id, name, sport) {
  activeCompetitionId = id;
  activeCompetitionSport = sport;
  localStorage.setItem(STORAGE_KEY, id);
  document.getElementById("active-competition-label").textContent = name ? `Active: ${name}` : "";
  updateScoringVisibility();
  refreshRoster();
  refreshSchedule();
}

function updateScoringVisibility() {
  const isTimeTrial = activeCompetitionSport === "swimming" || activeCompetitionSport === "track_and_field";
  document.getElementById("gymnastics-scoring").classList.toggle("hidden", isTimeTrial || !activeCompetitionId);
  document.getElementById("time-trial-scoring").classList.toggle("hidden", !isTimeTrial || !activeCompetitionId);
}

function requireCompetition() {
  if (!activeCompetitionId) {
    alert("Create or select a competition first.");
    return false;
  }
  return true;
}

// ---- Competitions ----
async function refreshCompetitions() {
  const list = await safeInvoke("list_competitions");
  const tbody = document.querySelector("#competitions-table tbody");
  tbody.innerHTML = "";
  for (const c of list) {
    const tr = document.createElement("tr");
    tr.innerHTML = `<td><button data-id="${c.id}">Select</button></td><td>${c.name}</td><td>${c.sport}</td><td>${c.date}</td>`;
    tr.querySelector("button").addEventListener("click", () => setActiveCompetition(c.id, c.name, c.sport));
    tbody.appendChild(tr);
  }
  return list;
}

document.getElementById("create-competition").addEventListener("click", async () => {
  const name = document.getElementById("comp-name").value.trim();
  const sport = document.getElementById("comp-sport").value;
  const date = document.getElementById("comp-date").value.trim();
  if (!name) return alert("Competition name is required.");
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) return alert("Date must be in YYYY-MM-DD format.");

  const competition = await safeInvoke("create_competition", { name, sport, date });
  setActiveCompetition(competition.id, competition.name, competition.sport);
  await refreshCompetitions();
});

// ---- Roster ----
async function refreshRoster() {
  if (!activeCompetitionId) return;
  [teamsCache, participantsCache, judgesCache] = await Promise.all([
    safeInvoke("list_teams", { competitionId: activeCompetitionId }),
    safeInvoke("list_participants", { competitionId: activeCompetitionId }),
    safeInvoke("list_judges", { competitionId: activeCompetitionId }),
  ]);
  renderRosterTable();
  renderJudgePanelTable();
  populateLiveDropdowns();
}

function renderRosterTable() {
  const tbody = document.querySelector("#roster-table tbody");
  tbody.innerHTML = "";
  for (const t of teamsCache) addRosterRow(tbody, "Team", t.name, t.primary_color || "");
  for (const p of participantsCache) addRosterRow(tbody, "Participant", `${p.first_name} ${p.last_name}`, p.bib_number || "");
  for (const j of judgesCache) addRosterRow(tbody, "Judge", j.name, j.role);
}

function addRosterRow(tbody, type, name, detail) {
  const tr = document.createElement("tr");
  tr.innerHTML = `<td>${type}</td><td>${name}</td><td>${detail}</td>`;
  tbody.appendChild(tr);
}

document.getElementById("add-team").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const name = document.getElementById("team-name").value.trim();
  if (!name) return alert("Team name is required.");

  const team = await safeInvoke("add_team", { competitionId: activeCompetitionId, name });
  const primary = document.getElementById("team-primary").value;
  const secondary = document.getElementById("team-secondary").value;
  const logoPath = document.getElementById("team-logo").value.trim() || null;
  await safeInvoke("set_team_colors", { teamId: team.id, primary, secondary, logoPath });
  team.primary_color = primary;
  teamsCache.push(team);
  renderRosterTable();
});

document.getElementById("add-participant").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const firstName = document.getElementById("p-first").value.trim();
  const lastName = document.getElementById("p-last").value.trim();
  if (!firstName || !lastName) return alert("First and last name are both required.");

  const participant = await safeInvoke("add_participant", { competitionId: activeCompetitionId, firstName, lastName });
  participantsCache.push(participant);
  renderRosterTable();
  populateLiveDropdowns();
});

document.getElementById("add-judge").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const name = document.getElementById("j-name").value.trim();
  const roleSelect = document.getElementById("j-role").value;
  if (!name) return alert("Judge name is required.");

  const judge = await safeInvoke("add_judge", { competitionId: activeCompetitionId, name, role: roleSelect });
  judgesCache.push(judge);
  renderRosterTable();
  renderJudgePanelTable();
});

// ---- Schedule ----
async function refreshSchedule() {
  if (!activeCompetitionId) return;
  eventsCache = await safeInvoke("list_events", { competitionId: activeCompetitionId });
  renderEventsTable();
  populateLiveDropdowns();
}

function renderEventsTable() {
  const tbody = document.querySelector("#events-table tbody");
  tbody.innerHTML = "";
  for (const e of eventsCache) {
    const tr = document.createElement("tr");
    tr.innerHTML = `<td>${e.order_index}</td><td>${e.name}</td><td>${e.status}</td>`;
    tbody.appendChild(tr);
  }
}

document.getElementById("add-event").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const name = document.getElementById("event-name").value.trim();
  const orderIndex = Number(document.getElementById("event-order").value);
  if (!name) return alert("Event name is required.");
  if (!Number.isFinite(orderIndex)) return alert("Order must be a number.");

  const event = await safeInvoke("add_event", { competitionId: activeCompetitionId, name, orderIndex });
  eventsCache.push(event);
  renderEventsTable();
  populateLiveDropdowns();
});

function populateLiveDropdowns() {
  const eventSelect = document.getElementById("live-event");
  const participantSelect = document.getElementById("live-participant");
  eventSelect.innerHTML = eventsCache.map((e) => `<option value="${e.id}">${e.name}</option>`).join("");
  participantSelect.innerHTML = participantsCache
    .map((p) => `<option value="${p.id}">${p.first_name} ${p.last_name}</option>`)
    .join("");
}

function renderJudgePanelTable() {
  const tbody = document.querySelector("#judge-panel-table tbody");
  tbody.innerHTML = "";
  for (const j of judgesCache) {
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td><input type="checkbox" class="judge-panel-check" data-judge-id="${j.id}" /></td>
      <td>${j.name} (${j.role})</td>
      <td><input type="number" step="0.1" value="0" class="judge-panel-deduction" data-judge-id="${j.id}" style="width:80px" /></td>
    `;
    tbody.appendChild(tr);
  }
}

// ---- Live scoring ----
document.getElementById("set-current").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const eventId = document.getElementById("live-event").value;
  const participantId = document.getElementById("live-participant").value;
  if (!eventId || !participantId) return alert("Add an event and participant first.");
  await safeInvoke("set_current_participant", { competitionId: activeCompetitionId, eventId, participantId });
  document.getElementById("live-status").textContent = "On the board.";
});

document.getElementById("clear-board").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  await safeInvoke("clear_scoreboard", { competitionId: activeCompetitionId });
  document.getElementById("live-status").textContent = "Scoreboard cleared for next event. History untouched.";
});

document.getElementById("submit-score").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const eventId = document.getElementById("live-event").value;
  const participantId = document.getElementById("live-participant").value;
  const apparatus = document.getElementById("apparatus").value;
  const difficultyScore = Number(document.getElementById("difficulty").value);
  const neutralDeductions = Number(document.getElementById("neutral").value);
  if (!eventId || !participantId) return alert("Add an event and participant first.");
  if (!Number.isFinite(difficultyScore) || difficultyScore < 0) return alert("Difficulty must be zero or a positive number.");
  if (!Number.isFinite(neutralDeductions) || neutralDeductions < 0) return alert("Neutral deductions must be zero or a positive number.");

  const checked = Array.from(document.querySelectorAll(".judge-panel-check:checked"));
  let judgeScores;
  if (checked.length > 0) {
    judgeScores = [];
    for (const cb of checked) {
      const judgeId = cb.dataset.judgeId;
      const deductionInput = document.querySelector(`.judge-panel-deduction[data-judge-id="${judgeId}"]`);
      const total = Number(deductionInput.value);
      if (!Number.isFinite(total) || total < 0) return alert("Judge deduction totals must be zero or a positive number.");
      judgeScores.push({ judgeId, deductions: total > 0 ? [{ reason: "panel deduction total", value: total }] : [] });
    }
  } else {
    const fallbackTotal = Number(document.getElementById("deduction-total-fallback").value);
    if (!Number.isFinite(fallbackTotal) || fallbackTotal < 0) return alert("Deduction total must be zero or a positive number.");
    judgeScores = [{ judgeId: null, deductions: fallbackTotal > 0 ? [{ reason: "recorded deduction", value: fallbackTotal }] : [] }];
  }

  const result = await safeInvoke("submit_gymnastics_panel_score", {
    eventId,
    participantId,
    apparatus,
    difficultyScore,
    judgeScores,
    neutralDeductions,
  });

  await safeInvoke("show_result_on_scoreboard", {
    competitionId: activeCompetitionId,
    eventId,
    participantId,
    liveValue: result.detail,
  });
  document.getElementById("live-status").textContent = `Final score: ${result.final_value.toFixed(2)}`;
});

document.getElementById("submit-time-score").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const eventId = document.getElementById("live-event").value;
  const participantId = document.getElementById("live-participant").value;
  const elapsedSeconds = Number(document.getElementById("elapsed-seconds").value);
  const timePenaltySeconds = Number(document.getElementById("time-penalty").value);
  if (!eventId || !participantId) return alert("Add an event and participant first.");
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds < 0) return alert("Elapsed time must be zero or a positive number.");
  if (!Number.isFinite(timePenaltySeconds) || timePenaltySeconds < 0) return alert("Time penalty must be zero or a positive number.");

  const result = await safeInvoke("submit_swimming_result", {
    eventId,
    participantId,
    elapsedSeconds,
    timePenaltySeconds,
  });

  await safeInvoke("show_result_on_scoreboard", {
    competitionId: activeCompetitionId,
    eventId,
    participantId,
    liveValue: result.detail,
  });
  document.getElementById("live-status").textContent = `Final time: ${result.final_value.toFixed(2)}s`;
});

// ---- Scoreboard / theme ----
document.getElementById("open-display").addEventListener("click", () => safeInvoke("open_scoreboard_window", { fullscreen: false }));
document.getElementById("open-display-fullscreen").addEventListener("click", () => safeInvoke("open_scoreboard_window", { fullscreen: true }));

function bindImagePicker(buttonId, hiddenInputId, labelId) {
  document.getElementById(buttonId).addEventListener("click", async () => {
    const path = await open({
      title: "Choose an image",
      multiple: false,
      directory: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"] }],
    });
    if (!path) return;
    document.getElementById(hiddenInputId).value = path;
    document.getElementById(labelId).textContent = path.split("/").pop();
  });
}

bindImagePicker("team-logo-pick", "team-logo", "team-logo-name");
bindImagePicker("theme-bg-image-pick", "theme-bg-image", "theme-bg-image-name");
bindImagePicker("theme-logo-pick", "theme-logo", "theme-logo-name");

function bindColorSwatch(id) {
  const input = document.getElementById(id);
  const sync = () => { input.style.backgroundColor = input.value; };
  sync();
  input.addEventListener("input", sync);
  input.addEventListener("change", sync);
}

["team-primary", "team-secondary", "theme-bg", "theme-primary", "theme-accent"].forEach(bindColorSwatch);

document.getElementById("save-theme").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const theme = {
    id: crypto.randomUUID(),
    name: "Custom theme",
    preset: document.getElementById("theme-preset").value,
    background_color: document.getElementById("theme-bg").value,
    primary_color: document.getElementById("theme-primary").value,
    accent_color: document.getElementById("theme-accent").value,
    background_image_path: document.getElementById("theme-bg-image").value.trim() || null,
    logo_path: document.getElementById("theme-logo").value.trim() || null,
    font_family: '"Courier New", "Share Tech Mono", monospace',
    show_grid_overlay: document.getElementById("theme-grid").checked,
  };
  await safeInvoke("set_theme", { competitionId: activeCompetitionId, theme });
  alert("Theme saved. Reopen or refresh the scoreboard display to see it.");
});

// ---- Export / import ----
document.getElementById("do-export").addEventListener("click", async () => {
  if (!requireCompetition()) return;

  const competitions = await safeInvoke("list_competitions");
  const active = competitions.find((c) => c.id === activeCompetitionId);
  const namePart = (active ? active.name : "abyssal-export").replace(/[^a-z0-9-_]+/gi, "-");
  const suggestedName = `${namePart}-${new Date().toISOString().slice(0, 10)}.abyssal`;

  const path = await save({
    title: "Export Abyssal Scoring data",
    defaultPath: suggestedName,
    filters: [{ name: "Abyssal export", extensions: ["abyssal"] }],
  });
  if (!path) return;

  await safeInvoke("export_data", { path });
  document.getElementById("data-status").textContent = `Exported to ${path}`;
});

document.getElementById("do-import").addEventListener("click", async () => {
  const path = await open({
    title: "Import Abyssal Scoring data",
    multiple: false,
    directory: false,
    filters: [{ name: "Abyssal export", extensions: ["abyssal"] }],
  });
  if (!path) return;

  const mode = document.getElementById("import-mode").value;
  await safeInvoke("import_data", { path, mode });
  document.getElementById("data-status").textContent = `Imported from ${path}. Reselect your competition.`;
  await refreshCompetitions();
});

// ---- Boot ----
(async () => {
  const competitions = await refreshCompetitions();
  if (activeCompetitionId) {
    const restored = competitions.find((c) => c.id === activeCompetitionId);
    if (restored) {
      setActiveCompetition(restored.id, restored.name, restored.sport);
    } else {
      // Previously active competition no longer exists (e.g. a fresh
      // import) -- clear the stale pointer instead of leaving the UI
      // half-configured.
      activeCompetitionId = null;
      localStorage.removeItem(STORAGE_KEY);
    }
  }
})();
