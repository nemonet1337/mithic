// Common primitives for SNS wireframe screens
// Exposed on window so other Babel files can use them.

const { useState } = React;

// Skeleton paragraph: stack of varied-width text bars
function Lines({ count = 3, widths }) {
  const ws = widths || ['100%', '92%', '78%', '88%', '64%', '95%'];
  return (
    <div className="wf-stack" style={{ gap: 6 }}>
      {Array.from({ length: count }).map((_, i) => (
        <span key={i} className="wf-line" style={{ width: ws[i % ws.length] }} />
      ))}
    </div>
  );
}

// Faux post — "postmark" style: date-stamp postmark on the left, content on the right.
// Distinctive vs Twitter/Misskey: no top-row avatar+name; instead an entry header
// with a stamp glyph and bracketed metadata. Much more journal/zine-like.
function Post({ variant = 'card', handle = '@designer', name = 'Hana K.', time = '2m', date = 'MAY·10', text, media, accent, children }) {
  const t = text || '思考の断片を書き出す。今日のUIは少しだけ違う方向へ。';
  const flat = variant === 'flat';
  return (
    <article className={`wf-entry ${flat ? 'flat' : ''}`}>
      {/* Postmark stamp on the left edge */}
      <div className={`wf-stamp ${accent || ''}`}>
        <span className="wf-stamp-date">{date}</span>
        <span className="wf-stamp-time">{time}</span>
      </div>
      <div className="wf-grow" style={{ minWidth: 0 }}>
        <div className="wf-spread" style={{ marginBottom: 6 }}>
          <div className="wf-row" style={{ gap: 6, minWidth: 0 }}>
            <span className="wf-hand" style={{ fontSize: 17, lineHeight: 1 }}>{name}</span>
            <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>「{handle}」</span>
          </div>
          <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>···</span>
        </div>
        <div style={{ fontSize: 13.5, marginBottom: 8, lineHeight: 1.55 }}>
          {t}
        </div>
        {media && (
          <div className="wf-media" style={{ height: 120, marginBottom: 8 }}>{media}</div>
        )}
        {children}
        <div className="wf-row" style={{ gap: 6, color: 'var(--ink-3)', fontSize: 10 }}>
          <button className="wf-btn sm" style={{ padding: '3px 8px', gap: 4 }}>
            <span style={{ fontSize: 12 }}>＋</span>
            <span className="wf-mono" style={{ fontSize: 9, letterSpacing: '0.08em' }}>REACT</span>
          </button>
          <span className="wf-pill" style={{ fontFamily: 'var(--font-body)', fontSize: 10, padding: '2px 7px' }}>🔥 24</span>
          <span className="wf-pill" style={{ fontFamily: 'var(--font-body)', fontSize: 10, padding: '2px 7px' }}>✨ 18</span>
          <span className="wf-mono" style={{ marginLeft: 'auto', fontSize: 10 }}>↻ 47 · ↪ 12</span>
        </div>
      </div>
    </article>
  );
}

