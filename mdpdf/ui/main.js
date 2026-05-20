'use strict';

// ---------------------------------------------------------------------------
// Tauri API helpers (injected via withGlobalTauri: true)
// ---------------------------------------------------------------------------
const invoke  = (...a) => window.__TAURI__.core.invoke(...a);
const listen  = (...a) => window.__TAURI__.event.listen(...a);
const dialog  = window.__TAURI__.dialog;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------
let cmEditor             = null;
let currentTemplateCss   = '';
let currentTemplateValue = 'print-a4';
let isDirty              = false;
let renderRunning        = false;
let extractRunning       = false;

// ---------------------------------------------------------------------------
// Tab switching
// ---------------------------------------------------------------------------
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(s => s.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById(`tab-${btn.dataset.tab}`).classList.add('active');
    updateBottomBarVisibility();
  });
});

function updateBottomBarVisibility() {
  const activeTab = document.querySelector('.tab-btn.active')?.dataset.tab;
  const renderBtn = document.getElementById('render-btn');
  const extractBtn = document.getElementById('extract-btn');
  const renderLog = document.getElementById('render-log');
  const extractLog = document.getElementById('extract-log');
  const renderBar = document.getElementById('render-progress-bar');
  const extractBar = document.getElementById('extract-progress-bar');

  if (activeTab === 'render') {
    renderBtn.style.display = '';
    extractBtn.style.display = 'none';
    renderLog.style.display = '';
    extractLog.style.display = 'none';
    renderBar.style.display = '';
    extractBar.style.display = 'none';
  } else {
    renderBtn.style.display = 'none';
    extractBtn.style.display = '';
    renderLog.style.display = 'none';
    extractLog.style.display = '';
    renderBar.style.display = 'none';
    extractBar.style.display = '';
  }
}

// ---------------------------------------------------------------------------
// Dirty badge helper
// ---------------------------------------------------------------------------
function setDirty(val) {
  isDirty = val;
  const badge = document.getElementById('dirty-badge');
  badge.classList.toggle('hidden', !val);
}

// ---------------------------------------------------------------------------
// CodeMirror setup
// ---------------------------------------------------------------------------
function setupEditor() {
  const textarea = document.getElementById('css-editor');
  cmEditor = CodeMirror.fromTextArea(textarea, {
    mode          : 'css',
    theme         : 'dracula',
    lineNumbers   : true,
    lineWrapping  : false,
    tabSize       : 2,
    indentWithTabs: false,
    autofocus     : false,
  });

  cmEditor.on('change', () => {
    setDirty(cmEditor.getValue() !== currentTemplateCss);
  });
}

// ---------------------------------------------------------------------------
// Load template CSS from Rust and put it in the editor
// ---------------------------------------------------------------------------
async function loadTemplate(templateName) {
  const css = await invoke('get_template_css', { templateName });
  currentTemplateCss = css;
  cmEditor.setValue(css);
  cmEditor.clearHistory();
  setDirty(false);
  currentTemplateValue = templateName;
}

// ---------------------------------------------------------------------------
// Template selector
// ---------------------------------------------------------------------------
function setupTemplateSelector() {
  const sel = document.getElementById('template-select');

  sel.addEventListener('change', async e => {
    const newValue = e.target.value;

    if (isDirty) {
      const confirmed = await dialog.confirm(
        'You have unsaved CSS edits.\nSwitch to a different template and discard those changes?',
        { title: 'Unsaved CSS edits', kind: 'warning' }
      );
      if (!confirmed) {
        sel.value = currentTemplateValue;
        return;
      }
    }

    await loadTemplate(newValue);
  });
}

// ---------------------------------------------------------------------------
// File pickers
// ---------------------------------------------------------------------------
async function pickOpenFile(inputId, filters) {
  const selected = await dialog.open({ filters, multiple: false, directory: false });
  if (selected) {
    document.getElementById(inputId).value = selected;
  }
  return selected || null;
}

