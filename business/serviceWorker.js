const CACHE_NAME = 'fuente-cache';
const CACHE_STORAGE = 'etag-cache';
const ROOT_URL = '/';
const INDEX_HTML = '/index.html';

const urlsToCache = [
    ROOT_URL,
    INDEX_HTML, // Cache the main entry HTML
];

self.addEventListener('install', (event) => {
    self.skipWaiting();
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            // Cache the root index.html and other essential assets during installation
            return cache.addAll(urlsToCache);
        })
    );
});

self.addEventListener('fetch', (event) => {
    if (event.request.mode === 'navigate') {
        event.respondWith(
            (async () => {
                try {

                    const cache = await caches.open(CACHE_NAME);
                    const cachedResponse = await cache.match(INDEX_HTML);

                    if (cachedResponse) {
                        const etag = cachedResponse.headers.get('ETag');
                        const headers = new Headers();
                        if (etag) {
                            headers.set('If-None-Match', etag); // Add If-None-Match header
                        }

                        const networkResponse = await fetch(event.request, { headers });

                        if (networkResponse.status === 304) {
                            return cachedResponse; // Use cached version if not modified
                        } else if (networkResponse.ok) {
                            await cache.put(INDEX_HTML, networkResponse.clone()); // Update cache
                            return networkResponse;
                        }
                    }

                    // Fallback to fetching and caching
                    const networkResponse = await fetch(event.request);
                    if (networkResponse.ok) {
                        const etag = networkResponse.headers.get('ETag');
                        if (etag) {
                            await cache.put(INDEX_HTML, networkResponse.clone()); // Update cache
                        }
                    }
                    return networkResponse;
                } catch (error) {
                    // Serve cached version if there's an error (e.g., offline)
                    const cache = await caches.open(CACHE_NAME);
                    return cache.match(INDEX_HTML);
                }
            })()
        );
    } else {
        // For non-navigation requests (like assets), serve from the cache first
        event.respondWith(
            caches.match(event.request).then((cachedResponse) => {
                return cachedResponse || fetch(event.request); // Return cached or fetch from network
            })
        );
    }
});

// Optional: Activate the service worker and clear old caches when a new version is available
self.addEventListener('activate', (event) => {
    const cacheWhitelist = [CACHE_NAME]; // Define the caches we want to keep
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames.map((cacheName) => {
                    if (!cacheWhitelist.includes(cacheName)) {
                        return caches.delete(cacheName); // Delete old caches
                    }
                })
            );
        })
    );
});

