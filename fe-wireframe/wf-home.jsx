// Home / Feed — 3 variations

function HomeA() {
  // V1: Classic — Misskey-ish 3-column desktop
  return (
    <Frame label="A · classic 3-col">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="home" />
        <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
          <TopBar title="ホーム" right={
            <>
              <div className="wf-tabs" style={{ borderBottom: 'none' }}>
                <span className="t on">フォロー中</span>
                <span className="t">ローカル</span>
                <span className="t">グローバル</span>
              </div>
            </>
          } />
          <div className="wf-stack" style={{ padding: 16, gap: 12, overflow: 'hidden' }}>
            <Post name="Hana K." handle="@hana" time="2m" accent="accent" />
            <Post name="Riku M." handle="@riku" time="14m" text="今読んでる本のスクショ。装丁の余白の取り方が好み。" media="cover crop" />
            <Post name="Aya T."  handle="@aya"  time="1h" />
            <Post name="Ken S."  handle="@ken_s" time="3h" accent="accent2" />
          </div>
        </main>
        <RightRail />
      </div>
    </Frame>
  );
}

function HomeB() {
  // V2: Compact rail + magazine-style cards with accent stripe
  return (
    <Frame label="B · magazine">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="home" compact />
        <main className="wf-grow wf-col" style={{ minWidth: 0, background: 'var(--paper)' }}>
          <div style={{ padding: '20px 24px 12px' }}>
            <span className="wf-label">FEED · 2026.05.10</span>
            <div className="wf-spread" style={{ marginTop: 4 }}>
              <h1 className="wf-hand" style={{ fontSize: 36, margin: 0, lineHeight: 1 }}>
                <span className="wf-uline">今日のタイムライン</span>
              </h1>
              <div className="wf-row" style={{ gap: 6 }}>
                <span className="wf-pill accent2">live · 24</span>
                <span className="wf-pill">@all</span>
              </div>
            </div>
          </div>
          <div style={{ padding: '0 24px 16px', display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, overflow: 'hidden' }}>
            <div className="wf-card raised" style={{ padding: 14, gridColumn: '1 / -1', borderLeft: '6px solid var(--accent)' }}>
              <div className="wf-row" style={{ marginBottom: 6 }}>
                <span className="wf-pill accent">PINNED</span>
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>@hana · 12分前</span>
              </div>
              <p className="wf-hand" style={{ fontSize: 22, margin: 0, lineHeight: 1.25 }}>
                ワイヤーの粒度って、未確定であることを「見せる」ためにある。決めすぎない。
              </p>
            </div>
            <Post name="Riku M." handle="@riku" time="14m" />
            <Post name="Aya T."  handle="@aya"  time="22m" accent="accent2" />
            <Post name="Ken S."  handle="@ken_s" time="1h" media="screenshot · 2:1" />
          </div>
        </main>
      </div>
    </Frame>
  );
}

function HomeC() {
  // V3: Experimental — chronological vertical timeline with time markers on the left
  return (
    <Frame label="C · timeline rail">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="home" />
        <main className="wf-grow" style={{ minWidth: 0, padding: '20px 28px', overflow: 'hidden' }}>
          <div className="wf-spread" style={{ marginBottom: 14 }}>
            <span className="wf-hand" style={{ fontSize: 26 }}>タイムライン</span>
            <div className="wf-row" style={{ gap: 6 }}>
              <button className="wf-btn sm">時系列</button>
              <button className="wf-btn sm ghost">注目順</button>
            </div>
          </div>
          <div className="wf-seg-tl" style={{ marginBottom: 14 }}>
            <button className="seg on">
              <span className="wf-mono" style={{ fontSize: 9, letterSpacing: '0.14em' }}>01</span>
              <span>ホーム</span>
            </button>
            <button className="seg">
              <span className="wf-mono" style={{ fontSize: 9, letterSpacing: '0.14em' }}>02</span>
              <span>ローカル</span>
            </button>
            <button className="seg">
              <span className="wf-mono" style={{ fontSize: 9, letterSpacing: '0.14em' }}>03</span>
              <span>グローバル</span>
            </button>
          </div>
          <div style={{ position: 'relative', paddingLeft: 80 }}>
            <div style={{ position: 'absolute', left: 60, top: 4, bottom: 4, width: 1.5, background: 'var(--line-soft)' }} />
            {[
              ['09:42', 'Hana K.', '@hana', 'いま'],
              ['09:18', 'Riku M.', '@riku', '24m'],
              ['08:55', 'Aya T.',  '@aya',  '47m'],
              ['08:02', 'Ken S.',  '@ken',  '1h'],
            ].map((p, i) => (
              <div key={i} style={{ position: 'relative', marginBottom: 14 }}>
                <span className="wf-mono" style={{ position: 'absolute', left: -80, top: 12, fontSize: 11, color: 'var(--ink-3)' }}>{p[0]}</span>
                <span style={{ position: 'absolute', left: -22, top: 14, width: 10, height: 10, borderRadius: '50%', background: 'var(--paper)', border: '1.5px solid var(--ink)' }} />
                <Post name={p[1]} handle={p[2]} time={p[3]} accent={i === 0 ? 'accent' : ''} />
              </div>
            ))}
          </div>
        </main>
      </div>
    </Frame>
  );
}

Object.assign(window, { HomeA, HomeB, HomeC });