async function pickSaveFile(inputId, filters, defaultPath) {
  const path = await dialog.save({ filters, defaultPath });
  if (path) {
    document.getElementById(inputId).value = path;
  }
  return path || null;
}

function autoPopulateOutput(mdPath) {
  if (!mdPath) return;
  const pdfPath = mdPath.replace(/\.md$/i, '') + '.pdf';
  const out = document.getElementById('render-output');
  if (!out.value) {
    out.value = pdfPath;
  }
}

function setupFilePickers() {
  document.getElementById('render-input-browse').addEventListener('click', async () => {
    const chosen = await pickOpenFile('render-input', [
      { name: 'Markdown', extensions: ['md', 'markdown'] }
    ]);
    if (chosen) autoPopulateOutput(chosen);
  });

  document.getElementById('render-output-browse').addEventListener('click', () => {
    pickSaveFile('render-output', [{ name: 'PDF', extensions: ['pdf'] }]);
  });

  document.getElementById('extract-input-browse').addEventListener('click', () => {
    pickOpenFile('extract-input', [{ name: 'PDF', extensions: ['pdf'] }]);
  });

  document.getElementById('extract-output-browse').addEventListener('click', () => {
    pickSaveFile('extract-output', [{ name: 'Markdown', extensions: ['md', 'markdown'] }]);
  });
}

// ---------------------------------------------------------------------------
// Metadata entries
// ---------------------------------------------------------------------------
function addMetadataEntry(key, val) {
  const list = document.getElementById('metadata-list');
  const row = document.createElement('div');
  row.className = 'metadata-entry';
  row.innerHTML = `
    <input class="metadata-key" type="text" placeholder="Key" value="${escHtml(key)}" />
    <span class="metadata-eq">=</span>
    <input class="metadata-val" type="text" placeholder="Value" value="${escHtml(val)}" />
    <button class="metadata-remove" title="Remove">✕</button>
  `;
  row.querySelector('.metadata-remove').addEventListener('click', () => row.remove());
  list.appendChild(row);
  return row;
}

function escHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function collectMetadata() {
  const entries = [];
  document.querySelectorAll('.metadata-entry').forEach(row => {
    const k = row.querySelector('.metadata-key').value.trim();
    const v = row.querySelector('.metadata-val').value.trim();
    if (k) entries.push(`${k}=${v}`);
  });
  return entries;
}

function collectMetadataObjects() {
  const entries = [];
  document.querySelectorAll('.metadata-entry').forEach(row => {
    const k = row.querySelector('.metadata-key').value.trim();
    const v = row.querySelector('.metadata-val').value.trim();
    if (k) entries.push({ key: k, value: v });
  });
  return entries;
}

// ---------------------------------------------------------------------------
// Metadata preset dropdown
// ---------------------------------------------------------------------------
function setupMetadataPreset() {
  const sel = document.getElementById('metadata-preset');
  sel.addEventListener('change', () => {
    const val = sel.value;
    sel.value = '';
    if (!val) return;

    if (val === '__custom__') {
      const row = addMetadataEntry('', '');
      row.querySelector('.metadata-key').focus();
    } else {
      const row = addMetadataEntry(val, '');
      row.querySelector('.metadata-val').focus();
    }
  });
}

// ---------------------------------------------------------------------------
// Progress helpers
// ---------------------------------------------------------------------------
function resetProgress(barId, logId) {
  document.getElementById(barId).style.width = '0%';
  document.getElementById(logId).innerHTML   = '';
}

function appendLogLine(logId, text, cls) {
  const log  = document.getElementById(logId);
  const line = document.createElement('div');
  line.className = `log-line${cls ? ' ' + cls : ''}`;
  line.textContent = text;
  log.appendChild(line);
  log.scrollTop = log.scrollHeight;
}

