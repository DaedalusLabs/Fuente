var CACHE_NAME = 'CRIB-v1';
var urlsToCache = [
    '/',
    '/img',
];


/* Start the service worker and cache all of the app's content */
// Install event: cache the URLs
self.addEventListener('install', function(_) {
    self.skipWaiting();
});

/* Serve cached content when offline */
self.addEventListener('fetch', function(event) {
    event.respondWith(
        caches.open(CACHE_NAME).then(async (cache) => {
            try {
                const response = await cache.match(event.request);
                if (response) {
                    return response;
                } else {
                    const networkResponse = await fetch(event.request);
                    cache.put(event.request, networkResponse.clone());
                    return networkResponse;
                }
            } catch (error) {
                console.error('Error in fetch handler:', error);
                throw error;
            }
        })
    );
});

