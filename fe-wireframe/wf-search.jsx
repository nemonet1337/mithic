// Search / Discover — 3 variations

function SearchA() {
  // V1: Classic search results with tabs
  return (
    <Frame label="A · search results">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="search" />
        <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
          <div style={{ padding: '14px 16px', borderBottom: '1.25px solid var(--line-soft)' }}>
            <div className="wf-input lg">
              <span className="glyph-ci" style={{ marginRight: 8 }} />
              <span style={{ color: 'var(--ink)' }}>余白 design</span>
              <span style={{ flex: 1 }} />
              <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>⌘K</span>
            </div>
          </div>
          <div className="wf-tabs" style={{ padding: '0 16px' }}>
            <span className="t on">すべて</span>
            <span className="t">投稿</span>
            <span className="t">ユーザー</span>
            <span className="t">タグ</span>
            <span className="t">メディア</span>
          </div>
          <div style={{ padding: 16, overflow: 'hidden' }}>
            <div className="wf-stack" style={{ gap: 10 }}>
              <Post name="Hana K." handle="@hana" time="2m" text="余白の取り方ひとつで意味が変わる。" accent="accent" />
              <Post name="Riku M." handle="@riku" time="3h" text="design における余白は、沈黙に近い。" />
              <div className="wf-card" style={{ padding: 10 }}>
                <div className="wf-label" style={{ marginBottom: 6 }}>関連ユーザー</div>
                <div className="wf-row" style={{ gap: 12 }}>
                  {[1,2,3].map(i => (
                    <div key={i} className="wf-row" style={{ gap: 8 }}>
                      <div className="wf-av sm" />
                      <span className="wf-mono" style={{ fontSize: 11 }}>@user_{i}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function SearchB() {
  // V2: Discover landing — categories grid
  return (
    <Frame label="B · discover grid">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="search" compact />
        <main className="wf-grow" style={{ minWidth: 0, padding: 20, overflow: 'hidden' }}>
          <h1 className="wf-hand" style={{ fontSize: 32, margin: 0 }}>
            <span className="wf-uline">発見する</span>
          </h1>
          <div className="wf-input lg" style={{ marginTop: 14 }}>
            <span className="glyph-ci" style={{ marginRight: 8 }} />
            検索ワードを入力…
          </div>
          <div style={{ marginTop: 16 }}>
            <span className="wf-label">急上昇</span>
            <div className="wf-row" style={{ flexWrap: 'wrap', gap: 6, marginTop: 6 }}>
              {['#wireframe','#design','#typography','#余白','#tokyo','#walk','#book','#ux','#flutter'].map((t,i) => (
                <span key={t} className={`wf-pill ${i === 0 ? 'accent' : i === 1 ? 'accent2' : ''}`}>{t}</span>
              ))}
            </div>
          </div>
          <div style={{ marginTop: 16 }}>
            <span className="wf-label">カテゴリ</span>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 10, marginTop: 8 }}>
              {[
                ['Art','作品とビジュアル','accent'],
                ['Tech','エンジニアリング',''],
                ['Books','本と読書','accent2'],
                ['Music','音楽',''],
                ['Food','食','accent'],
                ['Photo','写真',''],
              ].map(([n, sub, a]) => (
                <div key={n} className={`wf-card raised`} style={{ padding: 14, background: a === 'accent' ? 'var(--accent)' : a === 'accent2' ? 'var(--accent-2)' : 'var(--card)', color: a === 'accent' ? '#fff' : 'var(--ink)' }}>
                  <span className="wf-hand" style={{ fontSize: 22, lineHeight: 1 }}>{n}</span>
                  <p style={{ fontSize: 11, margin: '4px 0 0', opacity: 0.8 }}>{sub}</p>
                </div>
              ))}
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function SearchC() {
  // V3: Command palette style — keyboard-driven
  return (
    <Frame label="C · command palette">
      <div style={{ height: '100%', position: 'relative' }}>
        <div style={{ display: 'flex', height: '100%', opacity: 0.35 }}>
          <Sidebar active="search" />
          <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
            <TopBar title="ホーム" dense />
            <div style={{ padding: 16 }}><Post name="Riku M." handle="@riku" time="14m" /></div>
          </main>
        </div>
        <div style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'flex-start', justifyContent: 'center', paddingTop: 60, background: 'rgba(0,0,0,0.18)' }}>
          <div className="wf-card raised" style={{ width: 480, padding: 0, overflow: 'hidden' }}>
            <div className="wf-row" style={{ padding: 14, borderBottom: '1.25px solid var(--line-soft)', gap: 10 }}>
              <span className="glyph-ci" />
              <span className="wf-grow" style={{ fontSize: 14 }}>余白</span>
              <span className="wf-pill">⌘K</span>
            </div>
            <div className="wf-stack" style={{ padding: 6, gap: 0 }}>
              <div className="wf-label" style={{ padding: '6px 10px' }}>ユーザー</div>
              {[['@hana','Hana K.', true],['@haku','Haku','']].map(([h,n,on]) => (
                <div key={h} className="wf-row" style={{ padding: '6px 10px', borderRadius: 6, background: on ? 'var(--card-2)' : 'transparent' }}>
                  <div className="wf-av sm accent" />
                  <span className="wf-hand" style={{ fontSize: 14 }}>{n}</span>
                  <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>{h}</span>
                  <span style={{ flex: 1 }} />
                  {on && <span className="wf-pill" style={{ fontSize: 9 }}>↵</span>}
                </div>
              ))}
              <div className="wf-label" style={{ padding: '8px 10px 6px' }}>タグ</div>
              {['#余白','#余白の取り方'].map(t => (
                <div key={t} className="wf-row" style={{ padding: '6px 10px' }}>
                  <span className="wf-tag">{t}</span>
                  <span style={{ flex: 1 }} />
                  <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>{Math.floor(Math.random()*900)} 投稿</span>
                </div>
              ))}
              <div className="wf-label" style={{ padding: '8px 10px 6px' }}>アクション</div>
              <div className="wf-row" style={{ padding: '6px 10px' }}>
                <span className="glyph-plus" />
                <span style={{ fontSize: 13 }}>「余白」で新しい投稿を作成</span>
              </div>
            </div>
            <div className="wf-spread" style={{ padding: '8px 12px', borderTop: '1.25px solid var(--line-soft)', background: 'var(--card-2)' }}>
              <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>↑↓ 移動  ↵ 開く</span>
              <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>esc</span>
            </div>
          </div>
        </div>
      </div>
    </Frame>
  );
}

Object.assign(window, { SearchA, SearchB, SearchC });
