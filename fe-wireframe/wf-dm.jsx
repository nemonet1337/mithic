// DM — 3 variations

function DMA() {
  // V1: Classic two-pane (list + conversation)
  return (
    <Frame label="A · two pane">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="dm" compact />
        <aside style={{ width: 260, borderRight: '1.25px solid var(--line-soft)', display: 'flex', flexDirection: 'column' }}>
          <div style={{ padding: 12, borderBottom: '1.25px solid var(--line-soft)' }}>
            <div className="wf-spread" style={{ marginBottom: 8 }}>
              <span className="wf-hand" style={{ fontSize: 22 }}>DM</span>
              <button className="wf-btn icon sm"><span className="glyph-plus" /></button>
            </div>
            <div className="wf-input dashed">検索</div>
          </div>
          {[
            ['Hana K.','@hana','余白について話そう','2m', true],
            ['Riku M.','@riku','OK 送りました','14m', false],
            ['Aya T.','@aya','📚','1h', false],
            ['Group · design','3 人','Ken: たしかに','3h', false],
          ].map((c, i) => (
            <div key={i} className="wf-row" style={{ padding: 10, gap: 8, borderBottom: '1.25px solid var(--line-soft)', background: i === 0 ? 'var(--card-2)' : 'transparent' }}>
              <div className={`wf-av sm ${i === 0 ? 'accent' : i === 2 ? 'accent2' : ''}`} />
              <div className="wf-col wf-grow" style={{ minWidth: 0 }}>
                <div className="wf-spread">
                  <span className="wf-hand" style={{ fontSize: 14, lineHeight: 1 }}>{c[0]}</span>
                  <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>{c[3]}</span>
                </div>
                <span style={{ fontSize: 11, color: 'var(--ink-3)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{c[2]}</span>
              </div>
              {c[4] && <span style={{ width: 7, height: 7, borderRadius: '50%', background: 'var(--accent)' }} />}
            </div>
          ))}
        </aside>
        <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
          <div className="wf-spread" style={{ padding: 12, borderBottom: '1.25px solid var(--line-soft)' }}>
            <div className="wf-row" style={{ gap: 8 }}>
              <div className="wf-av sm accent" />
              <div className="wf-col">
                <span className="wf-hand" style={{ fontSize: 15, lineHeight: 1 }}>Hana K.</span>
                <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>@hana · オンライン</span>
              </div>
            </div>
            <button className="wf-btn icon ghost"><span className="wf-mono">···</span></button>
          </div>
          <div className="wf-grow wf-stack" style={{ padding: 14, gap: 8, overflow: 'hidden' }}>
            <div className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)', textAlign: 'center' }}>— 今日 —</div>
            <div className="wf-row" style={{ alignSelf: 'flex-start', maxWidth: '70%' }}>
              <div className="wf-box" style={{ padding: '8px 12px', borderRadius: '12px 12px 12px 4px' }}>
                ワイヤーの粒度ってどう決めてる？
              </div>
            </div>
            <div className="wf-row" style={{ alignSelf: 'flex-end', maxWidth: '70%' }}>
              <div className="wf-box fill-accent" style={{ padding: '8px 12px', borderRadius: '12px 12px 4px 12px' }}>
                決めすぎないように。会話が生まれる粒度。
              </div>
            </div>
            <div className="wf-row" style={{ alignSelf: 'flex-start', maxWidth: '70%' }}>
              <div className="wf-box" style={{ padding: '8px 12px', borderRadius: '12px 12px 12px 4px' }}>
                なるほど。じゃあ余白について話そう
              </div>
            </div>
          </div>
          <div style={{ padding: 12, borderTop: '1.25px solid var(--line-soft)' }}>
            <div className="wf-input lg">
              <span style={{ flex: 1, color: 'var(--ink-3)' }}>メッセージを入力…</span>
              <span className="wf-row" style={{ gap: 4 }}>
                <span className="wf-mono" style={{ fontSize: 10 }}>📎</span>
                <span className="wf-mono" style={{ fontSize: 10 }}>😊</span>
              </span>
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function DMB() {
  // V2: Inbox-first list
  return (
    <Frame label="B · inbox first">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="dm" />
        <main className="wf-grow" style={{ minWidth: 0, padding: 20, overflow: 'hidden' }}>
          <div className="wf-spread" style={{ marginBottom: 14 }}>
            <h1 className="wf-hand" style={{ fontSize: 30, margin: 0 }}>受信箱</h1>
            <div className="wf-row" style={{ gap: 6 }}>
              <span className="wf-pill accent">未読 2</span>
              <button className="wf-btn sm">新規</button>
            </div>
          </div>
          <div className="wf-tabs" style={{ marginBottom: 14 }}>
            <span className="t on">すべて</span>
            <span className="t">未読</span>
            <span className="t">グループ</span>
            <span className="t">リクエスト · 3</span>
          </div>
          <div className="wf-stack" style={{ gap: 8 }}>
            {[
              ['Hana K.','@hana','余白について話そう','2m', true,'accent'],
              ['Riku M.','@riku','OK 送りました — 添付ファイルがあります','14m', true,''],
              ['design group','3人','Ken: たしかに、それ便利',' 3h', false,'accent2'],
              ['Aya T.','@aya','📚 おすすめの本','1d', false,''],
            ].map((c, i) => (
              <div key={i} className="wf-card" style={{ padding: 12, borderLeft: c[4] ? '4px solid var(--accent)' : '1.25px solid var(--line)' }}>
                <div className="wf-row" style={{ gap: 10 }}>
                  <div className={`wf-av ${c[5]}`} />
                  <div className="wf-col wf-grow" style={{ minWidth: 0 }}>
                    <div className="wf-spread">
                      <div className="wf-row">
                        <span className="wf-hand" style={{ fontSize: 16 }}>{c[0]}</span>
                        <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>{c[1]}</span>
                      </div>
                      <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>{c[3]}</span>
                    </div>
                    <p style={{ margin: '4px 0 0', fontSize: 13, color: c[4] ? 'var(--ink)' : 'var(--ink-3)' }}>{c[2]}</p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </main>
      </div>
    </Frame>
  );
}

function DMC() {
  // V3: Floating chat windows (Facebook-ish)
  return (
    <Frame label="C · floating windows">
      <div style={{ height: '100%', position: 'relative' }}>
        <div style={{ display: 'flex', height: '100%' }}>
          <Sidebar active="home" />
          <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
            <TopBar title="ホーム" dense />
            <div style={{ padding: 16, opacity: 0.5 }}>
              <Post name="Riku M." handle="@riku" time="14m" />
            </div>
          </main>
        </div>
        {/* Bottom right floating chats */}
        <div style={{ position: 'absolute', right: 16, bottom: 0, display: 'flex', gap: 8, alignItems: 'flex-end' }}>
          {/* Minimized */}
          <div className="wf-card raised" style={{ width: 220, height: 36, padding: '0 12px', display: 'flex', alignItems: 'center', gap: 8, borderRadius: '10px 10px 0 0' }}>
            <div className="wf-av sm accent2" />
            <span className="wf-hand wf-grow" style={{ fontSize: 13 }}>Aya T.</span>
            <span className="wf-mono" style={{ fontSize: 10 }}>×</span>
          </div>
          {/* Open */}
          <div className="wf-card raised" style={{ width: 280, height: 380, display: 'flex', flexDirection: 'column', borderRadius: '10px 10px 0 0' }}>
            <div className="wf-spread" style={{ padding: '8px 12px', borderBottom: '1.25px solid var(--line-soft)' }}>
              <div className="wf-row" style={{ gap: 6 }}>
                <div className="wf-av sm accent" />
                <span className="wf-hand" style={{ fontSize: 14 }}>Hana K.</span>
              </div>
              <div className="wf-row" style={{ gap: 6 }}>
                <span className="wf-mono" style={{ fontSize: 11 }}>—</span>
                <span className="wf-mono" style={{ fontSize: 11 }}>×</span>
              </div>
            </div>
            <div className="wf-grow wf-stack" style={{ padding: 10, gap: 6, overflow: 'hidden' }}>
              <div className="wf-box" style={{ padding: '6px 10px', alignSelf: 'flex-start', fontSize: 12, borderRadius: '10px 10px 10px 2px' }}>こんにちは</div>
              <div className="wf-box fill-accent" style={{ padding: '6px 10px', alignSelf: 'flex-end', fontSize: 12, borderRadius: '10px 10px 2px 10px' }}>余白について話そう</div>
              <div className="wf-box" style={{ padding: '6px 10px', alignSelf: 'flex-start', fontSize: 12, borderRadius: '10px 10px 10px 2px' }}>いいね、どこから?</div>
            </div>
            <div style={{ padding: 8, borderTop: '1.25px solid var(--line-soft)' }}>
              <div className="wf-input">メッセージ…</div>
            </div>
          </div>
        </div>
      </div>
    </Frame>
  );
}

Object.assign(window, { DMA, DMB, DMC });
