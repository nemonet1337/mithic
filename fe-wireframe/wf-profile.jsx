// Profile — 3 variations

function ProfileA() {
  // V1: Classic banner + tabs
  return (
    <Frame label="A · banner + tabs">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="profile" />
        <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
          <div className="wf-media" style={{ height: 120, borderRadius: 0, borderLeft: 0, borderRight: 0, borderTop: 0 }}>cover · 4:1</div>
          <div style={{ padding: '0 20px' }}>
            <div className="wf-row" style={{ alignItems: 'flex-end', marginTop: -36, marginBottom: 12 }}>
              <div className="wf-av xl accent" style={{ background: 'var(--accent)', boxShadow: '0 0 0 4px var(--paper)' }} />
              <div style={{ flex: 1 }} />
              <div className="wf-row" style={{ gap: 6 }}>
                <button className="wf-btn sm ghost">···</button>
                <button className="wf-btn sm">編集</button>
                <button className="wf-btn sm primary">フォロー中</button>
              </div>
            </div>
            <div>
              <h2 className="wf-hand" style={{ fontSize: 28, margin: 0 }}>Hana K.</h2>
              <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>@hana@mithic.social</span>
              <p style={{ fontSize: 13, margin: '8px 0' }}>UI設計と植物。決めない自由を残す。</p>
              <div className="wf-row" style={{ gap: 16, fontSize: 12 }}>
                <span><b>248</b> <span className="wf-label">投稿</span></span>
                <span><b>1.2k</b> <span className="wf-label">フォロワー</span></span>
                <span><b>320</b> <span className="wf-label">フォロー</span></span>
              </div>
            </div>
            <div className="wf-tabs" style={{ marginTop: 12 }}>
              <span className="t on">投稿</span>
              <span className="t">返信</span>
              <span className="t">メディア</span>
              <span className="t">いいね</span>
            </div>
            <div className="wf-stack" style={{ paddingTop: 12, gap: 10 }}>
              <Post name="Hana K." handle="@hana" time="2m" accent="accent" />
              <Post name="Hana K." handle="@hana" time="2h" />
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function ProfileB() {
  // V2: Card-based — info card + grid of posts (Misskey-ish)
  return (
    <Frame label="B · card + grid">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="profile" compact />
        <main className="wf-grow" style={{ minWidth: 0, padding: 16, overflow: 'hidden' }}>
          <div style={{ display: 'grid', gridTemplateColumns: '280px 1fr', gap: 16 }}>
            <div className="wf-stack" style={{ gap: 12 }}>
              <div className="wf-card raised" style={{ padding: 16, textAlign: 'center' }}>
                <div className="wf-av xl accent" style={{ margin: '0 auto 10px' }} />
                <h3 className="wf-hand" style={{ fontSize: 22, margin: 0 }}>Hana K.</h3>
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>@hana</span>
                <p style={{ fontSize: 12, margin: '10px 0' }}>UI設計と植物。決めない自由を。</p>
                <div className="wf-row" style={{ gap: 6, justifyContent: 'center', flexWrap: 'wrap' }}>
                  <span className="wf-pill accent2">designer</span>
                  <span className="wf-pill">tokyo</span>
                </div>
                <div className="wf-stack" style={{ marginTop: 12, gap: 6 }}>
                  <button className="wf-btn full primary">フォロー中</button>
                  <button className="wf-btn full ghost sm">メッセージ</button>
                </div>
              </div>
              <div className="wf-card" style={{ padding: 12 }}>
                <div className="wf-label" style={{ marginBottom: 6 }}>STATS</div>
                <div className="wf-stack" style={{ gap: 4 }}>
                  <div className="wf-spread"><span style={{ fontSize: 12 }}>投稿</span><b>248</b></div>
                  <div className="wf-spread"><span style={{ fontSize: 12 }}>フォロワー</span><b>1.2k</b></div>
                  <div className="wf-spread"><span style={{ fontSize: 12 }}>フォロー</span><b>320</b></div>
                  <div className="wf-spread"><span style={{ fontSize: 12 }}>参加</span><span className="wf-mono" style={{ fontSize: 10 }}>2024.03</span></div>
                </div>
              </div>
            </div>
            <div>
              <div className="wf-tabs" style={{ marginBottom: 12 }}>
                <span className="t on">投稿</span>
                <span className="t">メディア</span>
                <span className="t">いいね</span>
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
                {[1,2,3,4].map(i => (
                  <div key={i} className="wf-card" style={{ padding: 10 }}>
                    <div className="wf-row" style={{ marginBottom: 4 }}>
                      <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>{i}h</span>
                    </div>
                    <Lines count={2} />
                    {i === 2 && <div className="wf-media" style={{ height: 60, marginTop: 6 }}>img</div>}
                  </div>
                ))}
              </div>
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function ProfileC() {
  // V3: Editorial — large name, asymmetric, tag clouds
  return (
    <Frame label="C · editorial">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="profile" />
        <main className="wf-grow" style={{ minWidth: 0, padding: '24px 32px', overflow: 'hidden' }}>
          <span className="wf-label">PROFILE · 002</span>
          <div className="wf-spread" style={{ alignItems: 'flex-start', marginTop: 4 }}>
            <h1 className="wf-hand" style={{ fontSize: 64, margin: 0, lineHeight: 0.95, maxWidth: '70%' }}>
              Hana<br/>
              <span className="wf-uline">Kitamura</span>
            </h1>
            <div className="wf-av xl accent" />
          </div>
          <div className="wf-row" style={{ gap: 8, marginTop: 12, flexWrap: 'wrap' }}>
            <span className="wf-pill ink">@hana</span>
            <span className="wf-pill">📍 tokyo</span>
            <span className="wf-pill">🌱 since 2024</span>
            <span className="wf-pill accent2">designer / writer</span>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 16, marginTop: 20, paddingTop: 16, borderTop: '1.25px solid var(--line-soft)' }}>
            <div>
              <span className="wf-label">BIO</span>
              <p style={{ fontSize: 14, margin: '6px 0 0', lineHeight: 1.5 }}>
                UI設計と植物。決めない自由を残す。最近は余白について考えている。
              </p>
            </div>
            <div>
              <span className="wf-label">最近の話題</span>
              <div className="wf-row" style={{ flexWrap: 'wrap', gap: 4, marginTop: 6 }}>
                {['#wireframe','#余白','#typography','#book','#design','#tokyo','#walk'].map(t => (
                  <span key={t} className="wf-tag" style={{ fontSize: 12 }}>{t}</span>
                ))}
              </div>
            </div>
            <div>
              <span className="wf-label">指標</span>
              <div className="wf-stack" style={{ gap: 4, marginTop: 6, fontSize: 13 }}>
                <div className="wf-spread"><span>投稿</span><b className="wf-mono">248</b></div>
                <div className="wf-spread"><span>フォロワー</span><b className="wf-mono">1,247</b></div>
                <div className="wf-spread"><span>フォロー</span><b className="wf-mono">320</b></div>
              </div>
            </div>
          </div>
          <div style={{ marginTop: 20 }}>
            <span className="wf-label">最新の投稿</span>
            <div className="wf-stack" style={{ marginTop: 8, gap: 10 }}>
              <Post name="Hana K." handle="@hana" time="2m" accent="accent" />
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

Object.assign(window, { ProfileA, ProfileB, ProfileC });
