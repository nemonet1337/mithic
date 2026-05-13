// Notifications — 3 variations

function NotifA() {
  // V1: Classic list
  return (
    <Frame label="A · classic list">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="notif" />
        <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
          <TopBar title="通知" right={<button className="wf-btn sm ghost">すべて既読</button>} />
          <div className="wf-tabs" style={{ padding: '0 16px' }}>
            <span className="t on">すべて</span>
            <span className="t">@メンション</span>
            <span className="t">いいね</span>
            <span className="t">フォロー</span>
          </div>
          <div className="wf-stack" style={{ padding: 8, gap: 0, overflow: 'hidden' }}>
            {[
              ['Riku M.','@riku','があなたの投稿に♥','2m', true,'accent'],
              ['Aya T.','@aya','がリノートしました','14m', true,''],
              ['Ken S.','@ken_s','があなたをフォロー','1h', false,'accent2'],
              ['Hana K.','@hana','があなたを引用','2h', false,''],
              ['Riku M.','@riku','が返信','3h', false,''],
            ].map((n, i) => (
              <div key={i} className="wf-row" style={{ padding: 12, gap: 10, borderBottom: '1.25px solid var(--line-soft)', background: n[4] ? 'rgba(255,61,139,0.06)' : 'transparent' }}>
                {n[4] && <div style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--accent)' }} />}
                <div className={`wf-av sm ${n[5]}`} />
                <div className="wf-grow" style={{ fontSize: 13 }}>
                  <b className="wf-hand" style={{ fontSize: 15 }}>{n[0]}</b>
                  <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)', marginLeft: 4 }}>{n[1]}</span>
                  <span style={{ marginLeft: 4 }}>{n[2]}</span>
                </div>
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>{n[3]}</span>
              </div>
            ))}
          </div>
        </main>
        <RightRail />
      </div>
    </Frame>
  );
}

function NotifB() {
  // V2: Grouped — by activity type
  return (
    <Frame label="B · grouped by type">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="notif" compact />
        <main className="wf-grow" style={{ minWidth: 0, padding: 20, overflow: 'hidden' }}>
          <div className="wf-spread" style={{ marginBottom: 14 }}>
            <h1 className="wf-hand" style={{ fontSize: 30, margin: 0 }}>通知 <span style={{ color: 'var(--accent)' }}>· 12</span></h1>
            <button className="wf-btn ghost sm">フィルター</button>
          </div>
          {[
            ['いいね · 7件', 'accent', '♥', [['Riku M.','2m'],['Aya T.','5m'],['Ken S.','12m'],['+4','22m']]],
            ['フォロー · 3件', 'accent2', '+', [['Ken S.','1h'],['Mana','2h'],['Yui','5h']]],
            ['返信・引用 · 2件', '', '↪', [['Riku M.','30m'],['Hana K.','2h']]],
          ].map(([title, ac, glyph, items], i) => (
            <div key={i} className="wf-card" style={{ padding: 14, marginBottom: 10 }}>
              <div className="wf-spread" style={{ marginBottom: 8 }}>
                <div className="wf-row" style={{ gap: 8 }}>
                  <span className={`wf-pill ${ac}`} style={{ fontSize: 12, padding: '3px 8px' }}>{glyph}</span>
                  <span className="wf-hand" style={{ fontSize: 18 }}>{title}</span>
                </div>
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>展開 ↓</span>
              </div>
              <div className="wf-row" style={{ gap: 10, flexWrap: 'wrap' }}>
                {items.map(([n, t]) => (
                  <div key={n} className="wf-row" style={{ gap: 6, padding: '4px 8px', border: '1px solid var(--line-soft)', borderRadius: 999 }}>
                    <div className="wf-av sm" style={{ width: 18, height: 18 }} />
                    <span className="wf-hand" style={{ fontSize: 13 }}>{n}</span>
                    <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>{t}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </main>
      </div>
    </Frame>
  );
}

function NotifC() {
  // V3: Activity stream with rich previews
  return (
    <Frame label="C · activity stream">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="notif" />
        <main className="wf-grow wf-col" style={{ minWidth: 0, padding: 16, overflow: 'hidden' }}>
          <div className="wf-spread" style={{ marginBottom: 12 }}>
            <span className="wf-hand" style={{ fontSize: 24 }}>アクティビティ</span>
            <div className="wf-row" style={{ gap: 6 }}>
              <span className="wf-pill accent">未読 5</span>
              <button className="wf-btn sm ghost">既読に</button>
            </div>
          </div>
          <div className="wf-stack" style={{ gap: 10 }}>
            {/* Like with preview */}
            <div className="wf-card" style={{ padding: 12 }}>
              <div className="wf-row" style={{ gap: 8, marginBottom: 8 }}>
                <div className="wf-av sm accent" />
                <span className="wf-hand" style={{ fontSize: 14 }}>Riku M.</span>
                <span style={{ fontSize: 12, color: 'var(--ink-3)' }}>があなたの投稿に</span>
                <span className="wf-pill accent" style={{ fontSize: 10 }}>♥</span>
                <span style={{ flex: 1 }} />
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>2m</span>
              </div>
              <div className="wf-box muted" style={{ padding: 10, fontSize: 12, color: 'var(--ink-2)' }}>
                "ワイヤーフレームは、決めない部分を会話するための道具。"
              </div>
            </div>
            {/* Reply */}
            <div className="wf-card" style={{ padding: 12 }}>
              <div className="wf-row" style={{ gap: 8, marginBottom: 8 }}>
                <div className="wf-av sm accent2" />
                <span className="wf-hand" style={{ fontSize: 14 }}>Aya T.</span>
                <span style={{ fontSize: 12, color: 'var(--ink-3)' }}>が返信しました</span>
                <span style={{ flex: 1 }} />
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>14m</span>
              </div>
              <p style={{ fontSize: 13, margin: '0 0 8px' }}>"同感。決めない自由を残したい。"</p>
              <div className="wf-row" style={{ gap: 6 }}>
                <button className="wf-btn sm">返信</button>
                <button className="wf-btn sm ghost">開く</button>
              </div>
            </div>
            {/* New follower */}
            <div className="wf-card" style={{ padding: 12 }}>
              <div className="wf-spread">
                <div className="wf-row" style={{ gap: 10 }}>
                  <div className="wf-av accent" />
                  <div className="wf-col">
                    <span className="wf-hand" style={{ fontSize: 16 }}>Ken S. <span style={{ fontSize: 12, color: 'var(--ink-3)', fontFamily: 'var(--font-body)' }}>があなたをフォロー</span></span>
                    <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>@ken_s · 共通 12人</span>
                  </div>
                </div>
                <button className="wf-btn primary sm">フォローバック</button>
              </div>
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

Object.assign(window, { NotifA, NotifB, NotifC });
