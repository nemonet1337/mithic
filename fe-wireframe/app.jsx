// Main app — DesignCanvas + Tweaks

const { useEffect } = React;

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "dark": false
}/*EDITMODE-END*/;

function App() {
  const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);

  // Apply dark/light to all artboards by toggling a class on documentElement
  useEffect(() => {
    document.documentElement.classList.toggle('dark', !!t.dark);
    document.body.style.background = t.dark ? '#0e0c08' : '#f0eee9';
  }, [t.dark]);

  // Wrap each variation in Frame already does the .wf class; the .dark class
  // on documentElement cascades since CSS variables are scoped on .dark.
  // To make .dark cascade into Frame's .wf root, we re-apply it inside Frame
  // when document.documentElement has it. Simpler: stamp .dark onto every .wf.
  useEffect(() => {
    document.querySelectorAll('.wf').forEach(el => {
      el.classList.toggle('dark', !!t.dark);
    });
  });

  // Chosen variations: C for everything except Compose (A)
  const screens = [
    { id: 'home',     title: 'ホーム / フィード',  Comp: HomeC,     pick: 'C · timeline rail' },
    { id: 'detail',   title: '投稿詳細',           Comp: DetailC,   pick: 'C · reactions split' },
    { id: 'compose',  title: '投稿作成',           Comp: ComposeA,  pick: 'A · centered modal' },
    { id: 'profile',  title: 'プロフィール',       Comp: ProfileC,  pick: 'C · editorial' },
    { id: 'search',   title: '検索 / 発見',        Comp: SearchC,   pick: 'C · command palette' },
    { id: 'notif',    title: '通知',               Comp: NotifC,    pick: 'C · activity stream' },
    { id: 'dm',       title: 'DM',                Comp: DMC,       pick: 'C · floating windows' },
    { id: 'settings', title: '設定',               Comp: SettingsC, pick: 'C · single page' },
  ];

  const W = 880, H = 620;

  return (
    <>
      <DesignCanvas>
        <DCSection id="picks" title="mithic / 採用案" subtitle="C を基本、投稿作成のみ A">
          {screens.map(s => (
            <DCArtboard
              key={s.id}
              id={`pick-${s.id}`}
              label={`${s.title} — ${s.pick}`}
              width={W}
              height={H}
            >
              <s.Comp />
            </DCArtboard>
          ))}
        </DCSection>
      </DesignCanvas>
      <TweaksPanel title="Tweaks">
        <TweakSection label="Theme" />
        <TweakToggle
          label="Dark mode"
          value={t.dark}
          onChange={(v) => setTweak('dark', v)}
        />
      </TweaksPanel>
    </>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