function handleProgressEvent(barId, logId, payload) {
  document.getElementById(barId).style.width = `${payload.percent}%`;

  if (payload.error) {
    appendLogLine(logId, `✗ ${payload.error}`, 'log-error');
  } else if (payload.done) {
    appendLogLine(logId, `✓ ${payload.message}`, 'log-success');
  } else {
    appendLogLine(logId, `  ${payload.message}`, '');
  }
}

// ---------------------------------------------------------------------------
// Render button wiring
// ---------------------------------------------------------------------------
function setupRenderButton() {
  const btn = document.getElementById('render-btn');

  btn.addEventListener('click', async () => {
    if (renderRunning) return;

    const inputPath  = document.getElementById('render-input').value.trim();
    const outputPath = document.getElementById('render-output').value.trim();

    if (!inputPath)  { alert('Please select an input Markdown file.'); return; }
    if (!outputPath) { alert('Please specify an output PDF path.'); return; }

    const css = cmEditor.getValue();
    if (!css.trim()) { alert('The CSS editor is empty. Select a template or add styles.'); return; }

    const scale        = parseFloat(document.getElementById('opt-scale').value) || 1.0;
    const generateHtml = document.getElementById('opt-ghtml').checked;
    const allowHtml    = document.getElementById('opt-allow-html').checked;
    const onePage      = document.getElementById('opt-one-page').checked;
    const manualBreaks = document.getElementById('opt-manual-breaks').checked;

    if (onePage && manualBreaks) {
      alert('Single-page and Manual page breaks cannot be used together.');
      return;
    }

    const customMetadata = collectMetadata();

    renderRunning = true;
    btn.disabled  = true;
    resetProgress('render-progress-bar', 'render-log');

    try {
      await invoke('render_pdf', {
        params: {
          inputPath,
          outputPath,
          css,
          scale,
          generateHtml,
          allowHtml,
          onePage,
          manualBreaks,
          customMetadata,
        }
      });
    } catch (err) {
      const msg = (err && err.message) ? err.message : String(err);
      appendLogLine('render-log', `✗ ${msg}`, 'log-error');
      renderRunning = false;
      btn.disabled  = false;
    }
  });
}

// ---------------------------------------------------------------------------
// Extract button wiring
// ---------------------------------------------------------------------------
function setupExtractButton() {
  const btn = document.getElementById('extract-btn');

  btn.addEventListener('click', async () => {
    if (extractRunning) return;

    const inputPath  = document.getElementById('extract-input').value.trim();
    const outputPath = document.getElementById('extract-output').value.trim();

    if (!inputPath)  { alert('Please select an input PDF file.'); return; }
    if (!outputPath) { alert('Please specify an output Markdown path.'); return; }

    extractRunning = true;
    btn.disabled   = true;
    resetProgress('extract-progress-bar', 'extract-log');

    try {
      await invoke('extract_markdown', {
        params: { inputPath, outputPath }
      });
    } catch (err) {
      const msg = (err && err.message) ? err.message : String(err);
      appendLogLine('extract-log', `✗ ${msg}`, 'log-error');
      extractRunning = false;
      btn.disabled   = false;
    }
  });
}

// ---------------------------------------------------------------------------
// Config save/load/export/import
// ---------------------------------------------------------------------------
function gatherConfig() {
  return {
    template: currentTemplateValue,
    css: cmEditor.getValue(),
    scale: parseFloat(document.getElementById('opt-scale').value) || 1.0,
    generateHtml: document.getElementById('opt-ghtml').checked,
    allowHtml: document.getElementById('opt-allow-html').checked,
    onePage: document.getElementById('opt-one-page').checked,
    manualBreaks: document.getElementById('opt-manual-breaks').checked,
    metadata: collectMetadataObjects(),
  };
}

