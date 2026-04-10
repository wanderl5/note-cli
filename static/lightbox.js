(function () {
  // Lightbox
  var overlay = document.createElement('div');
  overlay.className = 'lightbox-overlay';
  var lbImg = document.createElement('img');
  overlay.appendChild(lbImg);
  document.body.appendChild(overlay);

  overlay.addEventListener('click', function () {
    overlay.classList.remove('active');
  });

  document.querySelectorAll('.article-body img').forEach(function (el) {
    el.style.cursor = 'zoom-in';
    el.addEventListener('click', function () {
      lbImg.src = el.src;
      overlay.classList.add('active');
    });
  });

  // Copy buttons for code blocks
  document.querySelectorAll('.code-block').forEach(function (pre) {
    var btn = document.createElement('button');
    btn.className = 'copy-btn';
    btn.textContent = '复制';
    btn.addEventListener('click', function (e) {
      e.stopPropagation();
      var code = pre.querySelector('code');
      var text = code ? code.innerText : pre.innerText;
      if (navigator.clipboard) {
        navigator.clipboard.writeText(text).then(function () {
          btn.textContent = '已复制 ✓';
          setTimeout(function () { btn.textContent = '复制'; }, 1500);
        });
      } else {
        // Fallback
        var ta = document.createElement('textarea');
        ta.value = text;
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
        btn.textContent = '已复制 ✓';
        setTimeout(function () { btn.textContent = '复制'; }, 1500);
      }
    });
    pre.appendChild(btn);
  });
})();
