// Post detail — 3 variations

function DetailA() {
  // V1: Twitter-classic single column with replies below
  return (
    <Frame label="A · classic thread">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="home" />
        <main className="wf-grow wf-col" style={{ minWidth: 0 }}>
          <TopBar title="投稿" right={<button className="wf-btn sm ghost">← 戻る</button>} dense />
          <div style={{ padding: 16, overflow: 'hidden' }}>
            <div className="wf-row" style={{ marginBottom: 10 }}>
              <div className="wf-av lg accent" />
              <div className="wf-col wf-grow">
                <span className="wf-hand" style={{ fontSize: 20 }}>Hana K.</span>
                <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>@hana</span>
              </div>
              <button className="wf-btn">フォロー</button>
            </div>
            <p style={{ fontSize: 18, lineHeight: 1.5, margin: '0 0 10px' }}>
              ワイヤーフレームを作るときは、最終形を再現するためじゃなくて、<span className="wf-tag">#決めていない部分</span>を会話するために描いている。
            </p>
            <div className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)', marginBottom: 10 }}>
              09:42 · 2026年5月10日 · web から
            </div>
            <div className="wf-row" style={{ gap: 16, padding: '8px 0', borderTop: '1.25px solid var(--line-soft)', borderBottom: '1.25px solid var(--line-soft)' }}>
              <span><b>47</b> <span className="wf-label">リノート</span></span>
              <span><b>184</b> <span className="wf-label">いいね</span></span>
              <span><b>12</b> <span className="wf-label">引用</span></span>
            </div>
            <div className="wf-row" style={{ gap: 8, padding: '10px 0' }}>
              <button className="wf-btn icon"><span className="glyph-ci" /></button>
              <button className="wf-btn icon"><span className="glyph-sq" /></button>
              <button className="wf-btn icon"><span className="glyph-di" /></button>
              <button className="wf-btn icon"><span className="glyph-tr" style={{ borderBottom: '12px solid currentColor' }} /></button>
            </div>
            <div className="wf-stack" style={{ gap: 10 }}>
              <Post name="Riku M." handle="@riku" time="20m" text="同感。決めない自由を残したい。" />
              <Post name="Aya T."  handle="@aya"  time="35m" />
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function DetailB() {
  // V2: Conversation-focused — vertical thread connector lines
  return (
    <Frame label="B · threaded conversation">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="home" />
        <main className="wf-grow" style={{ minWidth: 0, padding: 16, overflow: 'hidden' }}>
          <div className="wf-row" style={{ marginBottom: 10 }}>
            <button className="wf-btn sm ghost">← 戻る</button>
            <span className="wf-label">スレッド · 4件</span>
          </div>
          <div style={{ position: 'relative' }}>
            {/* Original */}
            <div className="wf-row" style={{ alignItems: 'flex-start', gap: 12, marginBottom: 8 }}>
              <div className="wf-col" style={{ alignItems: 'center', gap: 0 }}>
                <div className="wf-av accent" />
                <div style={{ width: 1.5, flex: 1, background: 'var(--line-soft)', minHeight: 40, marginTop: 4 }} />
              </div>
              <div className="wf-grow" style={{ paddingBottom: 16 }}>
                <div className="wf-spread">
                  <div>
                    <span className="wf-hand" style={{ fontSize: 18 }}>Hana K.</span>
                    <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)', marginLeft: 6 }}>@hana</span>
                  </div>
                </div>
                <p style={{ fontSize: 15, margin: '4px 0 8px' }}>
                  ワイヤーフレームは、決めない部分を会話するための道具。
                </p>
                <div className="wf-row" style={{ gap: 14, color: 'var(--ink-3)', fontSize: 11 }}>
                  <span>♻ 47</span><span>♥ 184</span><span>↗ 12</span>
                </div>
              </div>
            </div>
            {/* Replies */}
            {['Riku M.', 'Aya T.', 'Ken S.'].map((n, i) => (
              <div key={n} className="wf-row" style={{ alignItems: 'flex-start', gap: 12, marginBottom: 8 }}>
                <div className="wf-col" style={{ alignItems: 'center', gap: 0 }}>
                  <div className="wf-av sm" />
                  {i < 2 && <div style={{ width: 1.5, flex: 1, background: 'var(--line-soft)', minHeight: 30, marginTop: 4, marginLeft: 0 }} />}
                </div>
                <div className="wf-card wf-grow" style={{ padding: 10 }}>
                  <div className="wf-row" style={{ gap: 6, marginBottom: 4 }}>
                    <span className="wf-hand" style={{ fontSize: 14 }}>{n}</span>
                    <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>· {[20,35,52][i]}m</span>
                  </div>
                  <Lines count={2} />
                </div>
              </div>
            ))}
            {/* Reply box */}
            <div className="wf-card dashed" style={{ padding: 12, marginTop: 8, marginLeft: 48 }}>
              <span className="wf-mono" style={{ fontSize: 11, color: 'var(--ink-3)' }}>返信を書く…</span>
            </div>
          </div>
        </main>
      </div>
    </Frame>
  );
}

