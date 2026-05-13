// Compose / Post creation — 3 variations

function ComposeA() {
  // V1: Modal-style centered composer
  return (
    <Frame label="A · centered modal">
      <div style={{ height: '100%', position: 'relative', background: 'var(--paper)' }}>
        <Sidebar active="home" />
        {/* Backdrop hint */}
        <div style={{
          position: 'absolute', inset: 0, background: 'rgba(0,0,0,0.18)',
          backdropFilter: 'blur(2px)',
          display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24,
        }}>
          <div className="wf-card raised" style={{ width: 540, padding: 18 }}>
            <div className="wf-spread" style={{ marginBottom: 12 }}>
              <div className="wf-row" style={{ gap: 8, alignItems: 'baseline' }}>
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)', letterSpacing: '0.14em' }}>[ COMPOSE ]</span>
                <span className="wf-hand" style={{ fontSize: 22 }}>新しい投稿</span>
              </div>
              <button className="wf-btn icon ghost"><span className="wf-mono">×</span></button>
            </div>

            {/* Visibility + warnings row */}
            <div className="wf-row" style={{ gap: 6, marginBottom: 10, flexWrap: 'wrap' }}>
              <button className="wf-btn sm">🌐 公開 ▾</button>
              <button className="wf-btn sm ghost">+ CW</button>
              <button className="wf-btn sm ghost" style={{ borderColor: 'var(--accent)', color: 'var(--accent)' }}>+ NSFW</button>
              <button className="wf-btn sm ghost">+ 予約</button>
              <span style={{ flex: 1 }} />
              <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>下書き · 自動保存</span>
            </div>

            {/* Scheduled (active example) */}
            <div className="wf-card" style={{ padding: 8, marginBottom: 6, background: 'var(--card-2)', borderStyle: 'dashed' }}>
              <div className="wf-row" style={{ gap: 8, alignItems: 'center' }}>
                <span className="wf-pill accent" style={{ fontSize: 9 }}>SCHED</span>
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>予約日時</span>
                <input className="wf-input" type="text" style={{ flex: '0 0 105px', height: 26, fontFamily: 'var(--font-mono)', fontSize: 11 }}
                       defaultValue="2026-05-12" />
                <input className="wf-input" type="text" style={{ flex: '0 0 70px', height: 26, fontFamily: 'var(--font-mono)', fontSize: 11 }}
                       defaultValue="09:00" />
                <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>JST</span>
                <span style={{ flex: 1 }} />
                <button className="wf-btn sm ghost">×</button>
              </div>
            </div>

            {/* CW input (active example) */}
            <div className="wf-card" style={{ padding: 8, marginBottom: 8, background: 'var(--card-2)' }}>
              <div className="wf-row" style={{ gap: 8 }}>
                <span className="wf-pill accent" style={{ fontSize: 9 }}>CW</span>
                <input className="wf-input" style={{ flex: 1, height: 26, border: 'none', background: 'transparent' }}
                       defaultValue="本のネタバレ含みます" />
                <button className="wf-btn sm ghost">×</button>
              </div>
            </div>

            <div className="wf-row" style={{ alignItems: 'flex-start', gap: 10 }}>
              <div className="wf-av accent" />
              <div className="wf-grow">
                <p className="wf-hand" style={{ fontSize: 22, color: 'var(--ink-3)', margin: '4px 0 12px' }}>
                  いま考えていること…
                </p>

                {/* Attachment preview area: file drop zone + thumbnails */}
                <div style={{ marginBottom: 12 }}>
                  <div className="wf-row" style={{ gap: 8, marginBottom: 6, flexWrap: 'wrap' }}>
                    {/* Image attachment */}
                    <div className="wf-thumb">
                      <div className="wf-media" style={{ width: 72, height: 72, borderRadius: 6 }}>IMG</div>
                      <span className="wf-thumb-meta">photo.jpg · 1.2MB</span>
                      <button className="wf-thumb-x">×</button>
                    </div>
                    {/* Video attachment */}
                    <div className="wf-thumb">
                      <div className="wf-media" style={{ width: 72, height: 72, borderRadius: 6, position: 'relative' }}>
                        <span style={{ position: 'absolute', fontSize: 18 }}>▶</span>
                      </div>
                      <span className="wf-thumb-meta">clip.mp4 · 0:08</span>
                      <button className="wf-thumb-x">×</button>
                    </div>
                    {/* Drop zone */}
                    <div className="wf-thumb dashed">
                      <div className="wf-thumb-drop">
                        <span className="wf-mono" style={{ fontSize: 10 }}>+</span>
                        <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>drop</span>
                      </div>
                    </div>
                  </div>
                  <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>
                    画像 / 動画 をドロップまたは [ 参照 ] · 最大 4 ファイル · 100MB
                  </span>
                </div>

                <div className="wf-spread" style={{ borderTop: '1.25px solid var(--line-soft)', paddingTop: 10 }}>
                  <div className="wf-row" style={{ gap: 4 }}>
                    <button className="wf-btn sm" title="画像・動画を添付">
                      <span style={{ fontSize: 12 }}>📎</span>
                      <span className="wf-mono" style={{ fontSize: 9 }}>FILE</span>
                    </button>
                    <button className="wf-btn icon ghost sm" title="投票"><span className="wf-mono" style={{ fontSize: 10 }}>📊</span></button>
                    <button className="wf-btn icon ghost sm" title="絵文字"><span className="wf-mono" style={{ fontSize: 10 }}>😊</span></button>
                    <button className="wf-btn icon ghost sm" title="場所"><span className="wf-mono" style={{ fontSize: 10 }}>📍</span></button>
                  </div>
                  <div className="wf-row" style={{ gap: 8 }}>
                    <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>248 / 500</span>
                    <button className="wf-btn accent">予約投稿 ⏰</button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Frame>
  );
}