// Sidebar — mithic "spine": narrow rail with rotated mono labels on a vertical
// stripe. Distinctive signature: square-bracket logo stamp at top, vertical
// type ribbon along the gutter, dog-ear page-number footer.
function NavIcon({ name }) {
  const s = { width: 16, height: 16, fill: 'none', stroke: 'currentColor', strokeWidth: 1.5, strokeLinecap: 'round', strokeLinejoin: 'round' };
  switch (name) {
    case 'home':
      return (<svg viewBox="0 0 16 16" {...s}><path d="M2 7.5 8 2.5l6 5"/><path d="M3.5 7v6.5h9V7"/><path d="M6.5 13.5V10h3v3.5"/></svg>);
    case 'search':
      return (<svg viewBox="0 0 16 16" {...s}><circle cx="7" cy="7" r="4.2"/><path d="m10.2 10.2 3 3"/></svg>);
    case 'notif':
      return (<svg viewBox="0 0 16 16" {...s}><path d="M3.5 11.5h9L11.5 10V7a3.5 3.5 0 1 0-7 0v3l-1 1.5Z"/><path d="M6.5 13.5a1.5 1.5 0 0 0 3 0"/></svg>);
    case 'dm':
      return (<svg viewBox="0 0 16 16" {...s}><rect x="2" y="4" width="12" height="8.5" rx="1"/><path d="m2.5 4.5 5.5 4 5.5-4"/></svg>);
    case 'profile':
      return (<svg viewBox="0 0 16 16" {...s}><circle cx="8" cy="6" r="2.5"/><path d="M3.5 13.5c.5-2.4 2.4-3.5 4.5-3.5s4 1.1 4.5 3.5"/></svg>);
    case 'settings':
      return (<svg viewBox="0 0 16 16" {...s}><circle cx="8" cy="8" r="2"/><path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4"/></svg>);
    default:
      return null;
  }
}

function Sidebar({ active = 'home', logo = 'mithic', compact = false }) {
  const items = [
    ['home',     'ホーム',      '01'],
    ['search',   '検索',        '02'],
    ['notif',    '通知',        '03'],
    ['dm',       'メッセージ',  '04'],
    ['profile',  'プロフィール','05'],
    ['settings', '設定',        '06'],
  ];
  const wide = !compact;
  return (
    <aside className={`wf-spine ${wide ? '' : 'compact'}`} style={{ width: wide ? 156 : 48 }}>
      {/* Logo stamp */}
      <div className="wf-mark">
        <span className="wf-mark-bracket">[</span>
        <span className="wf-mark-glyph">m</span>
        <span className="wf-mark-bracket">]</span>
        {wide && <span className="wf-mark-text">{logo}</span>}
      </div>
      <div className="wf-spine-rule" />

      {/* Nav items */}
      <div className="wf-stack" style={{ gap: 2 }}>
        {items.map(([id, label, n]) => {
          const on = active === id;
          if (!wide) {
            return (
              <div key={id} className={`wf-spine-icon ${on ? 'on' : ''}`} title={label}>
                <NavIcon name={id} />
              </div>
            );
          }
          return (
            <div key={id} className={`wf-spine-item ${on ? 'on' : ''}`}>
              <span className="wf-spine-num">{n}</span>
              <span className="wf-spine-icon-inline"><NavIcon name={id} /></span>
              <span className="wf-spine-label">{label}</span>
              {on && <span className="wf-spine-marker">●</span>}
            </div>
          );
        })}
      </div>

      <div style={{ flex: 1 }} />

      {/* Compose stamp */}
      {wide ? (
        <button className="wf-stamp-btn">
          <span className="wf-mono" style={{ fontSize: 9, letterSpacing: '0.18em' }}>NEW</span>
          <span className="wf-hand" style={{ fontSize: 18, lineHeight: 1 }}>+ 投稿</span>
        </button>
      ) : (
        <button className="wf-stamp-btn compact" title="新しい投稿">
          <span className="wf-hand" style={{ fontSize: 20, lineHeight: 1 }}>+</span>
        </button>
      )}

      {/* Footer: account */}
      {wide ? (
        <div className="wf-spine-foot">
          <div className="wf-av sm accent" />
          <div className="wf-col wf-grow" style={{ minWidth: 0 }}>
            <span className="wf-mono" style={{ fontSize: 10, lineHeight: 1.1 }}>@you</span>
            <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>SIG · ok</span>
          </div>
        </div>
      ) : (
        <div className="wf-av sm accent" style={{ alignSelf: 'center', marginTop: 6 }} />
      )}
    </aside>
  );
}

// Right rail — "marginalia" with bracketed labels and tally-mark counts
function RightRail({ items = ['#design', '#ux', '#typography', '#wireframe'] }) {
  return (
    <aside className="wf-col" style={{ width: 240, padding: 14, gap: 12, borderLeft: '1.25px solid var(--line-soft)', flex: '0 0 auto' }}>
      <div className="wf-input dashed">
        <span className="wf-mono" style={{ marginRight: 6, color: 'var(--ink-3)' }}>⌕</span> find…
      </div>
      <div>
        <div className="wf-label" style={{ marginBottom: 6 }}>[ 急上昇 / TRENDING ]</div>
        <div className="wf-stack" style={{ gap: 6 }}>
          {items.map((t, i) => (
            <div key={t} className="wf-spread" style={{ padding: '4px 6px', borderBottom: '1px dashed var(--line-soft)' }}>
              <span className="wf-row" style={{ gap: 6 }}>
                <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>{String(i+1).padStart(2,'0')}.</span>
                <span className="wf-tag">{t}</span>
              </span>
              <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>{Math.floor(Math.random()*9)+1}.{Math.floor(Math.random()*9)}k</span>
            </div>
          ))}
        </div>
      </div>
      <div>
        <div className="wf-label" style={{ marginBottom: 6 }}>[ おすすめ / SUGGESTED ]</div>
        <div className="wf-stack" style={{ gap: 8 }}>
          {[1,2,3].map(i => (
            <div key={i} className="wf-row">
              <div className="wf-av sm" />
              <div className="wf-col wf-grow" style={{ minWidth: 0 }}>
                <span className="wf-hand" style={{ fontSize: 14, lineHeight: 1 }}>User {i}</span>
                <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>@user_{i}</span>
              </div>
              <button className="wf-btn sm">+ 追う</button>
            </div>
          ))}
        </div>
      </div>
      <div className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)', marginTop: 'auto', textAlign: 'right' }}>
        — mithic ·  signal not noise —
      </div>
    </aside>
  );
}

// Frame — provides the artboard canvas with theme + font wiring
function Frame({ children, dark = false, density = 'regular', label }) {
  return (
    <div className={`wf ${dark ? 'dark' : ''}`}>
      {children}
      {label && (
        <div style={{
          position: 'absolute', left: 8, bottom: 8,
          fontFamily: 'var(--font-mono)', fontSize: 10,
          color: 'var(--ink-3)', letterSpacing: '0.08em', textTransform: 'uppercase',
        }}>{label}</div>
      )}
    </div>
  );
}

// Top bar — bracket-stamped title with a folio number
function TopBar({ title, right, dense, folio }) {
  return (
    <div className="wf-spread" style={{
      padding: dense ? '10px 16px' : '14px 18px',
      borderBottom: '1.25px solid var(--line-soft)',
      background: 'var(--paper)',
    }}>
      <div className="wf-row" style={{ gap: 10, alignItems: 'baseline' }}>
        <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)', letterSpacing: '0.14em' }}>[ {folio || '01'} ]</span>
        <span className="wf-hand" style={{ fontSize: 24, lineHeight: 1 }}>{title}</span>
      </div>
      <div className="wf-row" style={{ gap: 6 }}>{right}</div>
    </div>
  );
}

Object.assign(window, { Lines, Post, Sidebar, RightRail, Frame, TopBar });
