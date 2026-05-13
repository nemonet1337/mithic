// Settings — 3 variations

function SettingsA() {
  // V1: Classic sidebar within sidebar
  const groups = [
    ['アカウント', ['プロフィール', 'メール', 'パスワード', '連携アカウント']],
    ['プライバシー', ['公開範囲', 'ブロック', 'ミュート']],
    ['通知', ['プッシュ', 'メール', 'メンション']],
    ['表示', ['テーマ', '言語', 'タイムゾーン']],
  ];
  return (
    <Frame label="A · two-pane settings">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="settings" compact />
        <aside style={{ width: 220, borderRight: '1.25px solid var(--line-soft)', padding: 14 }}>
          <span className="wf-hand" style={{ fontSize: 22 }}>設定</span>
          <div className="wf-stack" style={{ marginTop: 12, gap: 14 }}>
            {groups.map(([g, items]) => (
              <div key={g}>
                <span className="wf-label">{g}</span>
                <div className="wf-stack" style={{ marginTop: 4, gap: 0 }}>
                  {items.map((it, i) => (
                    <div key={it} className="wf-spread" style={{ padding: '6px 8px', fontSize: 13, borderRadius: 6, background: g === 'アカウント' && i === 0 ? 'var(--ink)' : 'transparent', color: g === 'アカウント' && i === 0 ? 'var(--paper)' : 'var(--ink-2)' }}>
                      <span>{it}</span>
                      <span style={{ fontSize: 10, opacity: 0.5 }}>›</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </aside>
        <main className="wf-grow" style={{ minWidth: 0, padding: 24, overflow: 'hidden' }}>
          <span className="wf-label">アカウント / プロフィール</span>
          <h2 className="wf-hand" style={{ fontSize: 28, margin: '4px 0 16px' }}>プロフィール設定</h2>
          <div className="wf-stack" style={{ gap: 14, maxWidth: 480 }}>
            <div className="wf-row">
              <div className="wf-av xl accent" />
              <div className="wf-col" style={{ gap: 4 }}>
                <button className="wf-btn sm">画像を変更</button>
                <button className="wf-btn sm ghost">削除</button>
              </div>
            </div>
            <div>
              <span className="wf-label">表示名</span>
              <div className="wf-input lg" style={{ marginTop: 4 }}>Hana K.</div>
            </div>
            <div>
              <span className="wf-label">ハンドル</span>
              <div className="wf-input lg" style={{ marginTop: 4, color: 'var(--ink-3)' }}>@hana</div>
            </div>
            <div>
              <span className="wf-label">自己紹介</span>
              <div className="wf-input lg" style={{ marginTop: 4, height: 70, alignItems: 'flex-start', paddingTop: 10 }}>UI設計と植物。決めない自由を残す。</div>
            </div>
            <div className="wf-row" style={{ justifyContent: 'flex-end', gap: 6 }}>
              <button className="wf-btn ghost">キャンセル</button>
              <button className="wf-btn primary">保存</button>
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function SettingsB() {
  // V2: Card-based grouped settings
  return (
    <Frame label="B · cards">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="settings" />
        <main className="wf-grow" style={{ minWidth: 0, padding: 20, overflow: 'hidden' }}>
          <h1 className="wf-hand" style={{ fontSize: 30, margin: '0 0 4px' }}>設定</h1>
          <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>@hana · v2.4.1</span>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginTop: 16 }}>
            {[
              ['アカウント','メール、パスワード、認証','accent'],
              ['プライバシー','公開範囲、ブロック、ミュート',''],
              ['通知','プッシュ、メール、メンション','accent2'],
              ['表示','テーマ、言語、タイムゾーン',''],
              ['データ','エクスポート、削除',''],
              ['連携','他サービスとの連携','accent'],
            ].map(([n, sub, ac]) => (
              <div key={n} className="wf-card" style={{ padding: 14 }}>
                <div className="wf-spread" style={{ marginBottom: 6 }}>
                  <span className={`wf-pill ${ac}`}>•</span>
                  <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>›</span>
                </div>
                <span className="wf-hand" style={{ fontSize: 20, lineHeight: 1 }}>{n}</span>
                <p style={{ fontSize: 11, color: 'var(--ink-3)', margin: '4px 0 0' }}>{sub}</p>
              </div>
            ))}
          </div>
          <div className="wf-card dashed" style={{ padding: 14, marginTop: 14, textAlign: 'center' }}>
            <span className="wf-label">ZONE · DANGER</span>
            <div className="wf-row" style={{ justifyContent: 'center', gap: 8, marginTop: 8 }}>
              <button className="wf-btn sm ghost">アカウントを一時停止</button>
              <button className="wf-btn sm" style={{ borderColor: 'var(--accent)', color: 'var(--accent)' }}>削除する</button>
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function SettingsC() {
  // V3: All-in-one scrolling form with section anchors
  return (
    <Frame label="C · single page form">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="settings" compact />
        <aside style={{ width: 160, padding: 16, borderRight: '1.25px solid var(--line-soft)' }}>
          <span className="wf-label">JUMP TO</span>
          <div className="wf-stack" style={{ marginTop: 8, gap: 4, fontSize: 12 }}>
            {['アカウント','プライバシー','通知','表示','連携','危険な操作'].map((s, i) => (
              <span key={s} style={{ padding: '4px 8px', borderRadius: 4, background: i === 0 ? 'var(--accent-2)' : 'transparent', color: 'var(--ink)', borderLeft: i === 0 ? '2px solid var(--ink)' : '2px solid transparent' }}>{s}</span>
            ))}
          </div>
        </aside>
        <main className="wf-grow" style={{ minWidth: 0, padding: '24px 28px', overflow: 'hidden' }}>
          <h1 className="wf-hand" style={{ fontSize: 32, margin: '0 0 16px' }}>すべての設定</h1>
          {/* Section: Account */}
          <section style={{ marginBottom: 18 }}>
            <span className="wf-label">アカウント</span>
            <div className="wf-stack" style={{ gap: 10, marginTop: 8 }}>
              <div className="wf-spread wf-card" style={{ padding: 12 }}>
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>メールアドレス</div>
                  <div className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>hana@example.com</div>
                </div>
                <button className="wf-btn sm">変更</button>
              </div>
              <div className="wf-spread wf-card" style={{ padding: 12 }}>
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600 }}>2段階認証</div>
                  <div style={{ fontSize: 11, color: 'var(--ink-3)' }}>SMSによる認証 · 有効</div>
                </div>
                <div className="wf-pill accent2">ON</div>
              </div>
            </div>
          </section>
          {/* Section: Display */}
          <section style={{ marginBottom: 18 }}>
            <span className="wf-label">表示</span>
            <div className="wf-stack" style={{ gap: 10, marginTop: 8 }}>
              <div className="wf-spread wf-card" style={{ padding: 12 }}>
                <span style={{ fontSize: 13 }}>テーマ</span>
                <div className="wf-row" style={{ gap: 4 }}>
                  <button className="wf-btn sm primary">ライト</button>
                  <button className="wf-btn sm ghost">ダーク</button>
                  <button className="wf-btn sm ghost">自動</button>
                </div>
              </div>
              <div className="wf-spread wf-card" style={{ padding: 12 }}>
                <span style={{ fontSize: 13 }}>言語</span>
                <div className="wf-pill">日本語 ▾</div>
              </div>
              <div className="wf-spread wf-card" style={{ padding: 12 }}>
                <span style={{ fontSize: 13 }}>密度</span>
                <div className="wf-row" style={{ gap: 4 }}>
                  <button className="wf-btn sm ghost">コンパクト</button>
                  <button className="wf-btn sm primary">標準</button>
                  <button className="wf-btn sm ghost">ゆったり</button>
                </div>
              </div>
            </div>
          </section>
        </main>
      </div>
    </Frame>
  );
}

Object.assign(window, { SettingsA, SettingsB, SettingsC });