function ComposeB() {
  // V2: Inline composer at top of feed
  return (
    <Frame label="B · inline at top">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="home" />
        <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
          <TopBar title="ホーム" dense />
          <div style={{ padding: 16, overflow: 'hidden' }}>
            <div className="wf-card raised" style={{ padding: 14, marginBottom: 16, borderLeft: '6px solid var(--accent)' }}>
              <div className="wf-row" style={{ alignItems: 'flex-start', gap: 10, marginBottom: 8 }}>
                <div className="wf-av accent" />
                <div className="wf-grow">
                  <div className="wf-row" style={{ marginBottom: 4 }}>
                    <span className="wf-hand" style={{ fontSize: 16 }}>あなた</span>
                    <span className="wf-pill" style={{ marginLeft: 8 }}>🌐 公開</span>
                    <span className="wf-pill" style={{ marginLeft: 4 }}>CW なし</span>
                  </div>
                  <p className="wf-hand" style={{ fontSize: 20, color: 'var(--ink-3)', margin: 0, minHeight: 60 }}>
                    あなたのきもちを書く…
                  </p>
                </div>
              </div>
              <div className="wf-spread" style={{ borderTop: '1.25px solid var(--line-soft)', paddingTop: 10 }}>
                <div className="wf-row" style={{ gap: 4 }}>
                  {['IMG','GIF','📊','😊','📍','#'].map(g => (
                    <button key={g} className="wf-btn icon ghost"><span className="wf-mono" style={{ fontSize: 10 }}>{g}</span></button>
                  ))}
                </div>
                <div className="wf-row" style={{ gap: 8 }}>
                  <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>500 - 12 = 488</span>
                  <button className="wf-btn ghost sm">下書き</button>
                  <button className="wf-btn accent">投稿</button>
                </div>
              </div>
            </div>
            <div style={{ opacity: 0.5 }}>
              <Post name="Riku M." handle="@riku" time="14m" />
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function ComposeC() {
  // V3: Distraction-free fullscreen "writing mode"
  return (
    <Frame label="C · fullscreen writing">
      <div style={{ height: '100%', display: 'flex', flexDirection: 'column', background: 'var(--paper)' }}>
        <div className="wf-spread" style={{ padding: '12px 20px', borderBottom: '1.25px solid var(--line-soft)' }}>
          <button className="wf-btn sm ghost">× 閉じる</button>
          <div className="wf-row" style={{ gap: 6 }}>
            <span className="wf-pill">🌐 公開</span>
            <span className="wf-pill accent2">下書き 自動保存</span>
          </div>
          <button className="wf-btn accent">投稿する →</button>
        </div>
        <div className="wf-grow" style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'center', padding: '60px 24px 24px', overflow: 'hidden' }}>
          <div style={{ width: 540, maxWidth: '100%' }}>
            <div className="wf-row" style={{ gap: 10, marginBottom: 24 }}>
              <div className="wf-av accent" />
              <div className="wf-col">
                <span className="wf-hand" style={{ fontSize: 18 }}>あなた</span>
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>@you · 公開</span>
              </div>
            </div>
            <p className="wf-hand" style={{ fontSize: 36, lineHeight: 1.3, margin: 0 }}>
              <span style={{ color: 'var(--ink)' }}>今日の発見は </span>
              <span className="wf-uline">余白の取り方</span>
              <span style={{ color: 'var(--ink-3)' }}>│</span>
            </p>
            <div className="wf-row" style={{ marginTop: 32, gap: 8, flexWrap: 'wrap' }}>
              <button className="wf-btn ghost sm">+ 画像</button>
              <button className="wf-btn ghost sm">+ 投票</button>
              <button className="wf-btn ghost sm">+ CW</button>
              <button className="wf-btn ghost sm">+ 場所</button>
              <button className="wf-btn ghost sm">+ 引用</button>
            </div>
          </div>
        </div>
        <div className="wf-spread" style={{ padding: '8px 20px', borderTop: '1.25px solid var(--line-soft)' }}>
          <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>~~ ⌘+Enter で投稿 · ⌘+S で下書き</span>
          <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>32 / 500</span>
        </div>
      </div>
    </Frame>
  );
}

Object.assign(window, { ComposeA, ComposeB, ComposeC });
