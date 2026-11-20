//! Web engine icine enjekte edilen JavaScript ve CSS kaynaklari.
//!
//! Python surumundeki `engine/scripts.py` karsiligi (davranis esdeger).

/// Tercih edilen renk semasina gore koyu tema CSS'i.
pub const AUTO_DARK_MODE_CSS: &str = r#"
@media (prefers-color-scheme: dark) {
    :root { color-scheme: dark; }
    body { background-color: #1c1b22 !important; color: #fbfbfe !important; }
    a { color: #8ab4f8 !important; }
}
"#;

/// Yumusak kaydirma: tekerlek olaylarini yakalar ve yumusak animasyon uygular.
pub const SMOOTH_SCROLL_JS: &str = r#"
(function () {
  if (window.__swiftx_smooth) return;
  window.__swiftx_smooth = true;

  var TARGET_FPS = 60;
  var FRAME_MS = 1000 / TARGET_FPS;
  var EASE = 0.18;
  var current = 0;
  var target = 0;
  var running = false;

  function step() {
    var delta = target - current;
    if (Math.abs(delta) < 0.5) {
      current = target;
      window.scrollTo(0, current);
      running = false;
      return;
    }
    current += delta * EASE;
    window.scrollTo(0, current);
    setTimeout(step, FRAME_MS);
  }

  window.addEventListener('wheel', function (e) {
    if (e.ctrlKey || e.defaultPrevented) return;
    e.preventDefault();
    if (!running) current = window.scrollY || document.documentElement.scrollTop || 0;
    target = Math.max(0, current + e.deltaY);
    if (!running) {
      running = true;
      step();
    }
  }, { passive: false });
})();
"#;

/// Klavye ile yumusak kaydirma (bosluk, page up/down, ok tuslari).
pub const KEYBOARD_SCROLL_JS: &str = r#"
(function () {
  if (window.__swiftx_kb) return;
  window.__swiftx_kb = true;
  var KEYS = { 32: 600, 33: -600, 34: 600, 38: -120, 40: 120 };
  var DURATION = 220;

  function smoothBy(amount) {
    var start = window.scrollY || document.documentElement.scrollTop || 0;
    var goal = Math.max(0, start + amount);
    var t0 = performance.now();
    function frame(now) {
      var p = Math.min(1, (now - t0) / DURATION);
      var eased = p < 0.5 ? 2 * p * p : -1 + (4 - 2 * p) * p;
      window.scrollTo(0, start + (goal - start) * eased);
      if (p < 1) requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
  }

  window.addEventListener('keydown', function (e) {
    var tag = (e.target && e.target.tagName || '').toLowerCase();
    if (tag === 'input' || tag === 'textarea' || e.target.isContentEditable) return;
    var amount = KEYS[e.keyCode];
    if (amount === undefined) return;
    e.preventDefault();
    smoothBy(amount);
  });
})();
"#;

/// Otomatik koyu mod: sayfa arka planini ve metnini koyu temaya cevirir.
pub const AUTO_DARK_MODE_JS: &str = r#"
(function () {
    if (window.__auto_dark_mode) return;
    window.__auto_dark_mode = true;

    function applyDarkMode() {
        var href = location.href || '';
        if (href.startsWith('about:') || href.startsWith('data:')) return;
        var bg = getComputedStyle(document.body).backgroundColor;
        var isLight = bg === 'rgb(255, 255, 255)' || bg === 'rgba(0, 0, 0, 0)';
        if (!isLight) return;
        var style = document.createElement('style');
        style.id = '__swiftx_dark';
        style.textContent = 'html,body{background:#1c1b22 !important;color:#fbfbfe !important;}';
        (document.head || document.documentElement).appendChild(style);
    }

    if (document.readyState === 'complete' || document.readyState === 'interactive') {
        applyDarkMode();
    } else {
        document.addEventListener('DOMContentLoaded', applyDarkMode);
    }
})();
"#;

/// Sekme uyutuldugunde gosterilen yer tutucu sayfa.
pub const HIBERNATE_HTML: &str = r#"<html><head><style>
body { background: #1c1b22; display: flex; align-items: center; justify-content: center;
       height: 100vh; margin: 0; font-family: sans-serif; color: #9e9db5; flex-direction: column; gap: 8px; }
.big { font-size: 42px; } .msg { font-size: 14px; }
</style></head><body>
<div class="big">💤</div>
<div class="msg">Bu sekme uyutuldu, etkinlestirmek icin tiklayin.</div>
</body></html>"#;

/// Klavye kisayollarini yakalayip Rust cekirdegine ileten JS.
pub const HOTKEYS_JS: &str = r#"
(function () {
  if (window.__swiftx_hotkeys) return;
  window.__swiftx_hotkeys = true;
  function post(cmd) { try { window.ipc.postMessage(JSON.stringify(cmd)); } catch (e) {} }
  window.addEventListener('keydown', function (e) {
    var ctrl = e.ctrlKey || e.metaKey;
    if (ctrl && !e.shiftKey) {
      switch (e.key.toLowerCase()) {
        case 't': e.preventDefault(); post({cmd:'new_tab'}); return;
        case 'w': e.preventDefault(); post({cmd:'close_current'}); return;
        case 'l': e.preventDefault(); post({cmd:'focus_url'}); return;
        case 'd': e.preventDefault(); post({cmd:'add_bookmark'}); return;
        case 'h': e.preventDefault(); post({cmd:'toggle_history'}); return;
        case 'j': e.preventDefault(); post({cmd:'toggle_downloads'}); return;
        case 'b': e.preventDefault(); post({cmd:'toggle_sidebar'}); return;
        case 'r': e.preventDefault(); post({cmd:'reload'}); return;
        case 'f': e.preventDefault(); post({cmd:'find_open'}); return;
        case 's': e.preventDefault(); post({cmd:'export_bookmarks'}); return;
        case '=': case '+': e.preventDefault(); post({cmd:'zoom_in'}); return;
        case '-': e.preventDefault(); post({cmd:'zoom_out'}); return;
        case '0': e.preventDefault(); post({cmd:'zoom_reset'}); return;
      }
    }
    if (e.altKey) {
      if (e.key === 'ArrowLeft') { e.preventDefault(); post({cmd:'back'}); return; }
      if (e.key === 'ArrowRight') { e.preventDefault(); post({cmd:'forward'}); return; }
    }
    if (e.key === 'F5') { e.preventDefault(); post({cmd:'reload'}); return; }
    if (e.key === 'F11') { e.preventDefault(); post({cmd:'toggle_fullscreen'}); return; }
    if (e.key === 'F12') { e.preventDefault(); post({cmd:'toggle_devtools'}); return; }
    if (e.key === 'Escape') { post({cmd:'find_close'}); }
  }, true);
})();
"#;

/// Sayfa icinde metin arama (TreeWalker tabanli, tum platformlarda calisir).
pub const FIND_JS: &str = r#"
(function () {
  if (window.__swiftx_find_ready) return;
  window.__swiftx_find_ready = true;
  var marks = [];
  var idx = -1;
  var lastText = '';

  function clear() {
    for (var i = 0; i < marks.length; i++) {
      var m = marks[i];
      var p = m.parentNode;
      if (p) { p.replaceChild(document.createTextNode(m.textContent), m); p.normalize(); }
    }
    marks = []; idx = -1; lastText = '';
  }

  function draw() {
    for (var i = 0; i < marks.length; i++) {
      marks[i].style.background = (i === idx) ? '#ff9632' : '#ffe066';
      marks[i].style.color = '#000';
    }
  }

  function collect(text) {
    var lower = text.toLowerCase();
    var walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null);
    var nodes = []; var n;
    while ((n = walker.nextNode())) {
      var tag = n.parentNode && n.parentNode.nodeName;
      if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'NOSCRIPT' || tag === 'MARK') continue;
      if (n.nodeValue && n.nodeValue.toLowerCase().indexOf(lower) !== -1) nodes.push(n);
    }
    for (var k = 0; k < nodes.length; k++) {
      var node = nodes[k];
      var val = node.nodeValue; var lval = val.toLowerCase();
      var frag = document.createDocumentFragment(); var pos = 0; var i;
      while ((i = lval.indexOf(lower, pos)) !== -1) {
        if (i > pos) frag.appendChild(document.createTextNode(val.slice(pos, i)));
        var mark = document.createElement('mark');
        mark.textContent = val.slice(i, i + text.length);
        frag.appendChild(mark); marks.push(mark);
        pos = i + text.length;
      }
      if (pos < val.length) frag.appendChild(document.createTextNode(val.slice(pos)));
      if (node.parentNode) node.parentNode.replaceChild(frag, node);
    }
  }

  window.__swiftx_find_clear = clear;
  window.__swiftx_find = function (text, forward) {
    try {
      if (!text) { clear(); return; }
      if (text !== lastText) { clear(); lastText = text; collect(text); idx = -1; }
      if (marks.length === 0) return;
      if (forward === false) idx = (idx - 1 + marks.length) % marks.length;
      else idx = (idx + 1) % marks.length;
      draw();
      marks[idx].scrollIntoView({ block: 'center' });
    } catch (e) {}
  };
})();
"#;

/// Dahili self-test: chrome arayuzunu JS ile surer ve sonuclari Rust'a bildirir.
///
/// Yalnizca `SWIFTX_SELFTEST=1` iken enjekte edilir.
pub const SELFTEST_JS: &str = r#"
(function () {
  if (window.__swiftx_selftest) return;
  window.__swiftx_selftest = true;
  var results = [];
  function report(name, ok, extra) {
    results.push(name + '=' + (ok ? 'PASS' : 'FAIL') + (extra ? ' (' + extra + ')' : ''));
  }
  function post() {
    try { window.ipc.postMessage(JSON.stringify({ cmd: 'self_test', data: results.join(' | ') })); } catch (e) {}
  }
  function send(o) { try { window.ipc.postMessage(JSON.stringify(o)); } catch (e) {} }
  function st() { return (window.swiftx && window.swiftx._state) || {}; }
  var steps = [];
  function run() { if (steps.length) setTimeout(steps.shift(), 50); else post(); }
  function step(fn) { steps.push(fn); }

  step(function () { report('ipc_ready', !!window.swiftx, 'swiftx=' + !!window.swiftx); run(); });
  step(function () {
    var before = (st().tabs || []).length;
    var b = document.getElementById('newTabBtn');
    if (b) b.click(); else send({ cmd: 'new_tab' });
    setTimeout(function () {
      var after = (st().tabs || []).length;
      report('new_tab', after > before, before + '->' + after);
      run();
    }, 900);
  });
  step(function () {
    var tabCount = (st().tabs || []).length;
    send({ cmd: 'switch_tab', index: 0 });
    setTimeout(function () { report('switch_tab', (st().tabs || []).length === tabCount, 'tabs=' + tabCount); run(); }, 400);
  });
  step(function () {
    var u = document.getElementById('urlBar');
    u.value = 'example.com';
    u.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    setTimeout(function () { report('navigate_sent', true, 'url=' + (st().url || '')); run(); }, 1500);
  });
  step(function () {
    document.getElementById('histBtn').click();
    setTimeout(function () {
      var p = st().panel || {};
      report('panel_history', !!p.open && p.kind === 'history', 'kind=' + p.kind);
      run();
    }, 400);
  });
  step(function () {
    var before = !!st().settings && !!st().settings.dark_mode;
    send({ cmd: 'toggle_setting', name: 'dark_mode' });
    setTimeout(function () {
      var after = !!st().settings && !!st().settings.dark_mode;
      report('toggle_setting', before !== after, before + '->' + after);
      send({ cmd: 'toggle_setting', name: 'dark_mode' });
      run();
    }, 400);
  });
  step(function () { send({ cmd: 'add_bookmark' }); setTimeout(function () { report('add_bookmark_sent', true); run(); }, 500); });
  step(function () { report('summary', true, 'bitti'); post(); });

  setTimeout(run, 1500);
})();
"#;

/// Icerik web gorunumunun yuklendigini ve boyutunu bildiren teshis scripti.
pub const CONTENT_PROBE_JS: &str = r#"
(function () {
  if (window.__swiftx_probe) return;
  window.__swiftx_probe = true;
  function report() {
    try {
      window.ipc.postMessage(JSON.stringify({ cmd: 'probe', data: JSON.stringify({
        t: Date.now() % 100000,
        w: window.innerWidth,
        h: window.innerHeight,
        title: document.title || '',
        len: (document.body ? document.body.innerText.length : -1)
      }) }));
    } catch (e) {}
  }
  function schedule() { report(); setTimeout(report, 2000); setTimeout(report, 4500); }
  if (document.readyState === 'complete') { setTimeout(schedule, 400); }
  else { window.addEventListener('load', function () { setTimeout(schedule, 400); }); }
})();
"#;
