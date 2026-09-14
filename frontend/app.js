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
  populatePairingJudgeDropdown();
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
  populatePairingJudgeDropdown();
});

// ---- Roster import ----
const ROSTER_FIELDS = [
  { value: "", label: "Ignore this column" },
  { value: "first_name", label: "First name" },
  { value: "last_name", label: "Last name" },
  { value: "at_code", label: "A&T" },
  { value: "gym", label: "Gym" },
  { value: "taaf_athlete_number", label: "TAAF Athlete Competition #" },
  { value: "taaf_registration_number", label: "TAAF Registration #" },
  { value: "division", label: "Division" },
  { value: "qual_status", label: "Qual Status" },
];

let rosterImportPreview = null;

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

document.getElementById("roster-file-pick").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const path = await open({
    title: "Choose a roster file",
    multiple: false,
    directory: false,
    filters: [{ name: "Roster", extensions: ["xlsx", "xls", "csv", "pdf"] }],
  });
  if (!path) return;

  document.getElementById("roster-file-name").textContent = path.split("/").pop();
  rosterImportPreview = await safeInvoke("parse_roster_file", { path });
  renderRosterImportPreview();
});

function renderRosterImportPreview() {
  const container = document.getElementById("roster-import-preview");
  if (!rosterImportPreview) {
    container.classList.add("hidden");
    return;
  }
  container.classList.remove("hidden");
  document.getElementById("roster-import-notes").textContent = rosterImportPreview.notes.join(" ");

  const table = document.getElementById("roster-import-table");
  const thead = table.querySelector("thead");
  const tbody = table.querySelector("tbody");

  const headerCells = rosterImportPreview.headers
    .map((header, colIndex) => {
      const mapped = rosterImportPreview.column_mapping[colIndex] || "";
      const options = ROSTER_FIELDS.map(
        (f) => `<option value="${f.value}" ${f.value === mapped ? "selected" : ""}>${f.label}</option>`
      ).join("");
      return `<th>${escapeHtml(header)}<br/><select class="roster-col-map" data-col="${colIndex}">${options}</select></th>`;
    })
    .join("");
  thead.innerHTML = `<tr><th>Attending</th>${headerCells}<th>Warnings</th></tr>`;

  tbody.innerHTML = rosterImportPreview.rows
    .map((row, rowIndex) => {
      const warnings = (rosterImportPreview.row_warnings[rowIndex] || []).join(", ");
      const cells = row
        .map(
          (value, colIndex) =>
            `<td><input type="text" class="roster-cell" data-row="${rowIndex}" data-col="${colIndex}" value="${escapeHtml(value)}" /></td>`
        )
        .join("");
      return `<tr>
        <td><input type="checkbox" class="roster-attending" data-row="${rowIndex}" checked /></td>
        ${cells}
        <td style="color:#a00; font-size:12px;">${escapeHtml(warnings)}</td>
      </tr>`;
    })
    .join("");
}

document.getElementById("confirm-roster-import").addEventListener("click", async () => {
  if (!requireCompetition() || !rosterImportPreview) return;

  const colToField = {};
  document.querySelectorAll(".roster-col-map").forEach((sel) => {
    if (sel.value) colToField[Number(sel.dataset.col)] = sel.value;
  });
  if (!Object.values(colToField).includes("first_name") || !Object.values(colToField).includes("last_name")) {
    return alert("Map at least the First name and Last name columns before importing.");
  }

  const rows = [];
  for (let r = 0; r < rosterImportPreview.rows.length; r++) {
    const attending = document.querySelector(`.roster-attending[data-row="${r}"]`).checked;
    const values = {};
    for (const [col, field] of Object.entries(colToField)) {
      const input = document.querySelector(`.roster-cell[data-row="${r}"][data-col="${col}"]`);
      values[field] = input.value.trim();
    }
    if (!values.first_name && !values.last_name) continue;
    rows.push({
      firstName: values.first_name || "",
      lastName: values.last_name || "",
      atCode: values.at_code || null,
      gym: values.gym || null,
      taafAthleteNumber: values.taaf_athlete_number || null,
      taafRegistrationNumber: values.taaf_registration_number || null,
      division: values.division || null,
      qualStatus: values.qual_status || null,
      isAttending: attending,
    });
  }
  if (rows.length === 0) return alert("Nothing to import.");

  const summary = await safeInvoke("import_roster", { competitionId: activeCompetitionId, rows });
  document.getElementById("roster-import-status").textContent =
    `Created ${summary.created}, updated ${summary.updated}, skipped ${summary.skipped}.` +
    (summary.warnings.length ? ` Warnings: ${summary.warnings.join(" | ")}` : "");
  await refreshRoster();
});

