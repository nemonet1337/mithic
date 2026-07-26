// Mithic Service Worker (Workbox 7)
// Workbox を CDN から importScripts で読み込む

importScripts('https://storage.googleapis.com/workbox-cdn/releases/7.0.0/workbox-sw.js');

const { precaching, routing, strategies, expiration, backgroundSync } = workbox;

// ==================================================
// Precache (trunk build 時に自動生成されるリストで代替)
// 静的アセットをプリキャッシュ
// ==================================================
precaching.precacheAndRoute([
  { url: '/', revision: null },
  { url: '/offline.html', revision: null },
]);

// ==================================================
// Runtime Caching
// ==================================================

// API ルート: NetworkFirst (30秒タイムアウト、オフライン時キャッシュにフォールバック)
routing.registerRoute(
  ({ url }) => url.pathname.startsWith('/api/'),
  new strategies.NetworkFirst({
    cacheName: 'mithic-api-cache',
    networkTimeoutSeconds: 30,
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 100,
        maxAgeSeconds: 60 * 60, // 1時間
      }),
    ],
  })
);

// 画像: CacheFirst
routing.registerRoute(
  ({ request }) => request.destination === 'image',
  new strategies.CacheFirst({
    cacheName: 'mithic-images',
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 200,
        maxAgeSeconds: 7 * 24 * 60 * 60, // 7日
      }),
    ],
  })
);

// Google Fonts: StaleWhileRevalidate
routing.registerRoute(
  ({ url }) => url.origin === 'https://fonts.googleapis.com' || url.origin === 'https://fonts.gstatic.com',
  new strategies.StaleWhileRevalidate({
    cacheName: 'mithic-fonts',
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 30,
        maxAgeSeconds: 30 * 24 * 60 * 60, // 30日
      }),
    ],
  })
);

// JS / CSS / WASM: StaleWhileRevalidate
routing.registerRoute(
  ({ request }) =>
    request.destination === 'script' ||
    request.destination === 'style' ||
    request.url.endsWith('.wasm'),
  new strategies.StaleWhileRevalidate({
    cacheName: 'mithic-static',
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 50,
        maxAgeSeconds: 24 * 60 * 60, // 1日
      }),
    ],
  })
);

// ==================================================
// Offline Fallback
// ==================================================
routing.setCatchHandler(async ({ event }) => {
  if (event.request.destination === 'document') {
    return caches.match('/offline.html');
  }
  return Response.error();
});

// SW インストール時にオフラインページをキャッシュ
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open('mithic-offline').then((cache) => cache.add('/offline.html'))
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(clients.claim());
});
