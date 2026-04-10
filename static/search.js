(function () {
  var searchData = null;
  var pendingQuery = null;

  // ── Modal elements ──────────────────────────────────────────────────────────
  var overlay  = document.getElementById('search-overlay');
  var modalInput = document.getElementById('search-modal-input');
  var modalResults = document.getElementById('search-modal-results');

  // ── Sidebar elements (legacy inline search – kept for sidebar UX) ───────────
  var sideInput   = document.getElementById('search-input');
  var sideResults = document.getElementById('search-results');

  // Load search index once
  fetch('/search.json')
    .then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    })
    .then(function (data) {
      searchData = data;
      if (pendingQuery !== null) {
        runSearch(pendingQuery, modalResults);
        pendingQuery = null;
      }
    })
    .catch(function (e) {
      console.warn('[search] failed to load search.json:', e);
    });

  // ── Open / close modal ───────────────────────────────────────────────────────
  function openModal() {
    if (!overlay) return;
    overlay.classList.remove('hidden');
    modalInput.value = '';
    modalResults.innerHTML = '';
    modalInput.focus();
  }

  function closeModal() {
    if (!overlay) return;
    overlay.classList.add('hidden');
    modalResults.innerHTML = '';
  }

  // Ctrl+K / ⌘K global shortcut
  document.addEventListener('keydown', function (e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      if (overlay && overlay.classList.contains('hidden')) {
        openModal();
      } else {
        closeModal();
      }
    }
    if (e.key === 'Escape') {
      closeModal();
    }
  });

  // Click outside the modal panel → close
  if (overlay) {
    overlay.addEventListener('mousedown', function (e) {
      if (e.target === overlay) closeModal();
    });
  }

  // ── Sidebar Ctrl+K hint click ────────────────────────────────────────────────
  var hint = document.getElementById('search-shortcut-hint');
  if (hint) {
    hint.addEventListener('click', function () { openModal(); });
  }
  // Also open when sidebar input is focused
  if (sideInput) {
    sideInput.addEventListener('focus', function () { openModal(); });
  }

  // ── Modal input → search ─────────────────────────────────────────────────────
  if (modalInput) {
    modalInput.addEventListener('input', function () {
      var q = modalInput.value.trim();
      if (!q) { modalResults.innerHTML = ''; return; }
      if (!searchData) {
        pendingQuery = q;
        modalResults.innerHTML = '<div class="sr-empty">加载中…</div>';
        return;
      }
      runSearch(q, modalResults);
    });

    // ↑ ↓ keyboard navigation inside modal results
    modalInput.addEventListener('keydown', function (e) {
      var items = modalResults.querySelectorAll('.search-result-item');
      if (!items.length) return;
      var active = modalResults.querySelector('.search-result-item.kbd-active');
      var idx = -1;
      items.forEach(function (el, i) { if (el === active) idx = i; });

      if (e.key === 'ArrowDown') {
        e.preventDefault();
        var next = idx + 1 < items.length ? idx + 1 : 0;
        setActive(items, next);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        var prev = idx - 1 >= 0 ? idx - 1 : items.length - 1;
        setActive(items, prev);
      } else if (e.key === 'Enter') {
        if (active && active.dataset.url) {
          window.location.href = active.dataset.url;
        } else if (items.length === 1) {
          window.location.href = items[0].dataset.url;
        }
      }
    });
  }

  function setActive(items, idx) {
    items.forEach(function (el) { el.classList.remove('kbd-active'); });
    items[idx].classList.add('kbd-active');
    items[idx].scrollIntoView({ block: 'nearest' });
  }

  // Navigate on click inside modal results
  if (modalResults) {
    modalResults.addEventListener('mousedown', function (e) {
      var item = e.target.closest('.search-result-item');
      if (item && item.dataset.url) {
        e.preventDefault();
        window.location.href = item.dataset.url;
      }
    });
  }

  // ── Sidebar legacy search (unchanged behavior) ───────────────────────────────
  if (sideInput && sideResults) {
    function positionResults() {
      var rect = sideInput.getBoundingClientRect();
      sideResults.style.top   = (rect.bottom + window.scrollY + 4) + 'px';
      sideResults.style.left  = (rect.left + window.scrollX) + 'px';
      sideResults.style.width = rect.width + 'px';
    }

    sideInput.addEventListener('input', function () {
      var q = sideInput.value.trim();
      if (!q) { hideSide(); return; }
      if (!searchData) {
        pendingQuery = q;
        positionResults();
        sideResults.innerHTML = '<div style="padding:10px 14px;color:#57606a;font-size:13px">加载中…</div>';
        sideResults.classList.remove('hidden');
        return;
      }
      runSearchInto(q, sideResults);
      positionResults();
      sideResults.classList.remove('hidden');
    });

    sideResults.addEventListener('mousedown', function (e) {
      var item = e.target.closest('.search-result-item');
      if (item && item.dataset.url) {
        e.preventDefault();
        window.location.href = item.dataset.url;
      }
    });

    document.addEventListener('click', function (e) {
      if (e.target !== sideInput && !sideResults.contains(e.target)) hideSide();
    });

    window.addEventListener('resize', function () {
      if (!sideResults.classList.contains('hidden')) positionResults();
    });

    function hideSide() {
      sideResults.classList.add('hidden');
      sideResults.innerHTML = '';
    }
  }

  // ── Core search ───────────────────────────────────────────────────────────────
  function runSearch(q, container) {
    runSearchInto(q, container);
  }

  function runSearchInto(rawQuery, container) {
    var expr;
    try {
      expr = parseQuery(rawQuery.trim().toLowerCase());
    } catch (e) {
      container.innerHTML = '<div class="sr-empty">查询语法错误</div>';
      return;
    }

    var hits = searchData.filter(function (item) {
      return evalExpr(expr, item);
    }).slice(0, 20);

    if (hits.length === 0) {
      container.innerHTML = '<div class="sr-empty">无结果</div>';
      return;
    }

    container.innerHTML = hits.map(function (h) {
      var desc = h.description || (h.body ? h.body.slice(0, 100) : '');
      return '<div class="search-result-item" data-url="' + escAttr(h.url) + '">' +
        '<div class="s-title">' + escHtml(h.title) + '</div>' +
        (desc ? '<div class="s-desc">' + escHtml(desc) + '</div>' : '') +
        '</div>';
    }).join('');
  }

  // ── Logical query parser (recursive descent) ─────────────────────────────────
  // Grammar:
  //   expr    → or_expr
  //   or_expr → and_expr ( ('||') and_expr )*
  //   and_expr→ not_expr ( ('&&' | '&' | ' ') not_expr )*   (implicit AND for space)
  //   not_expr→ '!' primary | primary
  //   primary → '(' expr ')' | term
  //   term    → sequence of non-operator chars (trimmed)

  function parseQuery(input) {
    var pos = 0;

    function peek(str) {
      return input.slice(pos, pos + str.length) === str;
    }
    function consume(str) {
      pos += str.length;
    }
    function skipSpaces() {
      while (pos < input.length && input[pos] === ' ') pos++;
    }

    function parseExpr() { return parseOr(); }

    function parseOr() {
      var left = parseAnd();
      skipSpaces();
      while (peek('||')) {
        consume('||');
        skipSpaces();
        var right = parseAnd();
        left = { op: 'or', left: left, right: right };
        skipSpaces();
      }
      return left;
    }

    function parseAnd() {
      var left = parseNot();
      skipSpaces();
      // explicit && / & or implicit space (but not before '||' or ')')
      while (pos < input.length && !peek('||') && input[pos] !== ')') {
        var savedPos = pos;
        var explicit = false;
        if (peek('&&')) { consume('&&'); explicit = true; }
        else if (peek('&')) { consume('&'); explicit = true; }
        skipSpaces();
        if (pos >= input.length || peek('||') || input[pos] === ')') {
          // nothing follows — rollback
          pos = savedPos;
          break;
        }
        // If next char is '!' or '(' or a word char — implicit or explicit AND
        var right = parseNot();
        left = { op: 'and', left: left, right: right };
        skipSpaces();
      }
      return left;
    }

    function parseNot() {
      skipSpaces();
      if (peek('!')) {
        consume('!');
        skipSpaces();
        var operand = parsePrimary();
        return { op: 'not', operand: operand };
      }
      return parsePrimary();
    }

    function parsePrimary() {
      skipSpaces();
      if (peek('(')) {
        consume('(');
        var inner = parseExpr();
        skipSpaces();
        if (peek(')')) consume(')');
        return inner;
      }
      return parseTerm();
    }

    function parseTerm() {
      skipSpaces();
      var start = pos;
      // Consume until operator or end
      while (pos < input.length) {
        var ch = input[pos];
        if (ch === '(' || ch === ')' || ch === '!') break;
        if (peek('||') || peek('&&') || (ch === '&' && pos + 1 < input.length && input[pos+1] !== '&')) {
          // single & is also an operator — stop
          break;
        }
        if (ch === '&') break;
        if (ch === ' ') break;
        pos++;
      }
      var term = input.slice(start, pos).trim();
      if (!term) return { op: 'term', value: '' };
      return { op: 'term', value: term };
    }

    var result = parseExpr();
    return result;
  }

  function evalExpr(expr, item) {
    if (expr.op === 'term') {
      if (!expr.value) return true;
      return termMatch(item, expr.value);
    }
    if (expr.op === 'not') return !evalExpr(expr.operand, item);
    if (expr.op === 'and') return evalExpr(expr.left, item) && evalExpr(expr.right, item);
    if (expr.op === 'or')  return evalExpr(expr.left, item) || evalExpr(expr.right, item);
    return false;
  }

  function termMatch(item, q) {
    return wordMatch(item.title, q) ||
      tagMatch(item.tags, q) ||
      (item.description && wordMatch(item.description, q)) ||
      (item.body && wordMatch(item.body, q));
  }

  // Word-boundary match for ASCII; substring for CJK
  function wordMatch(text, q) {
    if (!text) return false;
    var t = text.toLowerCase();
    var idx = t.indexOf(q);
    if (idx === -1) return false;
    if (/^[a-z0-9]+$/.test(q)) {
      while (idx !== -1) {
        var prevChar = idx === 0 ? '' : t[idx - 1];
        if (!/[a-z0-9_]/.test(prevChar)) return true;
        idx = t.indexOf(q, idx + 1);
      }
      return false;
    }
    return true;
  }

  function tagMatch(tags, q) {
    if (!tags) return false;
    return tags.some(function (t) { return wordMatch(t, q); });
  }

  // ── Helpers ───────────────────────────────────────────────────────────────────
  function escHtml(s) {
    return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
  }
  function escAttr(s) {
    return String(s).replace(/&/g,'&amp;').replace(/"/g,'&quot;');
  }
})();
