// Mithic Service Worker (Workbox 7)
// HTML は常にネットワーク優先。古い index とハッシュ付き WASM の不整合を防ぐ。

importScripts('https://storage.googleapis.com/workbox-cdn/releases/7.0.0/workbox-sw.js');

const { routing, strategies, expiration } = workbox;
const CACHE_VERSION = 'mithic-sw-v2';

// ==================================================
// Navigation / document: NetworkFirst（ソフトリロードで古いシェルを出さない）
// ==================================================
routing.registerRoute(
  ({ request }) => request.mode === 'navigate' || request.destination === 'document',
  new strategies.NetworkFirst({
    cacheName: `${CACHE_VERSION}-pages`,
    networkTimeoutSeconds: 5,
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 10,
        maxAgeSeconds: 24 * 60 * 60,
      }),
    ],
  })
);

// ==================================================
// Runtime Caching
// ==================================================

// API: NetworkOnly（認証・書き込みをキャッシュしない）
routing.registerRoute(
  ({ url }) => url.pathname.startsWith('/api/'),
  new strategies.NetworkOnly()
);

// 画像: CacheFirst
routing.registerRoute(
  ({ request }) => request.destination === 'image',
  new strategies.CacheFirst({
    cacheName: `${CACHE_VERSION}-images`,
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 200,
        maxAgeSeconds: 7 * 24 * 60 * 60,
      }),
    ],
  })
);

// Google Fonts: StaleWhileRevalidate
routing.registerRoute(
  ({ url }) =>
    url.origin === 'https://fonts.googleapis.com' ||
    url.origin === 'https://fonts.gstatic.com',
  new strategies.StaleWhileRevalidate({
    cacheName: `${CACHE_VERSION}-fonts`,
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 30,
        maxAgeSeconds: 30 * 24 * 60 * 60,
      }),
    ],
  })
);

// JS / CSS: NetworkFirst（Trunk ハッシュ付き成果物の不整合を避ける）
routing.registerRoute(
  ({ request }) =>
    request.destination === 'script' || request.destination === 'style',
  new strategies.NetworkFirst({
    cacheName: `${CACHE_VERSION}-static`,
    networkTimeoutSeconds: 3,
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 50,
        maxAgeSeconds: 24 * 60 * 60,
      }),
    ],
  })
);

// WASM: NetworkFirst
routing.registerRoute(
  ({ request, url }) =>
    request.destination === 'wasm' || url.pathname.endsWith('.wasm'),
  new strategies.NetworkFirst({
    cacheName: `${CACHE_VERSION}-wasm`,
    networkTimeoutSeconds: 5,
    plugins: [
      new expiration.ExpirationPlugin({
        maxEntries: 10,
        maxAgeSeconds: 24 * 60 * 60,
      }),
    ],
  })
);

// ==================================================
// Offline Fallback（document のみ）
// ==================================================
routing.setCatchHandler(async ({ event, error }) => {
  if (event.request.destination === 'document' || event.request.mode === 'navigate') {
    const offline = await caches.match('/offline.html');
    if (offline) return offline;
  }
  throw error || new Error('Network request failed');
});

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(`${CACHE_VERSION}-offline`).then((cache) => cache.add('/offline.html'))
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys();
      await Promise.all(
        keys
          .filter((key) => !key.startsWith(CACHE_VERSION))
          .map((key) => caches.delete(key))
      );
      await self.clients.claim();
    })()
  );
});

// ==================================================
// Web Push
// ==================================================
self.addEventListener('push', (event) => {
  let data = { title: 'Mithic', body: '', url: '/notifications', tag: 'mithic' };
  try {
    if (event.data) {
      const parsed = event.data.json();
      data = { ...data, ...parsed };
    }
  } catch (_) {
    try {
      data.body = event.data ? event.data.text() : '';
    } catch (_) {}
  }
  event.waitUntil(
    self.registration.showNotification(data.title || 'Mithic', {
      body: data.body || '',
      tag: data.tag || 'mithic',
      data: { url: data.url || '/notifications' },
      icon: '/icon-192.png',
      badge: '/icon-192.png',
    })
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const url = (event.notification.data && event.notification.data.url) || '/notifications';
  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clients) => {
      for (const c of clients) {
        if ('focus' in c) {
          c.navigate(url);
          return c.focus();
        }
      }
      if (self.clients.openWindow) {
        return self.clients.openWindow(url);
      }
    })
  );
});