// ---- Live judging (device pairing) ----
function populatePairingJudgeDropdown() {
  const select = document.getElementById("pairing-judge");
  select.innerHTML = judgesCache.map((j) => `<option value="${j.id}">${escapeHtml(j.name)} (${j.role})</option>`).join("");
}

document.getElementById("generate-pairing-code").addEventListener("click", async () => {
  const judgeId = document.getElementById("pairing-judge").value;
  if (!judgeId) return alert("Add a judge first (Roster tab).");
  const pairing = await safeInvoke("create_pairing_code", { judgeId });
  const expires = new Date(pairing.expires_at).toLocaleTimeString();
  document.getElementById("pairing-code-status").textContent =
    `Code: ${pairing.code} - enter this on the judge's iPad along with the server address below. Expires ${expires}.`;
});

document.getElementById("refresh-devices").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  const devices = await safeInvoke("list_judge_devices", { competitionId: activeCompetitionId });
  const tbody = document.querySelector("#devices-table tbody");
  tbody.innerHTML = devices
    .map((d) => {
      const judge = judgesCache.find((j) => j.id === d.judge_id);
      const paired = new Date(d.paired_at).toLocaleString();
      const lastSeen = d.last_seen_at ? new Date(d.last_seen_at).toLocaleString() : "never";
      return `<tr>
        <td>${escapeHtml(judge ? judge.name : d.judge_id)}</td>
        <td>${escapeHtml(d.device_name)}</td>
        <td>${paired}</td>
        <td>${lastSeen}</td>
        <td><button class="secondary" data-device-id="${d.id}">Revoke</button></td>
      </tr>`;
    })
    .join("");
  tbody.querySelectorAll("button[data-device-id]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      await safeInvoke("revoke_judge_device", { deviceId: btn.dataset.deviceId });
      document.getElementById("refresh-devices").click();
    });
  });
});

(async () => {
  const info = await safeInvoke("get_live_server_info");
  document.getElementById("live-server-address").textContent = info.lan_ip
    ? `Judges' iPads connect to: http://${info.lan_ip}:${info.port}`
    : `Could not detect this computer's network address automatically - find it in your OS network settings and use port ${info.port}.`;
})();

// ---- Rotation sheet ----
let rotationEntriesCache = [];

document.getElementById("refresh-rotation").addEventListener("click", async () => {
  if (!requireCompetition()) return;
  rotationEntriesCache = await safeInvoke("list_roster_with_gymnast_detail", { competitionId: activeCompetitionId });
  renderRotationSheet();
});
document.getElementById("rotation-group-by").addEventListener("change", renderRotationSheet);
document.getElementById("print-rotation").addEventListener("click", () => window.print());

function renderRotationSheet() {
  const container = document.getElementById("rotation-sheet-content");
  const groupBy = document.getElementById("rotation-group-by").value;
  const attending = rotationEntriesCache.filter((e) => e.participant.is_attending);

  const groups = new Map();
  for (const entry of attending) {
    const key =
      groupBy === "gym"
        ? (entry.gymnastDetail && entry.gymnastDetail.club) || "(no gym listed)"
        : entry.participant.level_or_category || "(no division listed)";
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(entry);
  }

  const sortedKeys = Array.from(groups.keys()).sort();
  container.innerHTML =
    sortedKeys
      .map((key) => {
        const rows = groups
          .get(key)
          .sort((a, b) => a.participant.last_name.localeCompare(b.participant.last_name))
          .map((entry) => {
            const detail = entry.gymnastDetail || {};
            return `<tr>
              <td>${escapeHtml(entry.participant.first_name)}</td>
              <td>${escapeHtml(entry.participant.last_name)}</td>
              <td>${escapeHtml(detail.club || "")}</td>
              <td>${escapeHtml(detail.taaf_athlete_number || "")}</td>
            </tr>`;
          })
          .join("");
        return `
          <h3>${escapeHtml(key)}</h3>
          <table>
            <thead><tr><th>First</th><th>Last</th><th>Gym</th><th>TAAF #</th></tr></thead>
            <tbody>${rows}</tbody>
          </table>
        `;
      })
      .join("") || "<p>No attending athletes to show. Import a roster or add participants first.</p>";
}

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
