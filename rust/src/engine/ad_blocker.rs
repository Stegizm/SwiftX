//! URL tabanli reklam ve izleyici engelleme.
//!
//! Python surumundeki `engine/ad_blocker.py` karsiligi.
//! WebKitGTK'nin Rust baglarinda `UserContentFilter` API'si acik olmadigi icin
//! hem navigasyon handler'i (ust-seviye) hem de enjekte edilen JS (alt-kaynaklar)
//! birlikte kullanilir.

use crate::core::constants::AD_BLOCK_PATTERNS;

/// Verilen URL reklam/izleyici kalibiyla eslesiyor mu?
///
/// YouTube ana sayfasi/oynaticisi engellenmez (kendi kalibi haric).
pub fn should_block(url: &str) -> bool {
    let lower = url.to_lowercase();
    if lower.contains("youtube.com") || lower.contains("youtu.be") {
        // YouTube'un kendi istatistik ucu haric engelleme.
        return lower.contains("youtube.com/api/stats");
    }
    AD_BLOCK_PATTERNS.iter().any(|p| lower.contains(p))
}

/// Alt kaynak (XHR / fetch / script / iframe / img) engelleme icin enjekte edilen JS.
pub fn ad_block_js() -> String {
    let patterns = AD_BLOCK_PATTERNS
        .iter()
        .map(|p| format!("{:?}", p))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"
(function () {{
  if (window.__swiftx_adblock) return;
  window.__swiftx_adblock = true;
  var PATTERNS = [{patterns}];
  function isAd(url) {{
    if (!url) return false;
    var u = String(url).toLowerCase();
    if (u.indexOf('youtube.com') !== -1 || u.indexOf('youtu.be') !== -1) return false;
    for (var i = 0; i < PATTERNS.length; i++) {{
      if (u.indexOf(PATTERNS[i]) !== -1) return true;
    }}
    return false;
  }}

  var rawOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method, url) {{
    this.__swiftx_blocked = isAd(url);
    return rawOpen.apply(this, arguments);
  }};
  var rawSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.send = function () {{
    if (this.__swiftx_blocked) return;
    return rawSend.apply(this, arguments);
  }};

  var rawFetch = window.fetch;
  if (rawFetch) {{
    window.fetch = function (input, init) {{
      var url = (typeof input === 'string') ? input : (input && input.url);
      if (isAd(url)) return Promise.reject(new Error('blocked'));
      return rawFetch.apply(this, arguments);
    }};
  }}

  function scrub() {{
    var els = document.querySelectorAll('iframe,script,img');
    for (var i = 0; i < els.length; i++) {{
      var src = els[i].getAttribute('src') || '';
      if (isAd(src)) {{
        els[i].style.display = 'none';
        try {{ els[i].remove(); }} catch (e) {{}}
      }}
    }}
  }}
  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', scrub);
  }} else {{
    scrub();
  }}
}})();
"#
    )
}

/// "Privacy Guard" eklentisi: izleyici parametrelerini temizleyen JS.
pub fn privacy_js() -> &'static str {
    r#"
(function () {
  if (window.__swiftx_privacy) return;
  window.__swiftx_privacy = true;
  try {
    var params = new URLSearchParams(location.search);
    var removed = false;
    ['utm_source','utm_medium','utm_campaign','utm_term','utm_content','fbclid','gclid'].forEach(function (k) {
      if (params.has(k)) { params.delete(k); removed = true; }
    });
    if (removed) {
      var q = params.toString();
      var clean = location.pathname + (q ? '?' + q : '') + location.hash;
      history.replaceState(null, '', clean);
    }
  } catch (e) {}
})();
"#
}

/// "Read Mode" eklentisi: makaleleri sade okuma moduna ceviren JS.
pub fn read_mode_js() -> &'static str {
    r#"
(function () {
  var id = '__swiftx_readmode';
  if (document.getElementById(id)) return;
  var article = document.querySelector('article') || document.querySelector('main') || document.body;
  if (!article) return;
  var clone = article.cloneNode(true);
  clone.querySelectorAll('nav,aside,footer,header,script,style,.ad,.ads,.sidebar').forEach(function (n) { n.remove(); });
  var style = document.createElement('style');
  style.id = id;
  style.textContent = 'body>*{display:none !important;} #swiftx_reader{display:block !important;max-width:760px;margin:40px auto;padding:0 20px;font-size:18px;line-height:1.7;}';
  var wrap = document.createElement('div');
  wrap.id = 'swiftx_reader';
  wrap.appendChild(clone);
  document.body.appendChild(style);
  document.body.appendChild(wrap);
})();
"#
}

/// "Page Speed" eklentisi: kritik olmayan gorselleri geciktirir.
pub fn speedup_js() -> &'static str {
    r#"
(function () {
  if (window.__swiftx_speedup) return;
  window.__swiftx_speedup = true;
  try {
    var imgs = document.querySelectorAll('img:not([loading])');
    for (var i = 0; i < imgs.length; i++) imgs[i].setAttribute('loading', 'lazy');
  } catch (e) {}
})();
"#
}