function applyConfig(config) {
  if (config.template) {
    document.getElementById('template-select').value = config.template;
    currentTemplateValue = config.template;
  }
  if (config.css != null) {
    cmEditor.setValue(config.css);
    cmEditor.clearHistory();
    currentTemplateCss = config.css;
    setDirty(false);
  }
  if (config.scale != null) {
    document.getElementById('opt-scale').value = config.scale;
  }
  if (config.generateHtml != null) {
    document.getElementById('opt-ghtml').checked = config.generateHtml;
  }
  if (config.allowHtml != null) {
    document.getElementById('opt-allow-html').checked = config.allowHtml;
  }
  if (config.onePage != null) {
    document.getElementById('opt-one-page').checked = config.onePage;
  }
  if (config.manualBreaks != null) {
    document.getElementById('opt-manual-breaks').checked = config.manualBreaks;
  }
  if (config.metadata && Array.isArray(config.metadata)) {
    document.getElementById('metadata-list').innerHTML = '';
    config.metadata.forEach(m => addMetadataEntry(m.key, m.value));
  }
}

async function autoSaveConfig() {
  try {
    await invoke('save_config', { config: gatherConfig() });
  } catch (_) {
    // Silent failure for auto-save
  }
}

async function loadSavedConfig() {
  try {
    const config = await invoke('load_config');
    if (config) {
      applyConfig(config);
    }
  } catch (_) {
    // No saved config or error — use defaults
  }
}

function setupConfigButtons() {
  document.getElementById('config-export').addEventListener('click', async () => {
    const path = await dialog.save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'mdpdf-config.json',
    });
    if (!path) return;
    try {
      await invoke('export_config', { config: gatherConfig(), path });
    } catch (err) {
      alert('Export failed: ' + (err.message || err));
    }
  });

  document.getElementById('config-import').addEventListener('click', async () => {
    const path = await dialog.open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      multiple: false,
      directory: false,
    });
    if (!path) return;
    try {
      const config = await invoke('import_config', { path });
      applyConfig(config);
    } catch (err) {
      alert('Import failed: ' + (err.message || err));
    }
  });
}

// Auto-save config periodically and on window close
let autoSaveTimer = null;
function scheduleAutoSave() {
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  autoSaveTimer = setTimeout(autoSaveConfig, 2000);
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------
async function init() {
  setupEditor();
  setupTemplateSelector();
  setupFilePickers();
  setupMetadataPreset();
  setupConfigButtons();

  // Register progress listeners
  await Promise.all([
    listen('render_progress',  e => {
      handleProgressEvent('render-progress-bar', 'render-log', e.payload);
      if (e.payload.done || e.payload.error) {
        renderRunning = false;
        document.getElementById('render-btn').disabled = false;
      }
    }),
    listen('extract_progress', e => {
      handleProgressEvent('extract-progress-bar', 'extract-log', e.payload);
      if (e.payload.done || e.payload.error) {
        extractRunning = false;
        document.getElementById('extract-btn').disabled = false;
      }
    }),
  ]);

  setupRenderButton();
  setupExtractButton();

  // Load saved config or fall back to default template
  const config = await invoke('load_config').catch(() => null);
  if (config) {
    applyConfig(config);
  } else {
    await loadTemplate('print-a4');
  }

  updateBottomBarVisibility();

  // Watch for changes to auto-save
  const observer = new MutationObserver(scheduleAutoSave);
  observer.observe(document.getElementById('metadata-list'), { childList: true, subtree: true });

  // Auto-save on relevant input changes
  document.querySelectorAll('#opt-scale, #opt-ghtml, #opt-allow-html, #opt-one-page, #opt-manual-breaks')
    .forEach(el => el.addEventListener('change', scheduleAutoSave));
  document.getElementById('template-select').addEventListener('change', scheduleAutoSave);

  if (cmEditor) {
    cmEditor.on('change', scheduleAutoSave);
  }

  // Save on close
  window.addEventListener('beforeunload', () => autoSaveConfig());
}

document.addEventListener('DOMContentLoaded', init);