function DetailC() {
  // V3: Split — post on left, reactions panel on right
  return (
    <Frame label="C · reactions split">
      <div style={{ display: 'flex', height: '100%' }}>
        <Sidebar active="home" compact />
        <main className="wf-grow wf-col" style={{ minWidth: 0, padding: 16, overflow: 'hidden' }}>
          <div className="wf-row" style={{ marginBottom: 10 }}>
            <button className="wf-btn sm ghost">← Esc</button>
            <span className="wf-label" style={{ marginLeft: 'auto' }}>POST · /h/2k9w</span>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: '1.6fr 1fr', gap: 14, overflow: 'hidden' }}>
            {/* Left: post + replies */}
            <div className="wf-stack" style={{ gap: 10 }}>
              <div className="wf-card raised" style={{ padding: 14 }}>
                <div className="wf-row" style={{ marginBottom: 8 }}>
                  <div className="wf-av accent" />
                  <div className="wf-col wf-grow">
                    <span className="wf-hand" style={{ fontSize: 18 }}>Hana K.</span>
                    <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>「@hana@mithic.social」</span>
                  </div>
                  <button className="wf-btn sm">フォロー</button>
                </div>
                <p style={{ fontSize: 15, lineHeight: 1.55, margin: '0 0 10px' }}>
                  決めない自由を残しておくこと。<span className="wf-tag">#wireframe</span>
                </p>
                <div className="wf-row" style={{ flexWrap: 'wrap', gap: 4, marginBottom: 8 }}>
                  {['🔥 24','✨ 18','👀 7','💬 4','+'].map(r => (
                    <span key={r} className="wf-pill" style={{ fontFamily: 'var(--font-body)', fontSize: 11, padding: '3px 8px' }}>{r}</span>
                  ))}
                </div>
                <div className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>09:42 · 2026·05·10 · web</div>
              </div>

              {/* Reply composer */}
              <div className="wf-card dashed" style={{ padding: 10 }}>
                <div className="wf-row" style={{ alignItems: 'flex-start', gap: 8 }}>
                  <div className="wf-av sm accent2" />
                  <div className="wf-grow">
                    <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>↪ @hana への返信</span>
                    <p className="wf-hand" style={{ fontSize: 14, color: 'var(--ink-3)', margin: '4px 0 8px' }}>返信を書く…</p>
                    <div className="wf-spread">
                      <div className="wf-row" style={{ gap: 4 }}>
                        <button className="wf-btn sm ghost">📎</button>
                        <button className="wf-btn sm ghost">😊</button>
                      </div>
                      <button className="wf-btn sm accent">返信</button>
                    </div>
                  </div>
                </div>
              </div>

              {/* Replies */}
              <div className="wf-row" style={{ marginTop: 2 }}>
                <span className="wf-label">[ 返信 / REPLIES · 3 ]</span>
                <span style={{ flex: 1 }} />
                <span className="wf-mono" style={{ fontSize: 10, color: 'var(--ink-3)' }}>新しい順 ▾</span>
              </div>
              {[
                ['Riku M.','@riku','20m','同感。決めない自由を残したい。'],
                ['Aya T.','@aya','35m','余白の取り方、もう少し聞きたい'],
                ['Ken S.','@ken_s','1h','最近ずっと考えてた話だ。'],
              ].map(([n,h,t,txt], i) => (
                <div key={i} className="wf-row" style={{ alignItems: 'flex-start', gap: 10, paddingLeft: 12, borderLeft: '1.5px dashed var(--line-soft)' }}>
                  <div className={`wf-av sm ${i === 0 ? 'accent2' : ''}`} />
                  <div className="wf-card wf-grow" style={{ padding: 10 }}>
                    <div className="wf-spread" style={{ marginBottom: 4 }}>
                      <div className="wf-row" style={{ gap: 6 }}>
                        <span className="wf-hand" style={{ fontSize: 14 }}>{n}</span>
                        <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>「{h}」 · {t}</span>
                      </div>
                    </div>
                    <p style={{ fontSize: 12.5, margin: '0 0 6px', lineHeight: 1.5 }}>{txt}</p>
                    <div className="wf-row wf-mono" style={{ gap: 10, fontSize: 9, color: 'var(--ink-3)' }}>
                      <span>↪ 返信</span>
                      <span>＋ リアクション</span>
                      <span>↻ 引用</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Right: reactions panel */}
            <div className="wf-stack" style={{ gap: 10 }}>
              <div className="wf-card" style={{ padding: 12 }}>
                <div className="wf-label" style={{ marginBottom: 8 }}>[ リアクション ]</div>
                <div className="wf-stack" style={{ gap: 6 }}>
                  {['🔥', '✨', '👀', '💬'].map((e, i) => (
                    <div key={e} className="wf-spread">
                      <div className="wf-row" style={{ gap: 8 }}>
                        <span style={{ fontSize: 14 }}>{e}</span>
                        <div className="wf-av sm" />
                        <span className="wf-mono" style={{ fontSize: 10 }}>@u{i}</span>
                      </div>
                      <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>{i+1}m</span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="wf-card" style={{ padding: 12 }}>
                <div className="wf-spread" style={{ marginBottom: 10 }}>
                  <span className="wf-label">[ 引用 · 12 ]</span>
                  <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>すべて表示 →</span>
                </div>
                <div className="wf-stack" style={{ gap: 10 }}>
                  {[
                    ['Riku M.','@riku','7m','まさにこれ。決めない設計、もっと話したい。'],
                    ['Aya T.','@aya','22m','余白＝判断保留、というのが効く。'],
                    ['Ken S.','@ken_s','1h','保存。あとでブログに。'],
                  ].map(([n, h, t, txt], i) => (
                    <div key={i} className="wf-card" style={{ padding: 10, background: 'var(--card-2)' }}>
                      <div className="wf-row" style={{ gap: 6, marginBottom: 6 }}>
                        <div className="wf-av sm" />
                        <span className="wf-hand" style={{ fontSize: 13 }}>{n}</span>
                        <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>{h} · {t}</span>
                      </div>
                      <p style={{ fontSize: 11.5, margin: '0 0 6px', lineHeight: 1.45 }}>{txt}</p>
                      {/* Embedded original */}
                      <div style={{ borderLeft: '2px solid var(--accent)', paddingLeft: 8, marginTop: 6 }}>
                        <div className="wf-row" style={{ gap: 4, marginBottom: 2 }}>
                          <div className="wf-av sm accent" style={{ width: 14, height: 14, borderWidth: 1 }} />
                          <span className="wf-mono" style={{ fontSize: 9, color: 'var(--ink-3)' }}>@hana · 元投稿</span>
                        </div>
                        <p style={{ fontSize: 10, margin: 0, color: 'var(--ink-2)', lineHeight: 1.4, fontStyle: 'italic' }}>
                          “決めない自由を残しておくこと。”
                        </p>
                      </div>
                      <div className="wf-row wf-mono" style={{ gap: 10, fontSize: 9, color: 'var(--ink-3)', marginTop: 8 }}>
                        <span>♥ {3 + i * 2}</span>
                        <span>↪ {i + 1}</span>
                      </div>
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

Object.assign(window, { DetailA, DetailB, DetailC });
