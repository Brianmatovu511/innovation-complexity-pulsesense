const backendRaw = (localStorage.getItem("pulsesense_backend") || "http://127.0.0.1:8080").trim();

// Normalize backend URL: remove trailing slash(es)
const backend = backendRaw.replace(/\/+$/, "");
document.getElementById("backendUrl").textContent = backend;

// Build WS urls safely (try 127.0.0.1 first, then localhost fallback)
function toWs(url) {
  // http:// -> ws:// , https:// -> wss://
  return url.replace(/^http:\/\//i, "ws://").replace(/^https:\/\//i, "wss://");
}

const primaryWsUrl = toWs(backend) + "/ws/live";
const fallbackWsUrl = toWs(backend.replace("127.0.0.1", "localhost")) + "/ws/live";

const wsStatus = document.getElementById("wsStatus");
const lastUpdate = document.getElementById("lastUpdate");
const logEl = document.getElementById("log");

const series = {
  hr: [],
  temp: [],
  steps: [],
};

const maxPoints = 120; // ~1-2 minutes depending on simulator rate

function pushPoint(key, t, v) {
  const arr = series[key];
  arr.push({ t, v });
  if (arr.length > maxPoints) arr.shift();
}

function formatTs(ts) {
  const d = new Date(ts);
  return d.toLocaleTimeString();
}

function statusLabel(key, v) {
  if (key === "hr") {
    if (v < 50) return "Low";
    if (v > 120) return "High";
    return "Normal";
  }
  if (key === "temp") {
    if (v < 36.0) return "Low";
    if (v > 38.0) return "High";
    return "Normal";
  }
  if (key === "steps") {
    if (v === 0) return "Idle";
    if (v > 90) return "Active";
    return "Moving";
  }
  return "—";
}

function setStatus(id, label, value) {
  const el = document.getElementById(id);
  el.textContent = value === null ? "—" : `${label} (${value.toFixed(1)})`;
}

function addLog(obj) {
  const lines = logEl.textContent.trim().split("\n").filter(Boolean);
  lines.push(JSON.stringify(obj));
  while (lines.length > 5) lines.shift();
  logEl.textContent = lines.join("\n");
}

function makeChart(containerId, yLabel) {
  const el = document.getElementById(containerId);
  const width = el.clientWidth;
  const height = el.clientHeight;

  const svg = d3.select(el).append("svg")
    .attr("width", width)
    .attr("height", height);

  const margin = { top: 10, right: 14, bottom: 24, left: 42 };
  const innerW = width - margin.left - margin.right;
  const innerH = height - margin.top - margin.bottom;

  const g = svg.append("g").attr("transform", `translate(${margin.left},${margin.top})`);

  const x = d3.scaleTime().range([0, innerW]);
  const y = d3.scaleLinear().range([innerH, 0]);

  const xAxisG = g.append("g").attr("transform", `translate(0,${innerH})`);
  const yAxisG = g.append("g");

  g.append("text")
    .attr("x", 0)
    .attr("y", -2)
    .attr("fill", "currentColor")
    .attr("opacity", 0.7)
    .attr("font-size", 12)
    .text(yLabel);

  const path = g.append("path")
    .attr("fill", "none")
    .attr("stroke", "currentColor")
    .attr("stroke-width", 2)
    .attr("opacity", 0.9);

  function render(data) {
    if (!data.length) return;

    x.domain(d3.extent(data, d => d.t));
    const yMin = d3.min(data, d => d.v);
    const yMax = d3.max(data, d => d.v);
    const pad = (yMax - yMin) * 0.15 || 1;
    y.domain([yMin - pad, yMax + pad]);

    const line = d3.line()
      .x(d => x(d.t))
      .y(d => y(d.v));

    path.attr("d", line(data));

    xAxisG.call(d3.axisBottom(x).ticks(4));
    yAxisG.call(d3.axisLeft(y).ticks(5));
  }

  return { render };
}

const charts = {
  hr: makeChart("chart-hr", "bpm"),
  temp: makeChart("chart-temp", "°C"),
  steps: makeChart("chart-steps", "steps/min"),
};

function updateUI() {
  charts.hr.render(series.hr);
  charts.temp.render(series.temp);
  charts.steps.render(series.steps);

  const lastHr = series.hr.at(-1)?.v ?? null;
  const lastTemp = series.temp.at(-1)?.v ?? null;
  const lastSteps = series.steps.at(-1)?.v ?? null;

  if (lastHr !== null) setStatus("statusHr", statusLabel("hr", lastHr), lastHr); else setStatus("statusHr", "—", null);
  if (lastTemp !== null) setStatus("statusTemp", statusLabel("temp", lastTemp), lastTemp); else setStatus("statusTemp", "—", null);
  if (lastSteps !== null) setStatus("statusSteps", statusLabel("steps", lastSteps), lastSteps); else setStatus("statusSteps", "—", null);
}

// --- WebSocket connection with fallback + better status ---
let attempt = 0;

function connect() {
  const url = (attempt % 2 === 0) ? primaryWsUrl : fallbackWsUrl;
  attempt += 1;

  wsStatus.textContent = `connecting… (${url.includes("localhost") ? "localhost" : "127.0.0.1"})`;

  let ws;
  try {
    ws = new WebSocket(url);
  } catch (e) {
    wsStatus.textContent = "error (bad ws url)";
    setTimeout(connect, 800);
    return;
  }

  ws.onopen = () => {
    wsStatus.textContent = "connected";
  };

  ws.onmessage = (evt) => {
    let obj;
    try { obj = JSON.parse(evt.data); } catch { return; }

    if (obj.resourceType === "Observation") {
      const t = new Date(obj.effectiveDateTime);
      const v = obj.valueQuantity?.value;
      const codeText = (obj.code?.text || "").toLowerCase();

      if (typeof v === "number") {
        if (codeText.includes("heart")) pushPoint("hr", t, v);
        else if (codeText.includes("temperature")) pushPoint("temp", t, v);
        else if (codeText.includes("steps")) pushPoint("steps", t, v);
      }

      lastUpdate.textContent = formatTs(obj.effectiveDateTime);
      addLog(obj);
      updateUI();
    }
  };

  ws.onerror = () => {
    // onerror is often followed by onclose; keep it short
    wsStatus.textContent = "error";
  };

  ws.onclose = (e) => {
    // show useful close info
    wsStatus.textContent = `disconnected (${e.code}) retrying…`;
    setTimeout(connect, 800);
  };
}

connect();
